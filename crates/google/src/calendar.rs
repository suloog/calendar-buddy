//! Reading an account's calendars and events from the Google Calendar API (`MOD-02`) into the
//! app's own model: the calendar list with Google's colours, and each calendar's events within the
//! cache window, series as master plus exceptions (not expanded by Google), then only what changed
//! since (`syncToken`).

use std::cell::RefCell;
use std::path::Path;
use std::time::{Duration, Instant};

use calendar_core::cache::{Cache, parse_access, parse_color};
use calendar_core::create::NewEvent;
use calendar_core::editor::Patch;
use calendar_core::model::{About, Calendar, Show, Timing, Visibility};
use calendar_core::platform::{SecretStore, SecretStoreError};
use calendar_core::sync::{Fetched, Provider, Start, SyncError, SyncedEvent, Window};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use url::Url;

use crate::{Client, Endpoints, HTTP_TIMEOUT, agent, google_reason, refresh};

/// Google's largest pages: fewer requests for the first full synchronization.
const CALENDARS_PER_PAGE: &str = "250";
const EVENTS_PER_PAGE: &str = "2500";

/// How long an access token is used before a new one is asked for; Google's last an hour.
const ACCESS_FOR: Duration = Duration::from_secs(45 * 60);

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

    /// Google's answer to a patch of what it keeps in the version `etag`, as `get` handles
    /// answers; `SyncError::changed_elsewhere` when that is no longer the version kept.
    fn patch<T: DeserializeOwned>(
        &self,
        url: Url,
        change: &Value,
        etag: &str,
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
            return Err(SyncError(format!("Google refused ({reason})")));
        }
        serde_json::from_str(&body)
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
            Request::Patch(change, etag) => agent
                .patch(url.as_str())
                .header("Authorization", bearer)
                .header("Content-Type", "application/json")
                .header("If-Match", etag)
                .send(change.to_string()),
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
    /// The change, made to the version (ETag).
    Patch(&'a Value, &'a str),
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

    fn create(&self, calendar: &Calendar, event: &NewEvent) -> Result<String, SyncError> {
        #[derive(Deserialize)]
        struct Created {
            id: String,
        }
        let (start, end) = times(&event.timing, false);
        let change = json!({
            "summary": event.title,
            "description": event.about.description,
            "location": event.about.location,
            "transparency": transparency(event.about.show),
            "visibility": visibility(event.about.visibility),
            "start": start,
            "end": end,
        });
        let url = self.url(&["calendars", &calendar.id, "events"]);
        Ok(self.post::<Created>(url, Some(&change))?.id)
    }

    fn update(
        &self,
        calendar: &Calendar,
        id: &str,
        etag: &str,
        patch: &Patch,
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
        let url = self.url(&["calendars", &calendar.id, "events", id]);
        Ok(self
            .patch::<Version>(url, &Value::Object(change), etag)?
            .etag)
    }

    fn move_to(
        &self,
        calendar: &Calendar,
        id: &str,
        destination: &Calendar,
    ) -> Result<String, SyncError> {
        let mut url = self.url(&["calendars", &calendar.id, "events", id, "move"]);
        url.query_pairs_mut()
            .append_pair("destination", &destination.id);
        Ok(self.post::<Version>(url, None)?.etag)
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
            },
            etag: self.etag,
        })
    }
}

#[cfg(test)]
#[path = "calendar_tests.rs"]
mod tests;
