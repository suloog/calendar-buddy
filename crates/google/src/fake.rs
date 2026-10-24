//! Fakes for the tests: Google (its consent screen, as seen through the browser, its token
//! service, user info, the Calendar API and the People API's searches) and the secret store. Nothing here leaves
//! `127.0.0.1`.

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use calendar_core::accounts::Account;
use calendar_core::platform::{Browser, SecretStore, SecretStoreError};
use chrono::{DateTime, NaiveDate, NaiveDateTime, SecondsFormat, Utc};
use serde_json::{Value, json};
use url::Url;

use crate::{Endpoints, challenge};

pub const CLIENT_ID: &str = "test-client.apps.googleusercontent.com";
pub const ACCOUNT_ID: &str = "107691503500061507151";
pub const EMAIL: &str = "owner@example.com";
/// Another account signed in to the same browser; a personal one.
pub const OTHER_ID: &str = "100000000000000000001";
pub const OTHER_EMAIL: &str = "other@example.com";

/// How the user answers one of Google's consent screens.
#[derive(Clone, Copy)]
pub enum Reply {
    /// Consents, leaving these of the asked scopes unticked.
    Consent(&'static [&'static str]),
    /// Google sends this error back to the app.
    Error(&'static str),
    /// Consents to everything, but with another account picked in Google's account chooser.
    OtherAccount,
    /// Nothing comes back: the browser was closed, or Google stopped on its own error page.
    Nothing,
}

