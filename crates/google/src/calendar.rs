//! Reading an account's calendars and events from the Google Calendar API (`MOD-02`) into the
//! app's own model: the calendar list with Google's colours, and each calendar's events within the
//! cache window, series as master plus exceptions (not expanded by Google), then only what changed
//! since (`syncToken`). People to invite are searched for in the account's contacts and its
//! organization's directory with the Google People API (`MOD-06`).

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use calendar_core::cache::{Cache, parse_access, parse_color};
use calendar_core::colors::EventColor;
use calendar_core::create::NewEvent;
use calendar_core::editor::Patch;
use calendar_core::guests::{Guest, Person, Reach, Response, same};
use calendar_core::model::{About, Attachment, Calendar, Kind, Show, Timing, Visibility};
use calendar_core::platform::{SecretStore, SecretStoreError};
use calendar_core::sync::{Fetched, Provider, Start, SyncError, SyncedEvent, Window};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use url::Url;

use crate::{Client, Endpoints, HTTP_TIMEOUT, agent, google_reason, random, refresh};

/// Google's largest pages: fewer requests for the first full synchronization.
const CALENDARS_PER_PAGE: &str = "250";
const EVENTS_PER_PAGE: &str = "2500";

/// How an error starts when Google answered, but refused what was asked.
const REFUSED: &str = "Google refused";

/// How an error starts when Google refused what was asked as invalid (400), as it refuses a
/// Google Meet call the account may not have.
const INVALID: &str = "Google refused it as invalid";

/// How many people each search of the People API asks for.
const PEOPLE_PER_SEARCH: &str = "10";

/// The People API's searches of the account's contacts and of the people it has e-mailed.
const CONTACT_SEARCHES: [&str; 2] = ["people:searchContacts", "otherContacts:search"];

/// How long an access token is used before a new one is asked for; Google's last an hour.
const ACCESS_FOR: Duration = Duration::from_secs(45 * 60);

/// The page where Google Calendar in the browser edits the event of the calendar, as the account
/// signed in there with `email` (`MOD-05-F29`): Google names an event there by its ID and its
/// calendar's, in base64url.
pub fn edit_page(calendar_id: &str, event_id: &str, email: &str) -> String {
    let eid = URL_SAFE_NO_PAD.encode(format!("{event_id} {calendar_id}"));
    let mut page = Url::parse("https://calendar.google.com/calendar/r/eventedit/").unwrap();
    page.path_segments_mut().unwrap().pop_if_empty().push(&eid);
    page.query_pairs_mut().append_pair("authuser", email);
    page.to_string()
}

/// Synchronizes one connected account: signs in with its refresh token from the secret store,
/// then fetches every calendar and its events within the window into the cache, or only what
/// changed in a calendar since its last synchronization.
pub fn sync_account(
    client_file: &Path,
    endpoints: &Endpoints,
    store: &dyn SecretStore,
    account_id: &str,
    window: Window,
    cache: &mut Cache,
) -> Result<(), SyncError> {
    let session = Session::sign_in(client_file, endpoints, store, account_id)?;
    calendar_core::sync::sync_account(&session, account_id, window, cache)
}

/// A list's items, and the synchronization state Google gave with them.
type Listed<T> = (Vec<T>, Option<String>);

/// One connected account at the Google Calendar API, signed in for as long as the app runs.
pub struct Session {
    client: Client,
    endpoints: Endpoints,
    account_id: String,
    /// Not `Debug`, so it never reaches a log (`NFR-07`).
    refresh_token: String,
    /// The access token in use and when it was issued; `None` once Google refused it.
    access: RefCell<Option<(String, Instant)>>,
    /// Whether the People API's contact searches were warmed up, as Google asks before the first.
    warmed: Cell<bool>,
}

impl Session {
    /// Signs in with the account's refresh token from the secret store.
    pub fn sign_in(
        client_file: &Path,
        endpoints: &Endpoints,
        store: &dyn SecretStore,
        account_id: &str,
    ) -> Result<Session, SyncError> {
        let client = Client::read(client_file).map_err(|_| {
            SyncError(format!(
                "the Google OAuth client file {} is missing or unreadable",
                client_file.display()
            ))
        })?;
        let refresh_token = store
            .load(account_id)
            .map_err(|error| match error {
                SecretStoreError::NotRunning => SyncError("no secret store is running".into()),
                SecretStoreError::Failed(reason) => {
                    SyncError(format!("the secret store is locked ({reason})"))
                }
            })?
            .ok_or_else(|| SyncError("the account needs sign-in".into()))?;
        let session = Session {
            client,
            endpoints: endpoints.clone(),
            account_id: account_id.to_string(),
            refresh_token,
            access: RefCell::new(None),
            warmed: Cell::new(false),
        };
        session.access_token()?;
        Ok(session)
    }

