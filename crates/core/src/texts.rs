//! Every text of the interface, in English (`MOD-04-F04`, `DEC-31`): the window, its dialogs and
//! notices, the desktop notifications, the tray menu and the page the browser shows after
//! signing in. They are kept here, in one table, so a translation replaces this file and touches
//! no view. Dates, times and numbers are not texts: they follow the system (`DEC-29`).

use std::path::Path;

/// The app's name, as the window, its notifications and the tray show it; a debug build is
/// another app (`background::DEV`).
pub const APP_NAME: &str = if crate::background::DEV {
    "Calendar Buddy (dev)"
} else {
    "Calendar Buddy"
};

// The welcome screen and signing in (`MOD-01`).

pub const WELCOME: &str = "Welcome to Calendar Buddy";
/// What connecting Google brings, under the welcome (`MOD-09-U04`).
pub const WELCOME_HINT: &str = "Connect a Google account to synchronize your calendars, answer \
     invitations and meet on Google Meet.";
/// The welcome screen's main action (`MOD-01-F12`).
pub const CONNECT: &str = "Connect Google account";
/// The welcome screen's second action, which starts with a local calendar (`MOD-09-F17`).
pub const USE_WITHOUT_ACCOUNT: &str = "Use without an account";
pub const ADD_ACCOUNT: &str = "Add account";
/// The action that signs in to an account again (`MOD-01-F06`).
pub const SIGN_IN_AGAIN: &str = "Sign in again";
/// Shown while the browser has the sign-in.
pub const WAITING: &str = "Finish signing in in your browser. On a Workspace account Google asks \
     a second time, for your organization's directory.";
/// What the browser shows once the sign-in is back in the app.
pub const SIGNED_IN_PAGE: &str = "You can close this tab and go back to Calendar Buddy.";

pub fn signing_in_again(email: &str) -> String {
    format!("Signing in again to {email}.")
}

pub fn connected(email: &str) -> String {
    format!("Connected {email}.")
}

/// The features an account cannot offer because a permission was not granted (`DEC-52`).
pub const NO_CONTACT_SUGGESTIONS: &str = "contact suggestions (contacts access was not granted)";
pub const NO_COLLEAGUE_SUGGESTIONS: &str =
    "colleague suggestions (directory access was not granted)";

pub fn not_available(features: &[&str]) -> String {
    format!("Not available for this account: {}.", features.join("; "))
}

/// Why connecting an account failed, as the welcome screen says it.
pub fn client_file_missing(path: &Path) -> String {
    format!(
        "The Google OAuth client file is missing or unreadable. Download the OAuth client (type \
         Desktop app) from Google Cloud Console and save it as {}, then try again.",
        path.display()
    )
}
pub const NO_SECRET_STORE: &str = "No secret store is running, and Calendar Buddy keeps your \
     sign-in only there. Start or install one — KWallet on KDE Plasma, GNOME Keyring, or \
     KeePassXC with its Secret Service integration turned on — then try again.";
pub fn sign_in_not_kept(reason: &str) -> String {
    format!(
        "The secret store did not keep the sign-in ({reason}), so the account was not \
         connected. Unlock the secret store and try again."
    )
}
pub const SIGN_IN_OFFLINE: &str =
    "Signing in is not possible offline. Connect to the network and try again.";
pub const SIGN_IN_CANCELLED: &str = "The sign-in was cancelled, so no account was connected.";
pub const SIGN_IN_TIMED_OUT: &str =
    "The sign-in was not finished in time, so no account was connected.";
pub const CALENDAR_NOT_GRANTED: &str = "Calendar access was not granted, so the account was not \
     connected. Calendar Buddy needs it to show your calendars: start again and leave calendar \
     access ticked.";
pub fn sign_in_refused(reason: &str) -> String {
    format!("Google did not allow the sign-in: {reason}. The account was not connected.")
}
pub fn sign_in_failed(reason: &str) -> String {
    format!("Signing in failed: {reason}. The account was not connected.")
}