#[derive(Default)]
struct State {
    replies: VecDeque<Reply>,
    /// The account's Workspace domain; `None` for a personal account.
    hd: Option<&'static str>,
    /// Every authorization URL the browser was sent to.
    opened: Vec<Url>,
    /// Issued codes: the PKCE challenge they were asked with, the scopes they grant and the
    /// account they belong to.
    codes: HashMap<String, (String, String, &'static str)>,
    /// Issued access tokens and their accounts.
    access: HashMap<String, &'static str>,
    /// Everything the account has granted so far.
    granted: Vec<String>,
    /// The token service refuses every code with this error.
    token_error: Option<&'static str>,
    /// Issued refresh tokens and their accounts; revoking access removes them.
    refresh: HashMap<String, &'static str>,
    /// Every refresh token the app asked to refresh.
    refreshed: Vec<String>,
    /// Every request to the token service, user info, revocation and the Calendar API: method
    /// and path with the query.
    requests: Vec<String>,
    /// Each account's calendar list entries, as Google sends them.
    calendars: Vec<(&'static str, Value)>,
    /// The events of each account's calendars, as Google sends them: account, calendar ID, event
    /// and the change that last touched it. A deleted event stays as Google sends it then.
    events: Vec<(&'static str, String, Value, u64)>,
    /// Changes so far; a synchronization state is the number of changes it has seen.
    changes: u64,
    /// Google no longer accepts the synchronization states it gave out.
    states_expired: bool,
    /// Calendars whose events Google refuses to send.
    failing: Vec<String>,
    /// Google refuses to send the calendar lists.
    list_failing: bool,
    /// Calendars whose events Google refuses with a rate limit, and the HTTP status it answers.
    limited: Vec<(String, &'static str)>,
    /// Google cannot be reached: every connection is closed unanswered.
    unreachable: bool,
    /// Each account's default event length in minutes, where the user set one in Google.
    default_lengths: Vec<(&'static str, u32)>,
    /// Every patch of an event the app sent: the version it was sent against and the patch.
    patched: Vec<(String, Value)>,
    /// People the People API finds for each account: the search that finds them
    /// (`people:searchContacts`, `otherContacts:search` or `people:searchDirectoryPeople`) and
    /// the `person`.
    people: Vec<(&'static str, &'static str, Value)>,
    /// The contact searches warmed up for each account with an empty search, as Google asks;
    /// until then they find nobody.
    warmed: Vec<(&'static str, String)>,
    /// The addresses whose calendars share their free/busy, and when they are busy then (RFC 3339);
    /// Google knows nothing of the others.
    busy: Vec<(String, Vec<(String, String)>)>,
    /// Every `freeBusy` query the app sent.
    busy_queries: Vec<Value>,
}

impl State {
    /// Keeps the event as a new change with a new version (ETag), replacing the one with its ID.
    fn put_event(&mut self, account: &'static str, calendar: &str, mut event: Value) -> Value {
        self.changes += 1;
        let change = self.changes;
        event["etag"] = json!(format!("\"{change}\""));
        self.events.retain(|(owner, c, e, _)| {
            !(*owner == account && c == calendar && e["id"] == event["id"])
        });
        self.events
            .push((account, calendar.to_string(), event.clone(), change));
        event
    }

    /// The event of the account's calendar with the ID, deleted ones included.
    fn kept_event(&self, account: &str, calendar: &str, id: &str) -> Option<Value> {
        self.events
            .iter()
            .find(|(owner, c, e, _)| *owner == account && c == calendar && e["id"] == id)
            .map(|(_, _, event, _)| event.clone())
    }

    /// The event of the account's calendar with the ID, unless deleted.
    fn find_event(&self, account: &str, calendar: &str, id: &str) -> Option<Value> {
        self.kept_event(account, calendar, id)
            .filter(|event| !deleted(event))
    }

    /// The event of the account's calendar with the ID, or the occurrence of a series Google
    /// names by it, `<series>_<start>`, as Google makes it when it is first changed: the series'
    /// fields at the occurrence's times, with the version of the series.
    fn event_or_occurrence(&self, account: &str, calendar: &str, id: &str) -> Option<Value> {
        if let Some(event) = self.kept_event(account, calendar, id) {
            return Some(event);
        }
        let (series, start) = id.rsplit_once('_')?;
        let mut event = self
            .find_event(account, calendar, series)
            .filter(|series| series.get("recurrence").is_some())?;
        let (original, end) = match NaiveDateTime::parse_from_str(start, "%Y%m%dT%H%M%SZ") {
            Ok(at) => {
                let instant = |when: &Value| {
                    DateTime::parse_from_rfc3339(when["dateTime"].as_str().unwrap()).unwrap()
                };
                let length = instant(&event["end"]) - instant(&event["start"]);
                let zone = event["start"]["timeZone"].clone();
                let when = |at: DateTime<Utc>| {
                    json!({
                        "dateTime": at.to_rfc3339_opts(SecondsFormat::Secs, true),
                        "timeZone": zone,
                    })
                };
                (when(at.and_utc()), when(at.and_utc() + length))
            }
            Err(_) => {
                let date = |when: &Value| {
                    NaiveDate::parse_from_str(when["date"].as_str().unwrap(), "%Y-%m-%d").unwrap()
                };
                let length = date(&event["end"]) - date(&event["start"]);
                let day = NaiveDate::parse_from_str(start, "%Y%m%d").ok()?;
                (
                    json!({"date": day.to_string()}),
                    json!({"date": (day + length).to_string()}),
                )
            }
        };
        let fields = event.as_object_mut().unwrap();
        fields.remove("recurrence");
        fields.insert("id".into(), json!(id));
        fields.insert("recurringEventId".into(), json!(series));
        fields.insert("originalStartTime".into(), original.clone());
        fields.insert("start".into(), original);
        fields.insert("end".into(), end);
        Some(event)
    }

    /// Why the account may not change the calendar's events, as Google answers; `None` when it
    /// may.
    fn refuses_changes(&self, account: &str, calendar: &str) -> Option<(&'static str, String)> {
        let entry = self
            .calendars
            .iter()
            .find(|(owner, entry)| *owner == account && entry["id"] == calendar);
        let Some((_, entry)) = entry else {
            return Some(not_found());
        };
        if !matches!(entry["accessRole"].as_str(), Some("owner" | "writer")) {
            return Some((
                "403 Forbidden",
                r#"{"error": {"code": 403, "message": "You need to have writer access to this calendar.",
                    "errors": [{"domain": "calendar", "reason": "requiredAccessLevel"}]}}"#
                    .to_string(),
            ));
        }
        if self.failing.iter().any(|failing| failing == calendar) {
            return Some((
                "500 Internal Server Error",
                r#"{"error": {"code": 500, "message": "Backend Error"}}"#.to_string(),
            ));
        }
        None
    }
}

fn not_found() -> (&'static str, String) {
    (
        "404 Not Found",
        r#"{"error": {"code": 404, "message": "Not Found"}}"#.to_string(),
    )
}

/// How many items the fake sends per page; Google may send fewer than asked for.
const CALENDARS_PER_PAGE: usize = 2;
const EVENTS_PER_PAGE: usize = 100;

pub struct FakeGoogle {
    pub endpoints: Endpoints,
    state: Arc<Mutex<State>>,
}

impl FakeGoogle {
    pub fn start(hd: Option<&'static str>, replies: &[Reply]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(State {
            replies: replies.iter().copied().collect(),
            hd,
            ..State::default()
        }));
        let server = state.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                serve(stream, &server);
            }
        });
        FakeGoogle {
            endpoints: Endpoints {
                auth: format!("{base}/auth"),
                token: format!("{base}/token"),
                userinfo: format!("{base}/userinfo"),
                revoke: format!("{base}/revoke"),
                calendar: format!("{base}/calendar/v3"),
                people: format!("{base}/people/v1"),
            },
            state,
        }
    }

    pub fn refuse_tokens(&self, error: &'static str) {
        self.state.lock().unwrap().token_error = Some(error);
    }

    pub fn opened(&self) -> Vec<Url> {
        self.state.lock().unwrap().opened.clone()
    }

    /// The user revokes the app's access in the account's Google security settings.
    pub fn revoke(&self, account: &str) {
        self.state
            .lock()
            .unwrap()
            .refresh
            .retain(|_, owner| *owner != account);
    }

    pub fn refreshed(&self) -> Vec<String> {
        self.state.lock().unwrap().refreshed.clone()
    }

    /// Whether the app still has access to the account, i.e. is listed in its third-party apps.
    pub fn has_access(&self, account: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .refresh
            .values()
            .any(|owner| *owner == account)
    }

    pub fn requests(&self) -> Vec<String> {
        self.state.lock().unwrap().requests.clone()
    }

    /// The account has signed in before: Google knows the refresh token returned.
    pub fn signed_in(&self, account: &'static str) -> String {
        let token = format!("refresh-{account}");
        self.state
            .lock()
            .unwrap()
            .refresh
            .insert(token.clone(), account);
        token
    }

    /// A calendar on the account's calendar list, as Google's `calendarList` entry.
    pub fn add_calendar(&self, account: &'static str, entry: Value) {
        self.state.lock().unwrap().calendars.push((account, entry));
    }

    /// The calendar leaves the account's calendar list: deleted, or no longer shared with it.
    pub fn remove_calendar(&self, account: &'static str, calendar: &str) {
        self.state
            .lock()
            .unwrap()
            .calendars
            .retain(|(owner, entry)| !(*owner == account && entry["id"] == calendar));
    }

    /// An event of one of the account's calendars, as Google's `events` resource; it replaces
    /// the event with its ID, as a change made elsewhere.
    pub fn add_event(&self, account: &'static str, calendar: &str, event: Value) {
        self.state
            .lock()
            .unwrap()
            .put_event(account, calendar, event);
    }

    /// The patches of events the app sent, in order: the version each was sent against and the
    /// patch.
    pub fn patched(&self) -> Vec<(String, Value)> {
        self.state.lock().unwrap().patched.clone()
    }

    /// The event is deleted elsewhere.
    pub fn delete_event(&self, account: &'static str, calendar: &str, id: &str) {
        self.add_event(
            account,
            calendar,
            json!({"kind": "calendar#event", "id": id, "status": "cancelled"}),
        );
    }

    /// The event as Google keeps it, e.g. one the app created.
    pub fn event(&self, account: &str, calendar: &str, id: &str) -> Option<Value> {
        self.state.lock().unwrap().kept_event(account, calendar, id)
    }

    /// The user sets the length of new events in the account's Google Calendar settings.
    pub fn set_default_length(&self, account: &'static str, minutes: u32) {
        let mut state = self.state.lock().unwrap();
        state.default_lengths.retain(|(owner, _)| *owner != account);
        state.default_lengths.push((account, minutes));
    }

    /// Someone in the account's Google Contacts.
    pub fn add_contact(&self, account: &'static str, name: &str, email: &str) {
        self.add_person(account, "people:searchContacts", name, email);
    }

    /// Someone the account has e-mailed, in its "other contacts".
    pub fn add_other_contact(&self, account: &'static str, name: &str, email: &str) {
        self.add_person(account, "otherContacts:search", name, email);
    }

    /// A colleague in the directory of the account's organization.
    pub fn add_colleague(&self, account: &'static str, name: &str, email: &str) {
        self.add_person(account, "people:searchDirectoryPeople", name, email);
    }

    fn add_person(&self, account: &'static str, search: &'static str, name: &str, email: &str) {
        let person = json!({
            "resourceName": format!("people/{}", email),
            "names": [{"displayName": name}],
            "emailAddresses": [{"value": email}],
        });
        (self.state.lock().unwrap().people).push((account, search, person));
    }

    /// The address's calendar shares its free/busy, busy at these times (RFC 3339, from and to).
    pub fn share_busy(&self, email: &str, busy: &[(&str, &str)]) {
        let busy = (busy.iter())
            .map(|(from, to)| (from.to_string(), to.to_string()))
            .collect();
        (self.state.lock().unwrap().busy).push((email.to_string(), busy));
    }

    /// Every `freeBusy` query the app sent.
    pub fn busy_queries(&self) -> Vec<Value> {
        self.state.lock().unwrap().busy_queries.clone()
    }

    /// The access tokens given out expire.
    pub fn expire_access_tokens(&self) {
        self.state.lock().unwrap().access.clear();
    }

    /// Google no longer accepts the synchronization states it gave out, and answers 410 Gone.
    pub fn expire_sync_tokens(&self) {
        self.state.lock().unwrap().states_expired = true;
    }

    /// The account's access to the calendar changes, as its owner shares it differently.
    pub fn set_access_role(&self, calendar: &str, role: &str) {
        for (_, entry) in &mut self.state.lock().unwrap().calendars {
            if entry["id"] == calendar {
                entry["accessRole"] = role.into();
            }
        }
    }

    /// Google refuses to send the calendar lists.
    pub fn fail_calendar_list(&self) {
        self.state.lock().unwrap().list_failing = true;
    }

    /// Google refuses to send the calendar's events.
    pub fn fail_calendar(&self, calendar: &str) {
        self.state
            .lock()
            .unwrap()
            .failing
            .push(calendar.to_string());
    }

    /// Google answers requests for the calendar's events with a rate limit: `429 Too Many
    /// Requests` or `403 Forbidden`, with the reason `rateLimitExceeded`.
    pub fn limit_calendar(&self, calendar: &str, status: &'static str) {
        self.state
            .lock()
            .unwrap()
            .limited
            .push((calendar.to_string(), status));
    }

    /// The connection to Google is lost: nothing is answered.
    pub fn disconnect(&self) {
        self.state.lock().unwrap().unreachable = true;
    }

    /// Google works again: it can be reached, and sends every calendar list and calendar.
    pub fn recover(&self) {
        let mut state = self.state.lock().unwrap();
        state.failing.clear();
        state.list_failing = false;
        state.limited.clear();
        state.unreachable = false;
    }
}