    /// The access token, a new one when the one in use is old or was refused.
    fn access_token(&self) -> Result<String, SyncError> {
        let mut access = self.access.borrow_mut();
        match &*access {
            Some((token, issued)) if issued.elapsed() < ACCESS_FOR => Ok(token.clone()),
            _ => {
                let token = refresh(&self.client, &self.endpoints, &self.refresh_token)
                    .map_err(|_| SyncError("Google did not accept the account's sign-in".into()))?;
                *access = Some((token.clone(), Instant::now()));
                Ok(token)
            }
        }
    }

    fn url(&self, segments: &[&str]) -> Url {
        let mut url = Url::parse(&self.endpoints.calendar).expect("a valid Calendar API URL");
        url.path_segments_mut()
            .expect("a base URL")
            .extend(segments);
        url
    }

    /// The People API's search, for `query`, of names and e-mail addresses.
    fn people_url(&self, search: &str, query: &str) -> Url {
        let mut url = Url::parse(&self.endpoints.people).expect("a valid People API URL");
        url.path_segments_mut().expect("a base URL").push(search);
        url.query_pairs_mut()
            .append_pair("query", query)
            .append_pair("readMask", "names,emailAddresses")
            .append_pair("pageSize", PEOPLE_PER_SEARCH);
        url
    }

    /// Every item of a list, following Google's pages, and the state Google gives on the last
    /// one; `None` when Google answers that the state asked with is gone.
    fn all<T: DeserializeOwned>(&self, url: Url) -> Result<Option<Listed<T>>, SyncError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Page<T> {
            #[serde(default = "Vec::new")]
            items: Vec<T>,
            next_page_token: Option<String>,
            next_sync_token: Option<String>,
        }
        let mut items = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let mut page_url = url.clone();
            if let Some(token) = &page_token {
                page_url.query_pairs_mut().append_pair("pageToken", token);
            }
            let Some(page) = self.get::<Page<T>>(page_url)? else {
                return Ok(None);
            };
            items.extend(page.items);
            match page.next_page_token {
                Some(token) => page_token = Some(token),
                None => return Ok(Some((items, page.next_sync_token))),
            }
        }
    }

    /// Google's answer; `None` for 410 Gone, Google's word for a synchronization state it no
    /// longer accepts. An access token Google refuses (it expired, e.g. while the computer slept)
    /// is renewed once. Google's rate limits, 429 and 403 for `rateLimitExceeded` or
    /// `userRateLimitExceeded`, are told apart from other refusals (`MOD-02-F16`).
    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<Option<T>, SyncError> {
        self.send(url, Request::Get)
    }

    /// Google's answer to a change sent to it, as `get` handles answers.
    fn post<T: DeserializeOwned>(&self, url: Url, change: Option<&Value>) -> Result<T, SyncError> {
        self.send(url, Request::Post(change))?
            .ok_or_else(|| SyncError("Google refused (gone)".into()))
    }

    /// Google's answer to a patch of what it keeps in the version `etag`, or in any version
    /// without one, as `get` handles answers; `SyncError::changed_elsewhere` when that is no
    /// longer the version kept.
    fn patch<T: DeserializeOwned>(
        &self,
        url: Url,
        change: &Value,
        etag: Option<&str>,
    ) -> Result<T, SyncError> {
        self.send(url, Request::Patch(change, etag))?
            .ok_or_else(|| SyncError("Google refused (gone)".into()))
    }

    /// Asks Google, as `get` says.
    fn send<T: DeserializeOwned>(
        &self,
        url: Url,
        request: Request,
    ) -> Result<Option<T>, SyncError> {
        let (mut status, mut body) = self.call(&url, request)?;
        if status == 401 {
            self.access.replace(None);
            (status, body) = self.call(&url, request)?;
        }
        if status == 410 {
            return Ok(None);
        }
        if status == 412 {
            return Err(SyncError::changed_elsewhere());
        }
        if status == 429 || (status == 403 && rate_limited(&body)) {
            return Err(SyncError::rate_limited());
        }
        if !status.is_success() {
            let reason = google_reason(&body).unwrap_or_else(|| status.to_string());
            let refused = if status == 400 { INVALID } else { REFUSED };
            return Err(SyncError(format!("{refused} ({reason})")));
        }
        // A deletion is answered without a body.
        let body = if body.is_empty() { "null" } else { &body };
        serde_json::from_str(body)
            .map(Some)
            .map_err(|_| SyncError("Google's answer could not be read".into()))
    }

    fn call(
        &self,
        url: &Url,
        request: Request,
    ) -> Result<(ureq::http::StatusCode, String), SyncError> {
        let bearer = format!("Bearer {}", self.access_token()?);
        let agent = agent(HTTP_TIMEOUT);
        let response = match request {
            Request::Get => agent
                .get(url.as_str())
                .header("Authorization", bearer)
                .call(),
            Request::Post(None) => agent
                .post(url.as_str())
                .header("Authorization", bearer)
                .send_empty(),
            Request::Post(Some(change)) => agent
                .post(url.as_str())
                .header("Authorization", bearer)
                .header("Content-Type", "application/json")
                .send(change.to_string()),
            Request::Patch(change, etag) => {
                let request = agent
                    .patch(url.as_str())
                    .header("Authorization", bearer)
                    .header("Content-Type", "application/json");
                match etag {
                    Some(etag) => request.header("If-Match", etag),
                    None => request,
                }
                .send(change.to_string())
            }
            Request::Delete(etag) => {
                let request = agent.delete(url.as_str()).header("Authorization", bearer);
                match etag {
                    Some(etag) => request.header("If-Match", etag),
                    None => request,
                }
                .call()
            }
        };
        let mut response = response
            .map_err(|error| SyncError(format!("Google could not be reached ({error})")))?;
        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|error| SyncError(format!("Google's answer could not be read ({error})")))?;
        Ok((response.status(), body))
    }

    /// A calendar's events from Google, as the app keeps them.
    fn fetch(&self, url: Url, calendar: &Calendar) -> Result<Option<Fetched>, SyncError> {
        let Some((events, sync_token)) = self.all::<GoogleEvent>(url)? else {
            return Ok(None);
        };
        let mut fetched = Fetched {
            sync_token,
            ..Fetched::default()
        };
        for event in events {
            match event.into_synced(calendar.zone) {
                Ok(event) => fetched.events.push(event),
                Err(Some(deleted)) => fetched.removed.push(deleted),
                Err(None) => {}
            }
        }
        Ok(Some(fetched))
    }
}