// Accounts (`MOD-01`).

pub const ACCOUNT_CONNECTED: &str = "Connected";
pub const ACCOUNT_NEEDS_SIGN_IN: &str = "Needs sign-in";

/// What the app says about an account Google turned away (`MOD-01-F05`).
pub fn needs_sign_in(email: &str) -> String {
    format!(
        "{email} needs sign-in: Google ended Calendar Buddy's access to it. Your other accounts \
         keep working."
    )
}

/// The action that removes an account (`MOD-01-F07`).
pub const REMOVE: &str = "Remove";

pub fn confirm_removal(email: &str) -> String {
    format!(
        "Remove {email}? Its calendars, events, credentials and local settings will be deleted \
         from this computer, and Calendar Buddy's access to it will be withdrawn at Google. The \
         Google account itself stays untouched: its calendars and events stay in Google."
    )
}

pub fn removing(email: &str) -> String {
    format!("Removing {email}…")
}

pub fn not_removed_no_secret_store(email: &str) -> String {
    format!(
        "{email} was not removed: no secret store is running, and its credentials are kept \
         there. Start the secret store and try again."
    )
}

pub fn not_removed(email: &str, reason: &str) -> String {
    format!(
        "{email} was not removed: the secret store did not delete its credentials ({reason}). \
         Unlock the secret store and try again."
    )
}

/// Why the app cannot reach the accounts; the calendar still shows what it has.
const UNTIL: &str = "Your calendars are shown as they were, but nothing synchronizes or changes";
pub fn client_file_gone(path: &Path) -> String {
    format!(
        "The Google OAuth client file {} is missing or unreadable. {UNTIL} until it is back.",
        path.display()
    )
}
pub fn secret_store_not_running() -> String {
    format!(
        "No secret store is running, and your sign-ins are kept only there. {UNTIL} until it \
         runs."
    )
}
pub fn secret_store_locked(reason: &str) -> String {
    format!("The secret store is locked ({reason}). {UNTIL} until it is unlocked.")
}

// Reasons a sign-in, a synchronization or a change failed, put into the messages above and below.

pub const UNLOCKING_DECLINED: &str = "unlocking it was declined";
pub const SECRET_NOT_READ: &str = "the saved sign-in could not be read";
pub const NO_REFRESH_TOKEN: &str = "Google sent no refresh token";
pub const ANSWER_NOT_READ: &str = "Google's answer could not be read";
pub const BROWSER_NOT_OPENED: &str = "the browser could not be opened";
pub fn client_file_unreadable(path: &Path) -> String {
    format!(
        "the Google OAuth client file {} is missing or unreadable",
        path.display()
    )
}
pub const NO_SECRET_STORE_RUNNING: &str = "no secret store is running";
pub fn secret_store_is_locked(reason: &str) -> String {
    format!("the secret store is locked ({reason})")
}
pub const ACCOUNT_NEEDS_SIGN_IN_REASON: &str = "the account needs sign-in";
pub const SIGN_IN_NOT_ACCEPTED: &str = "Google did not accept the account's sign-in";
/// How an error starts when Google answered, but refused what was asked.
pub const REFUSED: &str = "Google refused";
/// How an error starts when Google refused what was asked as invalid (400), as it refuses a
/// Google Meet call the account may not have.
pub const REFUSED_AS_INVALID: &str = "Google refused it as invalid";
pub const GONE: &str = "Google refused (gone)";
pub fn unreachable(error: &str) -> String {
    format!("Google could not be reached ({error})")
}
pub fn answer_unreadable(error: &str) -> String {
    format!("Google's answer could not be read ({error})")
}
pub fn cache_not_used(error: &str) -> String {
    format!("the local cache could not be used ({error})")
}
pub const CACHE_NOT_READ: &str = "the local cache could not be read";
pub fn cache_not_made(error: &str) -> String {
    format!("the local cache could not be made ({error})")
}
/// Why a synchronization did not finish when the provider asked to slow down.
pub const RATE_LIMITED: &str = "the provider asked to slow down (rate limit)";
/// Why a change was not made: the event changed at the provider since the version it was made to.
pub const CHANGED_ELSEWHERE: &str = "it was changed elsewhere in the meantime";
/// Why a change to a series was not sent: its series is not in the cache, as for an invitation to
/// one occurrence of someone else's series, or not synchronized yet.
pub const NO_SERIES: &str =
    "its series is not in the calendar (yet); only this event can be changed or deleted";
