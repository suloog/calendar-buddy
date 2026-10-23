//! Reading an account's calendars and events from the Google Calendar API (`MOD-02`) into the
//! app's own model: the calendar list with Google's colours, and each calendar's events within the
//! cache window, series as master plus exceptions (not expanded by Google).

use std::path::Path;

use calendar_core::cache::{Cache, parse_access, parse_color};
use calendar_core::model::{Calendar, Timing};
use calendar_core::platform::{SecretStore, SecretStoreError};
use calendar_core::sync::{Provider, Start, SyncError, SyncedEvent, Window, full_sync};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::{Client, Endpoints, HTTP_TIMEOUT, agent, google_reason, refresh};

/// Google's largest pages: fewer requests for the first full synchronization.
const CALENDARS_PER_PAGE: &str = "250";
const EVENTS_PER_PAGE: &str = "2500";

/// Synchronizes one connected account in full: signs in with its refresh token from the secret
/// store, then fetches every calendar and its events within the window into the cache.
pub fn sync_account(
    client_file: &Path,
    endpoints: &Endpoints,
    store: &dyn SecretStore,
    account_id: &str,
    window: Window,
    cache: &mut Cache,
) -> Result<(), SyncError> {
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
    let access_token = refresh(&client, endpoints, &refresh_token)
        .map_err(|_| SyncError("Google did not accept the account's sign-in".into()))?;
    let google = GoogleCalendar {
        endpoints,
        account_id,
        access_token,
    };
    full_sync(&google, account_id, window, cache)
}

/// One account at the Google Calendar API, signed in.
struct GoogleCalendar<'a> {
    endpoints: &'a Endpoints,
    account_id: &'a str,
    /// Not `Debug`, so it never reaches a log (`NFR-07`).
    access_token: String,
}

impl GoogleCalendar<'_> {
    fn url(&self, segments: &[&str]) -> Url {
        let mut url = Url::parse(&self.endpoints.calendar).expect("a valid Calendar API URL");
        url.path_segments_mut()
            .expect("a base URL")
            .extend(segments);
        url
    }

    /// Every item of a list, following Google's pages.
    fn all<T: DeserializeOwned>(&self, url: Url) -> Result<Vec<T>, SyncError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Page<T> {
            #[serde(default = "Vec::new")]
            items: Vec<T>,
            next_page_token: Option<String>,
        }
        let mut items = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let mut page_url = url.clone();
            if let Some(token) = &page_token {
                page_url.query_pairs_mut().append_pair("pageToken", token);
            }
            let page: Page<T> = self.get(page_url)?;
            items.extend(page.items);
            match page.next_page_token {
                Some(token) => page_token = Some(token),
                None => return Ok(items),
            }
        }
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, SyncError> {
        let mut response = agent(HTTP_TIMEOUT)
            .get(url.as_str())
            .header("Authorization", format!("Bearer {}", self.access_token))
            .call()
            .map_err(|error| SyncError(format!("Google could not be reached ({error})")))?;
        let status = response.status();
        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|error| SyncError(format!("Google's answer could not be read ({error})")))?;
        if !status.is_success() {
            let reason = google_reason(&body).unwrap_or_else(|| status.to_string());
            return Err(SyncError(format!("Google refused ({reason})")));
        }
        serde_json::from_str(&body)
            .map_err(|_| SyncError("Google's answer could not be read".into()))
    }
}

impl Provider for GoogleCalendar<'_> {
    fn calendars(&self) -> Result<Vec<Calendar>, SyncError> {
        // Calendars hidden from the list in Google stay out, as in Google Calendar (`DEC-86`).
        let mut url = self.url(&["users", "me", "calendarList"]);
        url.query_pairs_mut()
            .append_pair("maxResults", CALENDARS_PER_PAGE);
        let entries: Vec<CalendarEntry> = self.all(url)?;
        Ok(entries
            .into_iter()
            .filter_map(|entry| entry.into_calendar(self.account_id))
            .collect())
    }

    fn events(&self, calendar: &Calendar, window: Window) -> Result<Vec<SyncedEvent>, SyncError> {
        let instant = |at: DateTime<Utc>| at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let mut url = self.url(&["calendars", &calendar.id, "events"]);
        // Not `singleEvents`: series come as master plus exceptions, cancelled occurrences
        // included.
        url.query_pairs_mut()
            .append_pair("timeMin", &instant(window.start()))
            .append_pair("timeMax", &instant(window.end()))
            .append_pair("maxResults", EVENTS_PER_PAGE);
        let events: Vec<GoogleEvent> = self.all(url)?;
        Ok(events
            .into_iter()
            .filter_map(|event| event.into_synced(calendar.zone))
            .collect())
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
    /// `None` for what the app cannot show: a cancelled event that is not an occurrence of a
    /// series, or an event without readable times.
    fn into_synced(self, calendar_zone: Tz) -> Option<SyncedEvent> {
        let instance_of = match (self.recurring_event_id, &self.original_start_time) {
            (Some(series), Some(original)) => Some((series, original.start()?)),
            _ => None,
        };
        let cancelled = self.status.as_deref() == Some("cancelled");
        let timing = if cancelled {
            instance_of.as_ref()?;
            None
        } else {
            let (start, end) = (self.start?, self.end?);
            Some(match (start.start()?, end.start()?) {
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
                _ => return None,
            })
        };
        Some(SyncedEvent {
            id: self.id,
            title: self.summary.unwrap_or_default(),
            timing,
            recurrence: self.recurrence,
            instance_of,
        })
    }
}

#[cfg(test)]
#[path = "calendar_tests.rs"]
mod tests;