/// What is asked of Google.
#[derive(Clone, Copy)]
enum Request<'a> {
    Get,
    /// With the change to send, if any.
    Post(Option<&'a Value>),
    /// The change, made to the version (ETag) if one is given.
    Patch(&'a Value, Option<&'a str>),
    /// Deleting the version (ETag) if one is given.
    Delete(Option<&'a str>),
}

/// The version a change is sent against: none for an occurrence of a series not changed yet,
/// which has no version of its own.
fn version(etag: &str) -> Option<&str> {
    (!etag.is_empty()).then_some(etag)
}

/// The fields of a new event, as Google takes them.
fn fields(event: &NewEvent) -> serde_json::Map<String, Value> {
    let (start, end) = times(&event.timing, false);
    let mut fields = json!({
        "summary": event.title,
        "description": event.about.description,
        "location": event.about.location,
        "transparency": transparency(event.about.show),
        "visibility": visibility(event.about.visibility),
        "start": start,
        "end": end,
    });
    if !event.recurrence.is_empty() {
        fields["recurrence"] = json!(event.recurrence);
    }
    if let Some(color) = event.about.color {
        fields["colorId"] = json!(color_id(color));
    }
    if !event.about.guests.is_empty() {
        fields["attendees"] = attendees(&event.about.guests, &[]);
    }
    // Out of office and focus time with what Google Calendar sets them up with: new invitations
    // that clash are declined, and chat is set to do not disturb during focus time; both busy.
    match event.about.kind {
        Kind::OutOfOffice => {
            fields["eventType"] = json!("outOfOffice");
            fields["outOfOfficeProperties"] = json!({
                "autoDeclineMode": "declineOnlyNewConflictingInvitations",
                "declineMessage": "Declined because I am out of office",
            });
            fields["transparency"] = json!("opaque");
        }
        Kind::FocusTime => {
            fields["eventType"] = json!("focusTime");
            fields["focusTimeProperties"] = json!({
                "autoDeclineMode": "declineOnlyNewConflictingInvitations",
                "declineMessage": "Declined because I am in focus time",
                "chatStatus": "doNotDisturb",
            });
            fields["transparency"] = json!("opaque");
        }
        // Not made by the app (`DEC-34`).
        Kind::Ordinary | Kind::WorkingLocation | Kind::Birthday => {}
    }
    let Value::Object(fields) = fields else {
        unreachable!("an object");
    };
    fields
}

/// Google's `attendees` of an event for the guests. A guest Google keeps an entry for in `kept`
/// keeps it as it is, with their answer, comment, additional guests and room mark, but for being
/// optional and their name: changing the guests, or copying them from an occurrence to its
/// series, changes nobody's answer. Another guest is new to the event, and sent unanswered.
/// Google gives the account's own entry its `self` mark.
fn attendees(guests: &[Guest], kept: &[Value]) -> Value {
    let attendees: Vec<Value> = (guests.iter())
        .map(|guest| {
            let entry = kept.iter().find(|attendee| {
                (attendee["email"].as_str()).is_some_and(|email| same(email, &guest.email))
            });
            if let Some(Value::Object(entry)) = entry {
                let mut attendee = entry.clone();
                attendee.remove("optional");
                if guest.optional {
                    attendee.insert("optional".into(), json!(true));
                }
                if !guest.name.is_empty() {
                    attendee.insert("displayName".into(), json!(guest.name));
                }
                return Value::Object(attendee);
            }
            let mut attendee = json!({"email": guest.email, "responseStatus": "needsAction"});
            if !guest.name.is_empty() {
                attendee["displayName"] = json!(guest.name);
            }
            if guest.optional {
                attendee["optional"] = json!(true);
            }
            attendee
        })
        .collect();
    Value::Array(attendees)
}

/// The entries of Google's `attendees`; none when it has none.
fn kept_list(attendees: &Value) -> &[Value] {
    attendees.as_array().map_or(&[], Vec::as_slice)
}

/// Google's request to make a Google Meet call for an event, each with an ID of its own.
fn meet_request() -> Value {
    json!({
        "createRequest": {
            "requestId": random(),
            "conferenceSolutionKey": {"type": "hangoutsMeet"},
        }
    })
}

/// Sends a new or changed event with its Google Meet call asked for (`send(true)`); when Google
/// refuses it as invalid, as for an account that cannot have Meet, sends it again without
/// (`send(false)`), so the event is saved without a call (MOD-06 *Constraints and edge cases*).
fn with_meet<T>(send: impl Fn(bool) -> Result<T, SyncError>) -> Result<T, SyncError> {
    match send(true) {
        Err(error) if error.0.starts_with(INVALID) => send(false),
        sent => sent,
    }
}

/// The link of an event's Google Meet call, from its `conferenceData`: the video entry point of a
/// `hangoutsMeet` conference; `None` for another conference, or one Google has not made.
fn meet_link(conference: &Value) -> Option<String> {
    if conference["conferenceSolution"]["key"]["type"] != "hangoutsMeet" {
        return None;
    }
    let entry_points = conference["entryPoints"].as_array()?;
    (entry_points.iter())
        .find(|entry| entry["entryPointType"] == "video")
        .and_then(|entry| entry["uri"].as_str())
        // Only web pages are handed to the desktop's URL handler.
        .filter(|uri| uri.starts_with("https://"))
        .map(str::to_string)
}

/// A new event's ID, as Google answers its creation.
#[derive(Deserialize)]
struct Created {
    id: String,
}

/// An event's version, as Google answers a change.
#[derive(Deserialize)]
struct Version {
    etag: String,
}

/// Google's `start` and `end` of an event: dates for an all-day one, otherwise times on the
/// event's own clock with its zone, as Google Calendar sends them. `clear` sets the fields of the
/// other kind to null, as a patch must to change the kind.
fn times(timing: &Timing, clear: bool) -> (Value, Value) {
    let (mut start, mut end) = match *timing {
        Timing::Timed { start, end, zone } => {
            let when = |at: DateTime<Utc>| {
                json!({
                    "dateTime": at.with_timezone(&zone).to_rfc3339_opts(SecondsFormat::Secs, true),
                    "timeZone": zone.name(),
                })
            };
            (when(start), when(end))
        }
        Timing::AllDay {
            start,
            end_exclusive,
        } => (
            json!({"date": start.to_string()}),
            json!({"date": end_exclusive.to_string()}),
        ),
    };
    if clear {
        for when in [&mut start, &mut end] {
            for field in ["date", "dateTime", "timeZone"] {
                if when.get(field).is_none() {
                    when[field] = Value::Null;
                }
            }
        }
    }
    (start, end)
}

/// Whether Google e-mails the event's guests about the change, as the user answered
/// (`MOD-06-F12`).
fn send_updates(url: &mut Url, notify: bool) {
    let send = if notify { "all" } else { "none" };
    url.query_pairs_mut().append_pair("sendUpdates", send);
}

/// Google's `responseStatus` of a guest's answer.
fn response_status(response: Response) -> &'static str {
    match response {
        Response::Accepted => "accepted",
        Response::Declined => "declined",
        Response::Maybe => "tentative",
        Response::Awaiting => "needsAction",
    }
}