/// Why a change was not sent: the event's account has not signed in in this run yet.
pub const NOT_SIGNED_IN: &str = "its account is not signed in yet; try again in a moment";
pub fn make_it_again(reason: &str) -> String {
    format!("{reason}; it is shown as it is now, make your change again")
}
pub fn not_moved(reason: &str) -> String {
    format!("it could not be moved to the other calendar ({reason}); its other changes were saved")
}
pub fn following_not_split(reason: &str) -> String {
    format!(
        "the series now ends before this event, but the following events could not be made a \
         series of their own ({reason})"
    )
}
pub fn changed_one_not_deleted(reason: &str) -> String {
    format!(
        "the series now ends before this event, but an event of it changed on its own could not \
         be deleted ({reason})"
    )
}

// The main window (`MOD-03`).

pub const HIDE_SIDE_PANEL: &str = "Hide the side panel";
pub const SHOW_SIDE_PANEL: &str = "Show the side panel";
pub const TODAY: &str = "Today";
pub const UPCOMING: &str = "Upcoming";
pub const SETTINGS: &str = "Settings";
pub const REFRESH: &str = "Refresh";
pub const HIDE_DECLINED: &str = "Hide declined events";
pub const SHOW_DECLINED: &str = "Show declined events";
pub const TRY_AGAIN: &str = "Try again";
pub const CANCEL: &str = "Cancel";
pub const UNDO: &str = "Undo";
pub const GO: &str = "Go";
pub const SAVE: &str = "Save";
pub const EDIT: &str = "Edit";
pub const DELETE: &str = "Delete";

/// The views, as the view bar names them.
pub const DAY: &str = "Day";
pub const THREE_DAYS: &str = "3 days";
pub const WEEK: &str = "Week";
pub const MONTH: &str = "Month";
pub const AGENDA: &str = "Agenda";

/// What the empty view says when no calendar is shown.
pub const NOTHING_SHOWN: &str = "No calendars are shown. Choose calendars in the side panel.";
pub const NO_UPCOMING_EVENTS: &str = "No upcoming events";
pub const ALL_DAY: &str = "All day";
pub const JOIN: &str = "Join";

/// A month-view day with more events than fit.
pub fn more(count: usize) -> String {
    format!("+{count} more")
}

pub fn calendar_gone(name: &str) -> String {
    format!("The calendar “{name}” is no longer available.")
}

pub fn date_as(example: &str) -> String {
    format!("Date (as {example})")
}

/// The dialogs' titles.
pub const KEYBOARD_SHORTCUTS: &str = "Keyboard shortcuts";
/// The groups of the shortcuts' list.
pub const SHORTCUTS_VIEWS: &str = "Views";
pub const SHORTCUTS_NAVIGATION: &str = "Navigation";
pub const SHORTCUTS_ACTIONS: &str = "Actions";
pub const GO_TO_DATE: &str = "Go to date";
pub const NEW_EVENT: &str = "New event";
pub const EDIT_EVENT: &str = "Edit event";
pub const EVENT: &str = "Event";
pub const CLOSE: &str = "Close";
pub const EDIT_RECURRING_EVENT: &str = "Edit recurring event";
pub const DELETE_RECURRING_EVENT: &str = "Delete recurring event";
pub const RESPOND_TO_RECURRING_EVENT: &str = "Respond to recurring event";
pub const NOTIFY_GUESTS: &str = "Notify guests";
pub const INVITATIONS: &str = "Invitations";