pub fn param(url: &Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

/// The browser shows Google's consent screen and the user answers it.
impl Browser for FakeGoogle {
    fn open(&self, url: &str) -> std::io::Result<()> {
        let url = Url::parse(url).unwrap();
        let mut state = self.state.lock().unwrap();
        state.opened.push(url.clone());
        let redirect = param(&url, "redirect_uri").unwrap();
        let back = param(&url, "state").unwrap();
        let reply = state.replies.pop_front().unwrap_or(Reply::Nothing);
        let query = match reply {
            Reply::OtherAccount => {
                let code = format!("code-{}", state.opened.len());
                let challenge = param(&url, "code_challenge").unwrap();
                let asked = param(&url, "scope").unwrap();
                state
                    .codes
                    .insert(code.clone(), (challenge, asked, OTHER_ID));
                format!("code={code}&state={back}")
            }
            Reply::Consent(unticked) => {
                let asked = param(&url, "scope").unwrap();
                for scope in asked.split(' ').filter(|s| !unticked.contains(s)) {
                    if !state.granted.iter().any(|g| g == scope) {
                        state.granted.push(scope.to_string());
                    }
                }
                let code = format!("code-{}", state.opened.len());
                let challenge = param(&url, "code_challenge").unwrap();
                let granted = state.granted.join(" ");
                state
                    .codes
                    .insert(code.clone(), (challenge, granted, ACCOUNT_ID));
                format!("code={code}&state={back}")
            }
            Reply::Error(error) => format!("error={error}&state={back}"),
            Reply::Nothing => return Ok(()),
        };
        // The browser follows the redirect while the app is still listening.
        std::thread::spawn(move || {
            let address = redirect.trim_start_matches("http://");
            let mut stream = TcpStream::connect(address).unwrap();
            write!(stream, "GET /?{query} HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
            let _ = stream.read_to_end(&mut Vec::new());
        });
        Ok(())
    }
}

/// Answers one request to the token service or user info.
fn serve(mut stream: TcpStream, state: &Mutex<State>) {
    let mut request = Vec::new();
    let mut buffer = [0; 4096];
    let (head, body) = loop {
        let read = stream.read(&mut buffer).unwrap_or(0);
        if read == 0 {
            // The network check only connects.
            return;
        }
        request.extend_from_slice(&buffer[..read]);
        let text = String::from_utf8_lossy(&request).to_string();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let length = head
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|n| n.trim().parse().unwrap())
                })
                .unwrap_or(0);
            if body.len() >= length {
                break (head.to_string(), body.to_string());
            }
        }
    };
    let mut state = state.lock().unwrap();
    if state.unreachable {
        return;
    }
    let request: Vec<&str> = head.split_whitespace().take(2).collect();
    state.requests.push(request.join(" "));
    if head.starts_with("POST /revoke") {
        let form: HashMap<String, String> = url::form_urlencoded::parse(body.as_bytes())
            .into_owned()
            .collect();
        // Revoking a token withdraws the app's whole grant on that account.
        let answer = match state.refresh.get(&form["token"]).copied() {
            Some(account) => {
                state.refresh.retain(|_, owner| *owner != account);
                state.access.retain(|_, owner| *owner != account);
                ("200 OK", "{}".to_string())
            }
            None => (
                "400 Bad Request",
                r#"{"error": "invalid_token", "error_description": "Token expired or revoked"}"#
                    .to_string(),
            ),
        };
        respond(&mut stream, answer);
        return;
    }
    let bearer = head
        .lines()
        .find_map(|line| line.split_once("Bearer "))
        .and_then(|(_, token)| state.access.get(token.trim()).copied());
    if head.starts_with("GET /people/v1/") {
        let answer = match bearer {
            Some(account) => people_api(&mut state, account, request[1]),
            None => (
                "401 Unauthorized",
                r#"{"error": {"code": 401, "message": "Invalid Credentials"}}"#.to_string(),
            ),
        };
        respond(&mut stream, answer);
        return;
    }
    let insert = head.starts_with("POST /calendar/v3/");
    let patch = head.starts_with("PATCH /calendar/v3/");
    let delete = head.starts_with("DELETE /calendar/v3/");
    if head.starts_with("GET /calendar/v3/") || insert || patch || delete {
        let if_match = head.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("if-match")
                .then(|| value.trim().to_string())
        });
        let answer = match bearer {
            Some(account) if patch => patch_event(&mut state, account, request[1], if_match, &body),
            Some(account) if delete => delete_event(&mut state, account, request[1], if_match),
            Some(_) if insert && request[1] == "/calendar/v3/freeBusy" => {
                free_busy(&mut state, &body)
            }
            Some(account) if insert && request[1].contains("/move?") => {
                move_event(&mut state, account, request[1])
            }
            Some(account) if insert => insert_event(&mut state, account, request[1], &body),
            Some(account) => calendar_api(&state, account, request[1]),
            None => (
                "401 Unauthorized",
                r#"{"error": {"code": 401, "message": "Invalid Credentials"}}"#.to_string(),
            ),
        };
        respond(&mut stream, answer);
        return;
    }
    let (status, json) = if head.starts_with("POST /token") {
        let form: HashMap<String, String> = url::form_urlencoded::parse(body.as_bytes())
            .into_owned()
            .collect();
        if form["grant_type"] == "refresh_token" {
            let token = &form["refresh_token"];
            state.refreshed.push(token.clone());
            let known = state.refresh.contains_key(token) && form["client_id"] == CLIENT_ID;
            let answer = if known {
                let account = state.refresh[token];
                state.access.insert(format!("access-{token}"), account);
                (
                    "200 OK",
                    format!(r#"{{"access_token": "access-{token}", "expires_in": 3599}}"#),
                )
            } else {
                (
                    "400 Bad Request",
                    r#"{"error": "invalid_grant", "error_description": "Token has been expired or revoked."}"#
                        .to_string(),
                )
            };
            respond(&mut stream, answer);
            return;
        }
        let issued = state.codes.remove(&form["code"]);
        match (state.token_error, issued) {
            (Some(error), _) => (
                "400 Bad Request",
                format!(r#"{{"error": "{error}", "error_description": "Refused by the fake"}}"#),
            ),
            (None, Some((asked, scope, account)))
                if challenge(&form["code_verifier"]) == asked
                    && form["client_id"] == CLIENT_ID
                    && form["grant_type"] == "authorization_code" =>
            {
                state
                    .access
                    .insert(format!("access-{}", form["code"]), account);
                state
                    .refresh
                    .insert(format!("refresh-{}", form["code"]), account);
                (
                    "200 OK",
                    format!(
                        r#"{{"access_token": "access-{0}", "refresh_token": "refresh-{0}",
                            "scope": "{scope}", "token_type": "Bearer", "expires_in": 3599}}"#,
                        form["code"]
                    ),
                )
            }
            _ => (
                "400 Bad Request",
                r#"{"error": "invalid_grant"}"#.to_string(),
            ),
        }
    } else if let Some(&sub) = head
        .starts_with("GET /userinfo")
        .then(|| head.lines().find_map(|line| line.split_once("Bearer ")))
        .flatten()
        .and_then(|(_, token)| state.access.get(token.trim()))
    {
        let (email, hd) = match sub {
            OTHER_ID => (OTHER_EMAIL, None),
            _ => (EMAIL, state.hd),
        };
        let hd = hd
            .map(|hd| format!(r#", "hd": "{hd}""#))
            .unwrap_or_default();
        (
            "200 OK",
            format!(r#"{{"sub": "{sub}", "email": "{email}", "name": "Owner"{hd}}}"#),
        )
    } else {
        (
            "401 Unauthorized",
            r#"{"error": {"code": 401, "message": "Invalid Credentials"}}"#.to_string(),
        )
    };
    respond(&mut stream, (status, json));
}

/// The Calendar API's calendar list and events lists, in pages.
fn calendar_api(state: &State, account: &str, target: &str) -> (&'static str, String) {
    let url = Url::parse(&format!("http://127.0.0.1{target}")).unwrap();
    let offset: usize = param(&url, "pageToken").map_or(0, |token| token.parse().unwrap());
    let segments = segments(&url);
    let (items, per_page): (Vec<&Value>, usize) = match segments.as_slice() {
        [_, _, users, me, settings, name]
            if users == "users" && me == "me" && settings == "settings" =>
        {
            if name != "defaultEventLength" {
                return (
                    "404 Not Found",
                    r#"{"error": {"code": 404, "message": "Not Found"}}"#.to_string(),
                );
            }
            let minutes = state
                .default_lengths
                .iter()
                .find(|(owner, _)| *owner == account)
                .map_or(60, |(_, minutes)| *minutes);
            let setting = json!({
                "kind": "calendar#setting",
                "id": name,
                "value": minutes.to_string(),
            });
            return ("200 OK", setting.to_string());
        }
        [_, _, users, me, list] if users == "users" && me == "me" && list == "calendarList" => {
            if state.list_failing {
                return (
                    "500 Internal Server Error",
                    r#"{"error": {"code": 500, "message": "Backend Error"}}"#.to_string(),
                );
            }
            (
                state
                    .calendars
                    .iter()
                    .filter(|(owner, _)| *owner == account)
                    .map(|(_, entry)| entry)
                    .collect(),
                CALENDARS_PER_PAGE,
            )
        }
        // `events.get`: one event, deleted ones included, as Google keeps them for a while, or an
        // occurrence of a series.
        [_, _, calendars, id, events, event] if calendars == "calendars" && events == "events" => {
            return match state.event_or_occurrence(account, id, event) {
                Some(event) => ("200 OK", event.to_string()),
                None => not_found(),
            };
        }
        [_, _, calendars, id, events] if calendars == "calendars" && events == "events" => {
            let listed = state
                .calendars
                .iter()
                .any(|(owner, entry)| *owner == account && entry["id"] == id.as_str());
            if !listed {
                return (
                    "404 Not Found",
                    r#"{"error": {"code": 404, "message": "Not Found"}}"#.to_string(),
                );
            }
            if state.failing.contains(id) {
                return (
                    "500 Internal Server Error",
                    r#"{"error": {"code": 500, "message": "Backend Error"}}"#.to_string(),
                );
            }
            if let Some((_, status)) = state.limited.iter().find(|(limited, _)| limited == id) {
                return (
                    status,
                    r#"{"error": {"errors": [{"domain": "usageLimits", "reason": "rateLimitExceeded",
                        "message": "Rate Limit Exceeded"}],
                        "message": "Rate Limit Exceeded"}}"#
                        .to_string(),
                );
            }
            let of_calendar = state
                .events
                .iter()
                .filter(|(owner, calendar, ..)| *owner == account && calendar == id);
            let events = match param(&url, "syncToken") {
                // What changed since the state, deleted events included; no time range.
                Some(_) if param(&url, "timeMin").is_some() || param(&url, "timeMax").is_some() => {
                    return (
                        "400 Bad Request",
                        r#"{"error": {"code": 400, "message": "Invalid sync token value."}}"#
                            .to_string(),
                    );
                }
                Some(_) if state.states_expired => {
                    return (
                        "410 Gone",
                        r#"{"error": {"code": 410, "message": "Sync token is no longer valid, a full sync is required."}}"#
                            .to_string(),
                    );
                }
                Some(since) => {
                    let since: u64 = since.strip_prefix("state-").unwrap().parse().unwrap();
                    of_calendar
                        .filter(|(.., change)| *change > since)
                        .map(|(_, _, event, _)| event)
                        .collect()
                }
                None => {
                    let instant = |name: &str| {
                        DateTime::parse_from_rfc3339(&param(&url, name).unwrap())
                            .unwrap()
                            .with_timezone(&Utc)
                    };
                    let (from, until) = (instant("timeMin"), instant("timeMax"));
                    of_calendar
                        .map(|(_, _, event, _)| event)
                        .filter(|event| !deleted(event) && in_window(event, from, until))
                        .collect()
                }
            };
            (events, EVENTS_PER_PAGE)
        }
        _ => {
            return (
                "404 Not Found",
                r#"{"error": {"code": 404, "message": "Not Found"}}"#.to_string(),
            );
        }
    };
    let page: Vec<&Value> = items.iter().skip(offset).take(per_page).copied().collect();
    let mut answer = json!({ "items": page });
    if offset + per_page < items.len() {
        answer["nextPageToken"] = json!((offset + per_page).to_string());
    } else if target.contains("/events") {
        answer["nextSyncToken"] = json!(format!("state-{}", state.changes));
    }
    ("200 OK", answer.to_string())
}

