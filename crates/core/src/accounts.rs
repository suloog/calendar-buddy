//! Google accounts and connecting one (`MOD-01`): the account, what connecting can end in, and the
//! words the welcome screen uses.

use std::path::PathBuf;

/// A connected Google account and what it allows the app to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    /// Google's stable account ID; the e-mail address can change.
    pub id: String,
    pub email: String,
    /// The account belongs to a Google Workspace organization.
    pub workspace: bool,
    /// Contacts access was granted (contact suggestions).
    pub contacts: bool,
    /// Directory access was granted (colleague suggestions); only asked on Workspace accounts.
    pub directory: bool,
}

impl Account {
    /// The features this account cannot offer because a permission was not granted (`DEC-52`).
    pub fn unavailable(&self) -> Vec<&'static str> {
        let mut features = Vec::new();
        if !self.contacts {
            features.push("contact suggestions (contacts access was not granted)");
        }
        if self.workspace && !self.directory {
            features.push("colleague suggestions (directory access was not granted)");
        }
        features
    }
}

/// Why connecting an account stopped; in every case no account was added.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectError {
    /// The Google OAuth client file is missing or unreadable (`DEC-77`).
    ClientFile(PathBuf),
    /// No secret store runs (`MOD-01-F10`).
    NoSecretStore,
    /// The secret store did not keep the sign-in.
    SecretStore(String),
    /// Google cannot be reached (`MOD-01-F13`).
    Offline,
    /// The user cancelled, in the app or on Google's screen.
    Cancelled,
    /// Nothing came back from the browser in time.
    TimedOut,
    /// Calendar access was unticked on Google's consent screen (`DEC-52`).
    CalendarNotGranted,
    /// Google refused the sign-in, with its reason (e.g. blocked by an administrator).
    Refused(String),
    /// Anything else on the way, e.g. the network dropping mid-way.
    Failed(String),
}

impl ConnectError {
    /// What the welcome screen says.
    pub fn message(&self) -> String {
        match self {
            ConnectError::ClientFile(path) => format!(
                "The Google OAuth client file is missing or unreadable. Download the OAuth \
                 client (type Desktop app) from Google Cloud Console and save it as {}, then try \
                 again.",
                path.display()
            ),
            ConnectError::NoSecretStore => "No secret store is running, and Calendar Buddy keeps \
                 your sign-in only there. Start or install one — KWallet on KDE Plasma, GNOME \
                 Keyring, or KeePassXC with its Secret Service integration turned on — then try \
                 again."
                .to_string(),
            ConnectError::SecretStore(reason) => format!(
                "The secret store did not keep the sign-in ({reason}), so the account was not \
                 connected. Unlock the secret store and try again."
            ),
            ConnectError::Offline => "Signing in is not possible offline. Connect to the \
                 network and try again."
                .to_string(),
            ConnectError::Cancelled => {
                "The sign-in was cancelled, so no account was connected.".to_string()
            }
            ConnectError::TimedOut => {
                "The sign-in was not finished in time, so no account was connected.".to_string()
            }
            ConnectError::CalendarNotGranted => "Calendar access was not granted, so the account \
                 was not connected. Calendar Buddy needs it to show your calendars: start again \
                 and leave calendar access ticked."
                .to_string(),
            ConnectError::Refused(reason) => {
                format!(
                    "Google did not allow the sign-in: {reason}. The account was not connected."
                )
            }
            ConnectError::Failed(reason) => {
                format!("Signing in failed: {reason}. The account was not connected.")
            }
        }
    }
}

/// The welcome screen's one action (`MOD-01-F12`).
pub const CONNECT: &str = "Connect Google account";

/// Shown before the browser opens (`MOD-01-U03`).
pub const UNVERIFIED_HEADS_UP: &str = "Google will now show a warning that it hasn't verified \
     this app — Calendar Buddy is your own app, not one published to everyone. To continue, \
     choose \"Advanced\" and then \"Go to … (unsafe)\". On the next screen leave calendar access \
     ticked; contacts and your organization's directory are optional.";

/// Shown while the browser has the sign-in.
pub const WAITING: &str = "Finish signing in in your browser. On a Workspace account Google asks \
     a second time, for your organization's directory.";

#[cfg(test)]
mod tests {
    use super::*;

    fn account(workspace: bool, contacts: bool, directory: bool) -> Account {
        Account {
            id: "1".into(),
            email: "a@example.com".into(),
            workspace,
            contacts,
            directory,
        }
    }

    #[test]
    fn a_missing_permission_makes_its_feature_unavailable() {
        assert!(account(false, true, false).unavailable().is_empty());
        assert!(account(true, true, true).unavailable().is_empty());
        assert_eq!(
            account(false, false, false).unavailable(),
            ["contact suggestions (contacts access was not granted)"]
        );
        assert_eq!(
            account(true, true, false).unavailable(),
            ["colleague suggestions (directory access was not granted)"]
        );
    }

    #[test]
    fn the_client_file_message_says_which_file_to_put_where() {
        let path = PathBuf::from("/home/u/.config/calendar-buddy/google-oauth-client.json");
        let message = ConnectError::ClientFile(path).message();
        assert!(message.contains("/home/u/.config/calendar-buddy/google-oauth-client.json"));
        assert!(message.contains("Google Cloud Console"));
    }

    #[test]
    fn the_failures_of_a_first_run_explain_themselves() {
        assert!(
            ConnectError::Offline
                .message()
                .contains("not possible offline")
        );
        let store = ConnectError::NoSecretStore.message();
        assert!(store.contains("KWallet") && store.contains("GNOME Keyring"));
        assert!(
            ConnectError::Cancelled
                .message()
                .contains("no account was connected")
        );
        assert!(
            ConnectError::Refused("admin_policy_enforced".into())
                .message()
                .contains("admin_policy_enforced")
        );
        assert!(
            ConnectError::CalendarNotGranted
                .message()
                .contains("Calendar access was not granted")
        );
    }
}