/// Google Calendar's keyboard shortcuts, as their list says them (`DEC-55`).
pub const DAY_VIEW: &str = "Day view";
pub const THREE_DAY_VIEW: &str = "3-day view";
pub const WEEK_VIEW: &str = "Week view";
pub const MONTH_VIEW: &str = "Month view";
pub const AGENDA_VIEW: &str = "Agenda view";
pub const NEXT_PERIOD: &str = "Next period";
pub const NEXT_MONTH: &str = "Next month";
pub const PREVIOUS_MONTH: &str = "Previous month";
pub const PREVIOUS_PERIOD: &str = "Previous period";

// The side panel and the calendars (`MOD-02`).

pub const ACCOUNTS: &str = "Accounts";
pub const CALENDARS: &str = "Calendars";
pub const DEFAULT_CALENDAR: &str = "Default calendar";
/// What a calendar's checkbox in the settings says on hover.
pub const ENABLED_HINT: &str =
    "A disabled calendar is not shown in the side panel or the views, and is not synchronized";
/// What a calendar's lock says on hover.
pub const READ_ONLY: &str = "Read-only: this calendar can't be changed";
pub const HIDE_ITS_EVENTS: &str = "Hide its events";
pub const SHOW_ITS_EVENTS: &str = "Show its events";
pub const RESET: &str = "Reset";
pub const MUTE: &str = "Mute notifications";
pub const UNMUTE: &str = "Unmute notifications";

/// The colours Google Calendar offers for calendars and events, by Google's names.
pub const RADICCHIO: &str = "Radicchio";
pub const TANGERINE: &str = "Tangerine";
pub const CITRON: &str = "Citron";
pub const BASIL: &str = "Basil";
pub const BLUEBERRY: &str = "Blueberry";
pub const GRAPE: &str = "Grape";
pub const CHERRY_BLOSSOM: &str = "Cherry blossom";
pub const PUMPKIN: &str = "Pumpkin";
pub const AVOCADO: &str = "Avocado";
pub const EUCALYPTUS: &str = "Eucalyptus";
pub const LAVENDER: &str = "Lavender";
pub const COCOA: &str = "Cocoa";
pub const TOMATO: &str = "Tomato";
pub const MANGO: &str = "Mango";
pub const PISTACHIO: &str = "Pistachio";
pub const PEACOCK: &str = "Peacock";
pub const WISTERIA: &str = "Wisteria";
pub const GRAPHITE: &str = "Graphite";
pub const FLAMINGO: &str = "Flamingo";
pub const BANANA: &str = "Banana";
pub const SAGE: &str = "Sage";
pub const COBALT: &str = "Cobalt";
pub const AMETHYST: &str = "Amethyst";
pub const BIRCH: &str = "Birch";
pub const CALENDAR_COLOUR: &str = "Calendar colour";

// Synchronization and working offline (`MOD-02`).

pub const SYNCHRONIZING: &str = "Synchronizing…";
pub const UP_TO_DATE: &str = "Up to date";
pub const OFFLINE: &str = "Offline";
pub const SYNC_DELAYED: &str = "Synchronization delayed: Google asked to slow down";
pub const SYNC_ERROR: &str = "Synchronization error";
pub const NOT_SYNCHRONIZED_YET: &str = "Not synchronized yet";
/// The synchronization's state with the time of the last one: `Up to date (14:05)`.
pub fn state_since(state: &str, when: &str) -> String {
    format!("{state} ({when})")
}
/// A date and a time together: `Wed 11 Nov 2026, 14:05`.
pub fn date_time(date: &str, time: &str) -> String {
    format!("{date}, {time}")
}
pub fn last_synchronized_at(time: &str) -> String {
    format!("Last synchronized at {time}")
}
pub fn last_synchronized_on(date: &str, time: &str) -> String {
    format!("Last synchronized on {date} at {time}")
}
/// A problem as its calendar's row in the side panel says it (`DEC-97`).
pub const ROW_FAILED: &str = "Could not be synchronized; trying again.";
pub const ROW_DELAYED: &str = "Synchronized later: Google asked to slow down.";
pub const ROW_NOT_SYNCHRONIZING: &str = "Not synchronizing: its account needs sign-in.";
/// The offline indicator in the main window.
pub const OFFLINE_INDICATOR: &str = "Offline — your calendars are shown as last synchronized.";
/// Why a change cannot be made offline.
pub const NO_CHANGES_OFFLINE: &str =
    "Creating, editing and deleting events and answering invitations are not available offline.";