/// Google's `transparency`.
fn transparency(show: Show) -> &'static str {
    match show {
        Show::Busy => "opaque",
        Show::Free => "transparent",
    }
}

/// Google's `visibility`.
fn visibility(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Default => "default",
        Visibility::Public => "public",
        Visibility::Private => "private",
    }
}

/// Google's event colours by their `colorId`, as Google Calendar shows them (`DEC-118`).
const COLOR_IDS: [(&str, EventColor); 11] = [
    ("1", EventColor::Lavender),
    ("2", EventColor::Sage),
    ("3", EventColor::Grape),
    ("4", EventColor::Flamingo),
    ("5", EventColor::Banana),
    ("6", EventColor::Tangerine),
    ("7", EventColor::Peacock),
    ("8", EventColor::Graphite),
    ("9", EventColor::Blueberry),
    ("10", EventColor::Basil),
    ("11", EventColor::Tomato),
];

/// Google's `colorId` of an event colour.
fn color_id(color: EventColor) -> &'static str {
    let (id, _) = COLOR_IDS
        .iter()
        .find(|(_, c)| *c == color)
        .expect("every event colour has an ID");
    id
}

/// The event colour of Google's `colorId`; `None` for one outside the palette.
fn event_color(id: &str) -> Option<EventColor> {
    COLOR_IDS.iter().find(|(i, _)| *i == id).map(|(_, c)| *c)
}