/// The People API's searches: the people whose name, a word of it or address starts with the
/// query, ignoring case; contact searches find nobody until warmed up with an empty query.
fn people_api(state: &mut State, account: &'static str, target: &str) -> (&'static str, String) {
    let url = Url::parse(&format!("http://127.0.0.1{target}")).unwrap();
    let Some(search) = segments(&url).last().cloned() else {
        return not_found();
    };
    let query = param(&url, "query").unwrap_or_default().to_lowercase();
    let directory = search == "people:searchDirectoryPeople";
    if directory && param(&url, "sources").is_none() {
        return (
            "400 Bad Request",
            r#"{"error": {"code": 400, "message": "Request must specify sources."}}"#.to_string(),
        );
    }
    let warmed = |state: &State| {
        (state.warmed.iter()).any(|(owner, warmed)| *owner == account && *warmed == search)
    };
    if query.is_empty() {
        if !directory && !warmed(state) {
            state.warmed.push((account, search.clone()));
        }
        return ("200 OK", "{}".to_string());
    }
    let found: Vec<Value> = (state.people.iter())
        .filter(|(owner, of, _)| *owner == account && *of == search)
        .map(|(_, _, person)| person.clone())
        .filter(|person| {
            let name = person["names"][0]["displayName"]
                .as_str()
                .unwrap_or_default()
                .to_lowercase();
            let email = person["emailAddresses"][0]["value"]
                .as_str()
                .unwrap_or_default()
                .to_lowercase();
            name.split_whitespace().any(|word| word.starts_with(&query))
                || email.starts_with(&query)
        })
        .filter(|_| directory || warmed(state))
        .collect();
    let answer = if directory {
        json!({"people": found})
    } else {
        let results: Vec<Value> = (found.into_iter())
            .map(|person| json!({"person": person}))
            .collect();
        json!({"results": results})
    };
    ("200 OK", answer.to_string())
}