/// The note over days the cache does not hold, while offline.
pub const NOT_CACHED: &str =
    "Days more than a year back or two years ahead are not available offline.";

// Events, and creating and editing them (`MOD-05`).

/// What the views add to the label of an event Google has not confirmed yet (`MOD-05-U03`).
pub const SAVING: &str = "Saving…";
/// What Google Calendar names an event without a title.
pub const NO_TITLE: &str = "(No title)";
/// The title of an event of a calendar shared as free/busy only.
pub const BUSY: &str = "Busy";
/// What the quick-entry box says when the user has no calendar to add events to.
pub const NO_CALENDAR: &str = "No calendar you can add events to.";
pub const ADD_TITLE: &str = "Add title";
pub const MORE_OPTIONS: &str = "More options";
/// What the notice says after a deletion.
pub const DELETED: &str = "Event deleted.";
/// What the notice says after a change.
pub const SAVED: &str = "Event saved.";
pub fn not_saved(reason: &str) -> String {
    format!("The event could not be saved: {reason}.")
}
pub fn not_deleted(reason: &str) -> String {
    format!("The event could not be deleted: {reason}.")
}
pub fn not_undone(reason: &str) -> String {
    format!("The change could not be undone: {reason}.")
}
pub fn meeting_not_opened(error: &str) -> String {
    format!("The meeting could not be opened: {error}.")
}
pub fn google_calendar_not_opened(error: &str) -> String {
    format!("Google Calendar could not be opened: {error}.")
}
pub fn attachment_not_opened(error: &str) -> String {
    format!("The attachment could not be opened: {error}.")
}

/// Why the user cannot change an event.
pub const READ_ONLY_EVENT: &str = "🔒 This event can't be changed: its calendar is read-only.";
pub const ORGANIZER_ONLY: &str = "This event can't be changed: only its organizer can change it.";
pub const DISPLAY_ONLY: &str = "This event can't be changed here: working locations and \
     birthdays are changed in Google.";

/// The types of events.
pub const KIND_EVENT: &str = "Event";
pub const OUT_OF_OFFICE: &str = "Out of office";
pub const FOCUS_TIME: &str = "Focus time";
/// What Google tells the organizer of an invitation that clashes with out of office or focus
/// time, as Google Calendar words it.
pub const DECLINED_OUT_OF_OFFICE: &str = "Declined because I am out of office";
pub const DECLINED_FOCUS_TIME: &str = "Declined because I am in focus time";
pub const WORKING_LOCATION: &str = "Working location";
pub const BIRTHDAY: &str = "Birthday";