/// Whether a 403 from Google's APIs is a rate limit rather than a refusal.
fn rate_limited(body: &str) -> bool {
    let value: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    value["error"]["errors"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|error| {
            matches!(
                error["reason"].as_str(),
                Some("rateLimitExceeded" | "userRateLimitExceeded")
            )
        })
}

impl Provider for Session {
    fn calendars(&self) -> Result<Vec<Calendar>, SyncError> {
        // Calendars hidden from the list in Google stay out, as in Google Calendar (`DEC-86`).
        let mut url = self.url(&["users", "me", "calendarList"]);
        url.query_pairs_mut()
            .append_pair("maxResults", CALENDARS_PER_PAGE);
        let (entries, _) = self
            .all::<CalendarEntry>(url)?
            .ok_or_else(|| SyncError("Google refused (gone)".into()))?;
        Ok(entries
            .into_iter()
            .filter_map(|entry| entry.into_calendar(&self.account_id))
            .collect())
    }

    fn events(&self, calendar: &Calendar, window: Window) -> Result<Fetched, SyncError> {
        let instant = |at: DateTime<Utc>| at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let mut url = self.url(&["calendars", &calendar.id, "events"]);
        // Not `singleEvents`: series come as master plus exceptions, cancelled occurrences
        // included.
        url.query_pairs_mut()
            .append_pair("timeMin", &instant(window.start()))
            .append_pair("timeMax", &instant(window.end()))
            .append_pair("maxResults", EVENTS_PER_PAGE);
        self.fetch(url, calendar)?
            .ok_or_else(|| SyncError("Google refused (gone)".into()))
    }

    fn changes(&self, calendar: &Calendar, sync_token: &str) -> Result<Option<Fetched>, SyncError> {
        // Google takes no time range with a `syncToken`: the window is applied by the app.
        let mut url = self.url(&["calendars", &calendar.id, "events"]);
        url.query_pairs_mut()
            .append_pair("syncToken", sync_token)
            .append_pair("maxResults", EVENTS_PER_PAGE);
        self.fetch(url, calendar)
    }

    /// Google e-mails the invitations to a new event's guests (`MOD-06-F08`), and makes its
    /// Google Meet call where the account allows it (`MOD-06-F16`).
    fn create(&self, calendar: &Calendar, event: &NewEvent) -> Result<String, SyncError> {
        let send = |meet: bool| {
            let mut url = self.url(&["calendars", &calendar.id, "events"]);
            if !event.about.guests.is_empty() {
                url.query_pairs_mut().append_pair("sendUpdates", "all");
            }
            let mut change = fields(event);
            if meet {
                url.query_pairs_mut()
                    .append_pair("conferenceDataVersion", "1");
                change.insert("conferenceData".into(), meet_request());
            }
            Ok(self.post::<Created>(url, Some(&Value::Object(change)))?.id)
        };
        if event.meet {
            with_meet(send)
        } else {
            send(false)
        }
    }

    /// The event as Google keeps it, but what Google gives an event of its own and what made it
    /// a series or an occurrence of one; with its conference and attachments, which Google takes
    /// only when asked to.
    fn copy(
        &self,
        calendar: &Calendar,
        id: &str,
        new: &NewEvent,
        notify: bool,
    ) -> Result<String, SyncError> {
        let url = self.url(&["calendars", &calendar.id, "events", id]);
        let Some(Value::Object(mut event)) = self.get::<Value>(url)? else {
            return Err(SyncError("Google refused (gone)".into()));
        };
        for own in [
            "id",
            "iCalUID",
            "etag",
            "htmlLink",
            "created",
            "updated",
            "sequence",
            "recurrence",
            "recurringEventId",
            "originalStartTime",
        ] {
            event.remove(own);
        }
        // The new series has the colour and the guests `new` gives it, none included; the
        // guests keep what Google keeps of them.
        event.remove("colorId");
        let kept = event.remove("attendees").unwrap_or_default();
        event.extend(fields(new));
        if !new.about.guests.is_empty() {
            event.insert(
                "attendees".into(),
                attendees(&new.about.guests, kept_list(&kept)),
            );
        }
        // The series' Google Meet call stays the copy's, unless it is switched off; one switched
        // on is made (`MOD-06-F16`).
        let had = event.get("conferenceData").and_then(meet_link).is_some();
        if !new.meet && had {
            event.remove("conferenceData");
        }
        let send = |meet: bool| {
            let mut event = event.clone();
            if meet {
                event.insert("conferenceData".into(), meet_request());
            }
            let mut url = self.url(&["calendars", &calendar.id, "events"]);
            url.query_pairs_mut()
                .append_pair("conferenceDataVersion", "1")
                .append_pair("supportsAttachments", "true");
            send_updates(&mut url, notify);
            Ok(self.post::<Created>(url, Some(&Value::Object(event)))?.id)
        };
        if new.meet && !had {
            with_meet(send)
        } else {
            send(false)
        }
    }