/// The Calendar API's `freeBusy.query`: the busy times within the query's span of each address
/// whose calendar shares them, cut to the span; `notFound` for the others.
fn free_busy(state: &mut State, body: &str) -> (&'static str, String) {
    let query: Value = serde_json::from_str(body).unwrap();
    state.busy_queries.push(query.clone());
    let at = |text: &str| {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    };
    let (from, until) = (
        at(query["timeMin"].as_str().unwrap()),
        at(query["timeMax"].as_str().unwrap()),
    );
    let mut calendars = serde_json::Map::new();
    for item in query["items"].as_array().unwrap() {
        let email = item["id"].as_str().unwrap();
        let shared = match state.busy.iter().find(|(owner, _)| owner == email) {
            Some((_, busy)) => {
                let busy: Vec<Value> = (busy.iter())
                    .map(|(start, end)| (at(start).max(from), at(end).min(until)))
                    .filter(|(start, end)| start < end)
                    .map(|(start, end)| {
                        let text = |t: DateTime<Utc>| t.to_rfc3339_opts(SecondsFormat::Secs, true);
                        json!({"start": text(start), "end": text(end)})
                    })
                    .collect();
                json!({"busy": busy})
            }
            None => json!({"errors": [{"domain": "global", "reason": "notFound"}], "busy": []}),
        };
        calendars.insert(email.to_string(), shared);
    }
    let answer = json!({
        "kind": "calendar#freeBusy",
        "timeMin": query["timeMin"],
        "timeMax": query["timeMax"],
        "calendars": calendars,
    });
    ("200 OK", answer.to_string())
}