/// The editor's fields.
pub const TYPE: &str = "Type";
pub const TITLE: &str = "Title";
pub const STARTS: &str = "Starts";
pub const ENDS: &str = "Ends";
pub const TIME_ZONE: &str = "Time zone";
pub const AVAILABILITY: &str = "Availability";
pub const REPEATS: &str = "Repeats";
pub const LOCATION: &str = "Location";
pub const ADD_LOCATION: &str = "Add location";
pub const GUESTS: &str = "Guests";
pub const ADD_GUESTS: &str = "Add guests";
pub const ADD_GOOGLE_MEET: &str = "Add Google Meet";
pub const NOTIFICATIONS: &str = "Notifications";
pub const DESCRIPTION: &str = "Description";
pub const ADD_DESCRIPTION: &str = "Add description";
pub const EDIT_FORMATTED_DESCRIPTION: &str = "Click to edit";
pub const FORMATTING_WILL_BE_LOST: &str = "This description has formatting from Google Calendar, which the app can't edit. Editing it here removes the formatting.";
pub const EDIT_AS_PLAIN_TEXT: &str = "Edit as plain text";
pub const CALENDAR: &str = "Calendar";
pub const COLOUR: &str = "Colour";
pub const SHOW_AS: &str = "Show as";
pub const VISIBILITY: &str = "Visibility";
/// How the event shows the user's time to others.
pub const SHOW_BUSY: &str = "Busy";
pub const FREE: &str = "Free";
pub const DEFAULT_VISIBILITY: &str = "Default visibility";
pub const PUBLIC: &str = "Public";
pub const PRIVATE: &str = "Private";

/// Why the editor cannot save.
pub const ENDS_BEFORE_STARTS: &str = "The event ends before it starts.";
pub const SERIES_ENDS_BEFORE_IT_STARTS: &str = "The series ends before it starts.";
pub const REMINDER_TOO_EARLY: &str = "A reminder can be at most 4 weeks before the event.";
pub const TOO_MANY_REMINDERS: &str = "An event can have at most 5 reminders.";
pub const SERIES_END_UNREADABLE: &str = "The end date of the series can't be read.";
pub const COUNT_UNREADABLE: &str = "The number of occurrences can't be read.";
pub const START_DATE_UNREADABLE: &str = "The start date can't be read.";
pub const END_DATE_UNREADABLE: &str = "The end date can't be read.";
pub const START_TIME_UNREADABLE: &str = "The start time can't be read.";
pub const END_TIME_UNREADABLE: &str = "The end time can't be read.";
/// What the app says when an event saved with Google Meet came back without a call
/// (MOD-06 *Constraints and edge cases*).
pub const WITHOUT_MEET: &str =
    "The event was saved without Google Meet: Google could not add a Meet call for this account.";

/// How an event repeats, and how its series ends (`MOD-05-F18`, `F19`).
pub const DOES_NOT_REPEAT: &str = "Does not repeat";
pub const DAILY: &str = "Daily";
pub const EVERY_WEEKDAY: &str = "Every weekday (Monday to Friday)";
pub const WEEKLY: &str = "Weekly";
pub const MONTHLY: &str = "Monthly";
pub const ANNUALLY: &str = "Annually";
pub const ON: &str = "On";
pub const SERIES_ENDS: &str = "Series ends";
pub const NEVER: &str = "Never";
pub const AFTER: &str = "After";
pub const OCCURRENCES: &str = "occurrences";
pub const CUSTOM: &str = "Custom…";
/// Google Calendar's custom recurrence dialog (`MOD-05-F33`).
pub const CUSTOM_RECURRENCE: &str = "Custom recurrence";
pub const REPEAT_EVERY: &str = "Repeat every";
pub const REPEAT_ON: &str = "Repeat on";
pub const DAY_UNIT: &str = "day";
pub const WEEK_UNIT: &str = "week";
pub const MONTH_UNIT: &str = "month";
pub const YEAR_UNIT: &str = "year";
pub const DONE: &str = "Done";
pub const INTERVAL_UNREADABLE: &str = "The number to repeat every can't be read.";
pub const CHANGED_IN_GOOGLE: &str = "Changed in Google Calendar";
pub const OPEN_IN_GOOGLE: &str = "Open in Google Calendar";

/// The reminders of an event (`MOD-07`).
pub const NOTIFICATION: &str = "Notification";
pub const EMAIL: &str = "Email";
pub const MINUTES: &str = "minutes";
pub const HOURS: &str = "hours";
pub const DAYS: &str = "days";
pub const ADD_NOTIFICATION: &str = "Add notification";
pub const USE_CALENDAR_DEFAULTS: &str = "Use calendar defaults";