    fn update(
        &self,
        calendar: &Calendar,
        id: &str,
        etag: &str,
        patch: &Patch,
        notify: bool,
    ) -> Result<String, SyncError> {
        let mut change = serde_json::Map::new();
        let mut set = |field: &str, value: Option<Value>| {
            if let Some(value) = value {
                change.insert(field.to_string(), value);
            }
        };
        set("summary", patch.title.clone().map(Value::from));
        set("description", patch.description.clone().map(Value::from));
        set("location", patch.location.clone().map(Value::from));
        set("transparency", patch.show.map(|s| transparency(s).into()));
        set("visibility", patch.visibility.map(|v| visibility(v).into()));
        if let Some(timing) = &patch.timing {
            let (start, end) = times(timing, true);
            set("start", Some(start));
            set("end", Some(end));
        }
        set("recurrence", patch.recurrence.clone().map(Value::from));
        // Null takes the event's own colour away.
        set(
            "colorId",
            patch.color.map(|color| json!(color.map(color_id))),
        );
        let mut url = self.url(&["calendars", &calendar.id, "events", id]);
        // What Google keeps of each guest is read first, as the list is replaced whole. The
        // user's own answer goes in their own entry (`self`) alone, all a guest may change in a
        // copy only its organizer may change (`MOD-06-F15`); a list Google left guests out of
        // stays so. The organizer's own answer e-mails nobody (`DEC-128`).
        let mut notify = notify;
        if patch.guests.is_some() || patch.response.is_some() {
            let event = self.get::<Value>(url.clone())?.unwrap_or_default();
            let kept = kept_list(&event["attendees"]);
            let mut list = match &patch.guests {
                Some(guests) => attendees(guests, kept),
                None => Value::Array(kept.to_vec()),
            };
            if let Some(response) = patch.response {
                for attendee in list.as_array_mut().into_iter().flatten() {
                    if attendee["self"] == true {
                        attendee["responseStatus"] = json!(response_status(response));
                    }
                }
                if event["attendeesOmitted"] == true {
                    set("attendeesOmitted", Some(json!(true)));
                }
                notify &= event["organizer"]["self"] != true;
            }
            set("attendees", Some(list));
        }
        send_updates(&mut url, notify);
        // Google Meet made or taken away (`MOD-06-F16`).
        if patch.meet.is_some() {
            url.query_pairs_mut()
                .append_pair("conferenceDataVersion", "1");
        }
        let send = |meet: bool| {
            let mut change = change.clone();
            match patch.meet {
                Some(true) if meet => {
                    change.insert("conferenceData".into(), meet_request());
                }
                Some(false) => {
                    change.insert("conferenceData".into(), Value::Null);
                }
                _ => {}
            }
            Ok(self
                .patch::<Version>(url.clone(), &Value::Object(change), version(etag))?
                .etag)
        };
        if patch.meet == Some(true) {
            with_meet(send)
        } else {
            send(false)
        }
    }

    fn move_to(
        &self,
        calendar: &Calendar,
        id: &str,
        destination: &Calendar,
        notify: bool,
    ) -> Result<String, SyncError> {
        let mut url = self.url(&["calendars", &calendar.id, "events", id, "move"]);
        url.query_pairs_mut()
            .append_pair("destination", &destination.id);
        send_updates(&mut url, notify);
        Ok(self.post::<Version>(url, None)?.etag)
    }

    /// Google answers a deletion of an event already deleted with 410 Gone: it is deleted.
    fn delete(
        &self,
        calendar: &Calendar,
        id: &str,
        etag: &str,
        notify: bool,
    ) -> Result<(), SyncError> {
        let mut url = self.url(&["calendars", &calendar.id, "events", id]);
        send_updates(&mut url, notify);
        self.send::<()>(url, Request::Delete(version(etag)))
            .map(|_| ())
    }

    /// Google keeps a deleted event, cancelled, for a while: confirming it again brings it back
    /// with everything it had.
    fn restore(&self, calendar: &Calendar, id: &str, notify: bool) -> Result<String, SyncError> {
        let mut url = self.url(&["calendars", &calendar.id, "events", id]);
        send_updates(&mut url, notify);
        let change = json!({"status": "confirmed"});
        Ok(self.patch::<Version>(url, &change, None)?.etag)
    }