/// The Calendar API's `events.insert`: the event is kept as a change, with an ID Google gives,
/// in a calendar the account can change; out of office and focus time only as Google takes them.
fn insert_event(
    state: &mut State,
    account: &'static str,
    target: &str,
    body: &str,
) -> (&'static str, String) {
    let url = Url::parse(&format!("http://127.0.0.1{target}")).unwrap();
    let segments = segments(&url);
    let [_, _, calendars, id, events] = segments.as_slice() else {
        return (
            "404 Not Found",
            r#"{"error": {"code": 404, "message": "Not Found"}}"#.to_string(),
        );
    };
    if calendars != "calendars" || events != "events" {
        return not_found();
    }
    if let Some(refusal) = state.refuses_changes(account, id) {
        return refusal;
    }
    let mut event: Value = serde_json::from_str(body).unwrap();
    // Out of office and focus time: only in a primary calendar, and only timed.
    if matches!(
        event["eventType"].as_str(),
        Some("outOfOffice" | "focusTime")
    ) {
        let primary = state.calendars.iter().any(|(owner, entry)| {
            *owner == account && entry["id"] == id.as_str() && entry["primary"] == true
        });
        if !primary || event["start"].get("date").is_some() {
            return (
                "400 Bad Request",
                r#"{"error": {"code": 400, "message": "Invalid event type for this calendar or times.",
                    "errors": [{"domain": "calendar", "reason": "invalid"}]}}"#
                    .to_string(),
            );
        }
    }
    if let Err(refusal) = conference(state, account, id, &url, &mut event) {
        return refusal;
    }
    event["kind"] = json!("calendar#event");
    event["id"] = json!(format!("created-{}", state.changes + 1));
    event["status"] = json!("confirmed");
    mark_self(&mut event, id);
    let event = state.put_event(account, id, event);
    ("200 OK", event.to_string())
}