// Series (`MOD-05`).

/// The scopes of a change to a series, as Google Calendar asks them.
pub const THIS_EVENT: &str = "This event";
pub const THIS_AND_FOLLOWING: &str = "This and following events";
pub const ALL_EVENTS: &str = "All events";
/// Why the following or all events cannot move to another day, or between all-day and timed.
pub const OTHER_DAY: &str = "Moving the following or all events to another day changes how the \
     series repeats; that is done in Google Calendar.";
/// Why a new rule, set from an occurrence, cannot be for all events (`DEC-172`).
pub const RULE_FROM_FIRST: &str = "From the series' first event this rule falls on other days; \
     it can be set for this and following events.";
/// Why an occurrence, or those following it, cannot move to another calendar.
pub const OTHER_CALENDAR: &str = "Only all events of a series can move to another calendar.";

// Guests and invitations (`MOD-06`).

pub const GOING: &str = "Going?";
pub const JOIN_WITH_MEET: &str = "Join with Google Meet";
pub const OPTIONAL: &str = "Optional";
/// The hint of a guest's switch, as Google Calendar's.
pub const MARK_OPTIONAL: &str = "Mark optional";
pub const MARK_REQUIRED: &str = "Mark required";
pub const SHOW_FEWER: &str = "Show fewer";
pub fn show_more(count: usize) -> String {
    format!("Show {count} more")
}
pub fn guests(count: usize) -> String {
    match count {
        1 => "1 guest".to_string(),
        n => format!("{n} guests"),
    }
}
/// What the guest field says when what was typed is not an e-mail address.
pub const NOT_AN_ADDRESS: &str = "This is not a valid e-mail address.";
/// A guest's answer, in words.
pub const ACCEPTED: &str = "Accepted";
pub const DECLINED: &str = "Declined";
pub const MAYBE: &str = "Maybe";
pub const AWAITING: &str = "Awaiting";
/// The answers to an invitation, as their buttons say them (`MOD-06-U03`).
pub const ACCEPT: &str = "Accept";
pub const DECLINE: &str = "Decline";
/// What the list of invitations says when none awaits an answer.
pub const NO_INVITATIONS: &str = "No invitations await an answer.";
/// What the side panel's counter says, with the number of invitations awaiting an answer.
pub fn invitations_awaiting(count: usize) -> String {
    match count {
        1 => "1 invitation awaiting an answer".to_string(),
        n => format!("{n} invitations awaiting an answer"),
    }
}
/// The side panel's "Meet with…" (`MOD-06-F23`…`F27`): its field, why it is unavailable
/// offline, a person whose calendar shares nothing, and the button starting a meeting.
pub const MEET_WITH: &str = "Meet with…";
pub const MEET_WITH_OFFLINE: &str = "Meet with… needs a connection to Google.";
/// "Meet with…" with no Google account connected (MOD-09 *Constraints and edge cases*).
pub const MEET_WITH_NEEDS_ACCOUNT: &str = "Meet with… needs a Google account.";
pub const CALENDAR_CANNOT_BE_SEEN: &str = "Calendar can't be seen";
pub const CREATE_EVENT: &str = "Create event";
/// What a strip says of a guest who does not share their availability.
pub const NO_DATA: &str = "No data";
/// What a strip says of a guest busy at the chosen time.
pub const CONFLICT: &str = "Conflict";
/// What the question about e-mails to the guests says for a change and a deletion, and its
/// answers, as Google words them; Back saves nothing.
pub const SEND_UPDATE: &str = "Send update e-mails to the guests?";
pub const SEND_CANCELLATION: &str = "Send cancellation e-mails to the guests?";
pub const SEND: &str = "Send";
pub const DONT_SEND: &str = "Don't send";
pub const BACK: &str = "Back";

// Notifications, the tray and settings (`MOD-07`, `MOD-04`).