    /// The account's contacts, the people it has e-mailed and its organization's directory,
    /// each searched as `reach` allows; a search Google does not answer adds nobody.
    fn people(&self, query: &str, reach: Reach) -> Vec<Person> {
        let mut found = Vec::new();
        if reach.contacts {
            // Google searches a cache it brings up to date on an empty search; tried again at
            // the next search until both get through.
            if !self.warmed.get() {
                let warmed = CONTACT_SEARCHES
                    .map(|search| self.get::<Value>(self.people_url(search, "")).is_ok());
                self.warmed.set(warmed.iter().all(|ok| *ok));
            }
            for search in CONTACT_SEARCHES {
                if let Ok(Some(answer)) = self.get::<Results>(self.people_url(search, query)) {
                    found.extend(answer.results.into_iter().map(|result| result.person));
                }
            }
        }
        if reach.directory {
            let mut url = self.people_url("people:searchDirectoryPeople", query);
            url.query_pairs_mut()
                .append_pair("sources", "DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE");
            if let Ok(Some(answer)) = self.get::<Directory>(url) {
                found.extend(answer.people);
            }
        }
        found
            .into_iter()
            .flat_map(GooglePerson::into_people)
            .collect()
    }

    fn default_length(&self) -> Result<u32, SyncError> {
        #[derive(Deserialize)]
        struct Setting {
            value: String,
        }
        let url = self.url(&["users", "me", "settings", "defaultEventLength"]);
        let setting: Setting = self
            .get(url)?
            .ok_or_else(|| SyncError("Google refused (gone)".into()))?;
        setting
            .value
            .parse()
            .map_err(|_| SyncError("Google's answer could not be read".into()))
    }
}

/// The People API's answer to a search of contacts.
#[derive(Deserialize)]
struct Results {
    #[serde(default)]
    results: Vec<Found>,
}

#[derive(Deserialize)]
struct Found {
    person: GooglePerson,
}

/// The People API's answer to a search of the directory.
#[derive(Deserialize)]
struct Directory {
    #[serde(default)]
    people: Vec<GooglePerson>,
}

/// A People API `person`, with only what the app uses.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GooglePerson {
    #[serde(default)]
    names: Vec<PersonName>,
    #[serde(default)]
    email_addresses: Vec<PersonAddress>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersonName {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct PersonAddress {
    value: Option<String>,
}

impl GooglePerson {
    /// One person to invite for each of their addresses, under their first name.
    fn into_people(self) -> Vec<Person> {
        let name = (self.names.into_iter())
            .find_map(|name| name.display_name)
            .unwrap_or_default();
        (self.email_addresses.into_iter())
            .filter_map(|address| address.value.filter(|value| !value.is_empty()))
            .map(|email| Person {
                name: name.clone(),
                email,
            })
            .collect()
    }
}

/// A `calendarList` entry.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEntry {
    id: String,
    summary: Option<String>,
    /// The name the user gave a calendar shared with them; Google shows it instead.
    summary_override: Option<String>,
    background_color: Option<String>,
    access_role: String,
    time_zone: Option<String>,
}

impl CalendarEntry {
    fn into_calendar(self, account_id: &str) -> Option<Calendar> {
        Some(Calendar {
            name: self.summary_override.or(self.summary).unwrap_or_default(),
            id: self.id,
            account_id: account_id.to_string(),
            color: parse_color(self.background_color.as_deref()?)?,
            access: parse_access(&self.access_role)?,
            zone: self
                .time_zone
                .and_then(|zone| zone.parse().ok())
                .unwrap_or(Tz::UTC),
        })
    }
}

/// An `events` resource, with only what the app uses.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEvent {
    id: String,
    status: Option<String>,
    summary: Option<String>,
    start: Option<When>,
    end: Option<When>,
    #[serde(default)]
    recurrence: Vec<String>,
    recurring_event_id: Option<String>,
    original_start_time: Option<When>,
    description: Option<String>,
    location: Option<String>,
    transparency: Option<String>,
    visibility: Option<String>,
    #[serde(default)]
    etag: String,
    organizer: Option<Organizer>,
    #[serde(default)]
    guests_can_modify: bool,
    event_type: Option<String>,
    color_id: Option<String>,
    #[serde(default)]
    attachments: Vec<GoogleAttachment>,
    #[serde(default)]
    attendees: Vec<GoogleAttendee>,
    #[serde(default)]
    conference_data: Value,
}

/// One of Google's `attendees` of an event.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleAttendee {
    email: Option<String>,
    display_name: Option<String>,
    #[serde(default)]
    optional: bool,
    response_status: Option<String>,
    #[serde(rename = "self", default)]
    is_self: bool,
}