/// Google Meet as the Calendar API makes it for an event inserted or patched: `conferenceData` is
/// taken only with `conferenceDataVersion=1`, and a `createRequest` for `hangoutsMeet` gets a
/// call, unless the calendar's `conferenceProperties` do not allow it (400).
fn conference(
    state: &State,
    account: &str,
    calendar: &str,
    url: &Url,
    event: &mut Value,
) -> Result<(), (&'static str, String)> {
    if param(url, "conferenceDataVersion").as_deref() != Some("1") {
        if let Some(fields) = event.as_object_mut() {
            fields.remove("conferenceData");
        }
        return Ok(());
    }
    // A call copied with its event stays the same call.
    let asked = &event["conferenceData"]["createRequest"];
    let Some(kind) = asked["conferenceSolutionKey"]["type"].as_str() else {
        return Ok(());
    };
    if event["conferenceData"]["entryPoints"].is_array() {
        return Ok(());
    }
    let entry = (state.calendars.iter())
        .find(|(owner, entry)| *owner == account && entry["id"] == calendar)
        .map(|(_, entry)| entry);
    let allowed = entry
        .and_then(|entry| {
            entry["conferenceProperties"]["allowedConferenceSolutionTypes"].as_array()
        })
        .is_none_or(|types| types.iter().any(|t| t == kind));
    if kind != "hangoutsMeet" || !allowed {
        return Err((
            "400 Bad Request",
            r#"{"error": {"code": 400, "message": "Invalid conference type value.",
                "errors": [{"domain": "global", "reason": "invalid",
                "message": "Invalid conference type value."}]}}"#
                .to_string(),
        ));
    }
    let code = format!("fak-{:04}-meet", state.changes + 1);
    event["conferenceData"] = json!({
        "createRequest": {
            "requestId": asked["requestId"],
            "conferenceSolutionKey": {"type": "hangoutsMeet"},
            "status": {"statusCode": "success"}
        },
        "entryPoints": [{
            "entryPointType": "video",
            "uri": format!("https://meet.google.com/{code}"),
            "label": format!("meet.google.com/{code}")
        }],
        "conferenceSolution": {"key": {"type": "hangoutsMeet"}, "name": "Google Meet"},
        "conferenceId": code
    });
    Ok(())
}

/// The Calendar API's `events.patch`, also of a series' occurrence: the fields sent replace those
/// kept, a null one goes; only when the version sent in `If-Match` is the one kept, and in a
/// guest's copy the organizer does not let guests change only the guest's own answer. A deleted
/// event is kept cancelled, and patching its status to confirmed brings it back.
fn patch_event(
    state: &mut State,
    account: &'static str,
    target: &str,
    if_match: Option<String>,
    body: &str,
) -> (&'static str, String) {
    let url = Url::parse(&format!("http://127.0.0.1{target}")).unwrap();
    let [_, _, calendars, calendar, events, id] = segments(&url).try_into().unwrap_or_default();
    if calendars != "calendars" || events != "events" {
        return not_found();
    }
    if let Some(refusal) = state.refuses_changes(account, &calendar) {
        return refusal;
    }
    let Some(mut event) = state.event_or_occurrence(account, &calendar, &id) else {
        return not_found();
    };
    if if_match.as_ref().is_some_and(|etag| *etag != event["etag"]) {
        return precondition_failed();
    }
    let mut patch: Value = serde_json::from_str(body).unwrap();
    if let Err(refusal) = conference(state, account, &calendar, &url, &mut patch) {
        return refusal;
    }
    // A guest's copy of an event whose organizer does not let guests change it: only the guest's
    // own answer may change.
    if event["organizer"].is_object()
        && event["organizer"]["self"] != true
        && event["guestsCanModify"] != true
        && !answers_only(&event, &patch)
    {
        return (
            "403 Forbidden",
            r#"{"error": {"code": 403,
                "message": "Shared properties can only be changed by the organizer of the event.",
                "errors": [{"domain": "calendar", "reason": "forbiddenForNonOrganizer"}]}}"#
                .to_string(),
        );
    }
    state
        .patched
        .push((if_match.unwrap_or_default(), patch.clone()));
    merge(&mut event, &patch);
    mark_self(&mut event, &calendar);
    let event = state.put_event(account, &calendar, event);
    ("200 OK", event.to_string())
}

/// The Calendar API's `events.delete`, also of a series' occurrence: the event stays, cancelled,
/// with everything it had, and so do a series' occurrences; only when the version sent in
/// `If-Match` is the one kept.
fn delete_event(
    state: &mut State,
    account: &'static str,
    target: &str,
    if_match: Option<String>,
) -> (&'static str, String) {
    let url = Url::parse(&format!("http://127.0.0.1{target}")).unwrap();
    let [_, _, calendars, calendar, events, id] = segments(&url).try_into().unwrap_or_default();
    if calendars != "calendars" || events != "events" {
        return not_found();
    }
    if let Some(refusal) = state.refuses_changes(account, &calendar) {
        return refusal;
    }
    let Some(mut event) = state.event_or_occurrence(account, &calendar, &id) else {
        return not_found();
    };
    if event["status"] == "cancelled" {
        return (
            "410 Gone",
            r#"{"error": {"code": 410, "message": "Resource has been deleted"}}"#.to_string(),
        );
    }
    if if_match.as_ref().is_some_and(|etag| *etag != event["etag"]) {
        return precondition_failed();
    }
    event["status"] = json!("cancelled");
    state.put_event(account, &calendar, event);
    // A series goes with its occurrences changed on their own.
    let occurrences: Vec<Value> = (state.events.iter())
        .filter(|(owner, c, e, _)| {
            *owner == account && *c == calendar && e["recurringEventId"] == id
        })
        .map(|(_, _, occurrence, _)| occurrence.clone())
        .collect();
    for mut occurrence in occurrences {
        occurrence["status"] = json!("cancelled");
        state.put_event(account, &calendar, occurrence);
    }
    ("204 No Content", String::new())
}