pub const OPEN: &str = "Open";
pub const DISMISS: &str = "Dismiss";
pub fn snooze(minutes: u32) -> String {
    format!("Snooze {minutes} minutes")
}
pub const OPEN_APP: &str = "Open Calendar Buddy";
pub const QUIT: &str = "Quit";
/// The settings' switch of all notifications (`MOD-07-F11`), as Google Calendar names it.
pub const DESKTOP_NOTIFICATIONS: &str = "Desktop notifications";
pub const DESKTOP_NOTIFICATIONS_HINT: &str = "Reminders of events are shown as desktop \
     notifications; turning them off leaves the reminders in Google as they are";
/// The name of the setting that starts the app at login (`MOD-07-F16`).
pub const START_AT_LOGIN: &str = "Start at login";
pub const START_AT_LOGIN_HINT: &str = "Calendar Buddy starts in the system tray when you log in, so reminders arrive without opening \
     it.";
pub const APPEARANCE: &str = "Appearance";
pub const THEME: &str = "Theme";
pub const SYSTEM: &str = "System";
pub const LIGHT: &str = "Light";
pub const DARK: &str = "Dark";
/// The setting of how tall the hours of the day, 3-day and week views are (`MOD-03-F28`).
pub const HOUR_HEIGHT: &str = "Hour height";

// Local calendars (`MOD-09`).

/// The group of local calendars, after the accounts (`MOD-09-U02`).
pub const LOCAL: &str = "Local";
/// The local calendar "Use without an account" makes (`MOD-09-F17`).
pub const MY_CALENDAR: &str = "My calendar";
/// The settings' section of local calendars (`MOD-09-U01`).
pub const LOCAL_CALENDARS: &str = "Local calendars";
/// The field naming a new local calendar, and its button (`MOD-09-F02`).
pub const NEW_CALENDAR_NAME: &str = "Name of a new calendar";
pub const ADD_CALENDAR: &str = "Add calendar";
/// Hovering a local calendar's name in the settings (`MOD-09-F03`).
pub const RENAME_HINT: &str = "Type a new name and press Enter";
/// Hovering a local calendar's colour in the settings (`MOD-09-F03`).
pub const COLOR_HINT: &str = "Choose its colour";
/// Deleting a local calendar with events asks first (`MOD-09-U03`).
pub fn confirm_local_deletion(name: &str, events: usize) -> String {
    let events = match events {
        1 => "its event".to_string(),
        n => format!("its {n} events"),
    };
    format!("Delete “{name}”? It will be deleted with {events}. This cannot be undone.")
}
/// The local file could not be read at start; it is left as it is (`MOD-09-NFR-01`).
pub fn local_not_read(path: &Path, reason: &str) -> String {
    format!(
        "Local calendars cannot be read from {}: {reason}. The file is left as it is.",
        path.display()
    )
}
/// Why a change to the local calendars was not kept.
pub fn local_not_used(error: &str) -> String {
    format!("the local calendars could not be used ({error})")
}
/// Why the local file could not be read: something in it is damaged.
pub const LOCAL_DAMAGED: &str = "a calendar or an event in it is damaged";
/// Why the local file could not be read: a later version of the app made it.
pub const LOCAL_FROM_NEWER: &str = "it was made by a newer version of Calendar Buddy";
/// A local event changed, moved or restored that is no longer in the local calendars' file.
pub const LOCAL_EVENT_GONE: &str = "the event is no longer in the local calendar";

#[cfg(test)]
mod tests {
    #[test]
    fn every_text_is_in_english() {
        // English letters only, with the typographic marks English text uses and the lock of a
        // read-only event (`MOD-04-F04`).
        let table = include_str!("texts.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let marks = ['…', '—', '–', '“', '”', '’', '🔒'];
        for (n, line) in table.lines().enumerate() {
            for c in line.chars() {
                assert!(c.is_ascii() || marks.contains(&c), "line {}: {c:?}", n + 1);
            }
        }
    }
}