impl GoogleAttendee {
    /// The guest as the app keeps them; `None` without an address, which cannot be invited again.
    fn into_guest(self) -> Option<Guest> {
        Some(Guest {
            email: self.email.filter(|email| !email.is_empty())?,
            name: self.display_name.unwrap_or_default(),
            optional: self.optional,
            response: match self.response_status.as_deref() {
                Some("accepted") => Response::Accepted,
                Some("declined") => Response::Declined,
                Some("tentative") => Response::Maybe,
                _ => Response::Awaiting,
            },
            own: self.is_self,
        })
    }
}

/// One of Google's `attachments` of an event: a file, mostly on Google Drive.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleAttachment {
    file_url: Option<String>,
    title: Option<String>,
}

impl GoogleAttachment {
    /// The attachment as the app lists it; `None` without a web page to open, as only web pages
    /// are handed to the desktop's URL handler.
    fn into_attachment(self) -> Option<Attachment> {
        let url = self.file_url.filter(|url| url.starts_with("https://"))?;
        Some(Attachment {
            title: self
                .title
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| url.clone()),
            url,
        })
    }
}

/// Google's `organizer` of an event; `self` when it is the calendar the copy is on.
#[derive(Deserialize)]
struct Organizer {
    #[serde(rename = "self", default)]
    is_self: bool,
}

/// Google's `start`, `end` and `originalStartTime`: a date for all-day events, otherwise an
/// RFC 3339 time with the zone the event was made in.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct When {
    date: Option<String>,
    date_time: Option<String>,
    time_zone: Option<String>,
}

impl When {
    fn start(&self) -> Option<Start> {
        match (&self.date_time, &self.date) {
            (Some(at), _) => Some(Start::At(
                DateTime::parse_from_rfc3339(at).ok()?.with_timezone(&Utc),
            )),
            (None, Some(date)) => Some(Start::On(date.parse::<NaiveDate>().ok()?)),
            (None, None) => None,
        }
    }
}

impl GoogleEvent {
    /// The event as the app keeps it. `Err` for what the app cannot show: with the event's ID
    /// when it was deleted (cancelled, not an occurrence of a series), otherwise `None` (no
    /// readable times).
    fn into_synced(self, calendar_zone: Tz) -> Result<SyncedEvent, Option<String>> {
        let instance_of = match (self.recurring_event_id, &self.original_start_time) {
            (Some(series), Some(original)) => Some((series, original.start().ok_or(None)?)),
            _ => None,
        };
        let cancelled = self.status.as_deref() == Some("cancelled");
        let timing = if cancelled {
            if instance_of.is_none() {
                return Err(Some(self.id));
            }
            None
        } else {
            let (start, end) = (self.start.ok_or(None)?, self.end.ok_or(None)?);
            Some(
                match (start.start().ok_or(None)?, end.start().ok_or(None)?) {
                    (Start::At(from), Start::At(until)) => Timing::Timed {
                        start: from,
                        end: until,
                        zone: start
                            .time_zone
                            .and_then(|zone| zone.parse().ok())
                            .unwrap_or(calendar_zone),
                    },
                    (Start::On(from), Start::On(until)) => Timing::AllDay {
                        start: from,
                        end_exclusive: until,
                    },
                    _ => return Err(None),
                },
            )
        };
        Ok(SyncedEvent {
            id: self.id,
            title: self.summary.unwrap_or_default(),
            timing,
            recurrence: self.recurrence,
            instance_of,
            about: About {
                description: self.description.unwrap_or_default(),
                location: self.location.unwrap_or_default(),
                show: match self.transparency.as_deref() {
                    Some("transparent") => Show::Free,
                    _ => Show::Busy,
                },
                // Google keeps "confidential" for compatibility; it means private.
                visibility: match self.visibility.as_deref() {
                    Some("public") => Visibility::Public,
                    Some("private" | "confidential") => Visibility::Private,
                    _ => Visibility::Default,
                },
                // Other types (`default`, `fromGmail`) are ordinary events to the app.
                kind: match self.event_type.as_deref() {
                    Some("outOfOffice") => Kind::OutOfOffice,
                    Some("focusTime") => Kind::FocusTime,
                    Some("workingLocation") => Kind::WorkingLocation,
                    Some("birthday") => Kind::Birthday,
                    _ => Kind::Ordinary,
                },
                color: self.color_id.as_deref().and_then(event_color),
                attachments: (self.attachments.into_iter())
                    .filter_map(GoogleAttachment::into_attachment)
                    .collect(),
                guests: (self.attendees.into_iter())
                    .filter_map(GoogleAttendee::into_guest)
                    .collect(),
                meet: meet_link(&self.conference_data),
            },
            etag: self.etag,
            // A guest's copy, with the organizer's event elsewhere, as Google tells them apart.
            organizer_only: self.organizer.is_some_and(|organizer| !organizer.is_self)
                && !self.guests_can_modify,
        })
    }
}

#[cfg(test)]
#[path = "calendar_tests.rs"]
mod tests;