/// Whether the patch changes nothing of the event but the answer of the guest whose calendar the
/// copy is in (`self`).
fn answers_only(event: &Value, patch: &Value) -> bool {
    let without_own_answer = |attendees: &Value| -> Vec<Value> {
        (attendees.as_array().into_iter().flatten())
            .map(|attendee| {
                let mut attendee = attendee.clone();
                if attendee["self"] == true {
                    attendee.as_object_mut().unwrap().remove("responseStatus");
                }
                attendee
            })
            .collect()
    };
    patch.as_object().is_some_and(|fields| {
        fields.len() == 1
            && fields.contains_key("attendees")
            && without_own_answer(&patch["attendees"]) == without_own_answer(&event["attendees"])
    })
}

fn precondition_failed() -> (&'static str, String) {
    (
        "412 Precondition Failed",
        r#"{"error": {"code": 412, "message": "Precondition Failed"}}"#.to_string(),
    )
}

/// Marks the guest who is the calendar, as Google marks them `self` in the calendar's copy.
fn mark_self(event: &mut Value, calendar: &str) {
    let attendees = event.get_mut("attendees").and_then(Value::as_array_mut);
    for attendee in attendees.into_iter().flatten() {
        if attendee["email"] == calendar {
            attendee["self"] = json!(true);
        }
    }
}

/// JSON merge patch: objects merged field by field, null removing a field.
fn merge(kept: &mut Value, patch: &Value) {
    match patch.as_object() {
        Some(fields) if kept.is_object() => {
            for (name, value) in fields {
                if value.is_null() {
                    kept.as_object_mut().unwrap().remove(name);
                } else {
                    merge(&mut kept[name], value);
                }
            }
        }
        _ => *kept = patch.clone(),
    }
}

/// The Calendar API's `events.move`: the event leaves its calendar, deleted there, and arrives in
/// the destination, both calendars the account can change.
fn move_event(state: &mut State, account: &'static str, target: &str) -> (&'static str, String) {
    let url = Url::parse(&format!("http://127.0.0.1{target}")).unwrap();
    let [_, _, _, calendar, _, id, _] = segments(&url).try_into().unwrap_or_default();
    let destination = param(&url, "destination").unwrap_or_default();
    for calendar in [&calendar, &destination] {
        if let Some(refusal) = state.refuses_changes(account, calendar) {
            return refusal;
        }
    }
    let Some(event) = state.find_event(account, &calendar, &id) else {
        return not_found();
    };
    state.put_event(
        account,
        &calendar,
        json!({"kind": "calendar#event", "id": id, "status": "cancelled"}),
    );
    let event = state.put_event(account, &destination, event);
    ("200 OK", event.to_string())
}

/// The URL's path, segment by segment, decoded.
fn segments(url: &Url) -> Vec<String> {
    url.path_segments()
        .unwrap()
        .map(|segment| {
            percent_encoding::percent_decode_str(segment)
                .decode_utf8()
                .unwrap()
                .into_owned()
        })
        .collect()
}

/// An event deleted as a whole, which Google leaves out of a full list.
fn deleted(event: &Value) -> bool {
    event["status"] == "cancelled" && event.get("recurringEventId").is_none()
}

/// Google's `timeMin` and `timeMax`: an event ending after the one and starting before the other;
/// a series is sent whole, a cancelled occurrence by its original start.
fn in_window(event: &Value, from: DateTime<Utc>, until: DateTime<Utc>) -> bool {
    let at = |when: &Value| match when["dateTime"].as_str() {
        Some(at) => DateTime::parse_from_rfc3339(at)
            .unwrap()
            .with_timezone(&Utc),
        None => NaiveDate::parse_from_str(when["date"].as_str().unwrap(), "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
    };
    if event.get("recurrence").is_some() {
        return true;
    }
    if event["status"] == "cancelled" {
        let original = at(&event["originalStartTime"]);
        return original >= from && original < until;
    }
    at(&event["end"]) > from && at(&event["start"]) < until
}

fn respond(stream: &mut TcpStream, (status, json): (&str, String)) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{json}",
        json.len()
    );
}

/// The secret store, in memory.
#[derive(Default)]
pub struct FakeStore {
    pub not_running: bool,
    /// The store runs but refuses to save, as when it stays locked.
    pub locked: bool,
    pub saved: Mutex<Vec<(String, String)>>,
}

impl SecretStore for FakeStore {
    fn check(&self) -> Result<(), SecretStoreError> {
        if self.not_running {
            return Err(SecretStoreError::NotRunning);
        }
        Ok(())
    }

    fn save(&self, account: &Account, refresh_token: &str) -> Result<(), SecretStoreError> {
        self.check()?;
        if self.locked {
            return Err(SecretStoreError::Failed("the collection is locked".into()));
        }
        let mut saved = self.saved.lock().unwrap();
        saved.retain(|(id, _)| *id != account.id);
        saved.push((account.id.clone(), refresh_token.to_string()));
        Ok(())
    }

    fn load(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
        self.check()?;
        if self.locked {
            return Err(SecretStoreError::Failed("unlocking it was declined".into()));
        }
        let saved = self.saved.lock().unwrap();
        Ok(saved
            .iter()
            .find(|(id, _)| id == account_id)
            .map(|(_, token)| token.clone()))
    }

    fn take(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
        let refresh_token = self.load(account_id)?;
        self.saved
            .lock()
            .unwrap()
            .retain(|(id, _)| id != account_id);
        Ok(refresh_token)
    }
}
