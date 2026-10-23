//! Fakes for the tests: Google (its consent screen, as seen through the browser, its token
//! service, user info and the Calendar API) and the secret store. Nothing here leaves
//! `127.0.0.1`.

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use calendar_core::accounts::Account;
use calendar_core::platform::{Browser, SecretStore, SecretStoreError};
use chrono::{DateTime, NaiveDate, Utc};
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
        let mut state = self.state.lock().unwrap();
        state.changes += 1;
        let change = state.changes;
        state.events.retain(|(owner, c, e, _)| {
            !(*owner == account && c == calendar && e["id"] == event["id"])
        });
        state
            .events
            .push((account, calendar.to_string(), event, change));
    }

    /// The event is deleted elsewhere.
    pub fn delete_event(&self, account: &'static str, calendar: &str, id: &str) {
        self.add_event(
            account,
            calendar,
            json!({"kind": "calendar#event", "id": id, "status": "cancelled"}),
        );
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
    if head.starts_with("GET /calendar/v3/") {
        let answer = match bearer {
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
    let segments: Vec<String> = url
        .path_segments()
        .unwrap()
        .map(|segment| {
            percent_encoding::percent_decode_str(segment)
                .decode_utf8()
                .unwrap()
                .into_owned()
        })
        .collect();
    let (items, per_page): (Vec<&Value>, usize) = match segments.as_slice() {
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
