//! Google accounts and connecting one (`MOD-01`): the account, what connecting can end in, and the
//! words the welcome screen uses.

use std::path::{Path, PathBuf};
use std::time::Duration;

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

/// An account's state in the settings (`MOD-01-U01`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountState {
    Connected,
    /// Google ended the app's access; the account waits for the user to sign in again.
    NeedsSignIn,
}

impl AccountState {
    pub fn label(self) -> &'static str {
        match self {
            AccountState::Connected => "Connected",
            AccountState::NeedsSignIn => "Needs sign-in",
        }
    }

    fn key(self) -> &'static str {
        match self {
            AccountState::Connected => "connected",
            AccountState::NeedsSignIn => "needs-sign-in",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub account: Account,
    pub state: AccountState,
}

/// The connected accounts, personal and Workspace side by side (`MOD-01-F02`), in the order they
/// were connected and keyed by Google's account ID. There is no cap: five is the profile the app
/// is measured on (`MOD-01-NFR-01`, `DEC-14`).
#[derive(Clone, Debug, Default)]
pub struct Accounts {
    entries: Vec<Entry>,
}

impl Accounts {
    /// Adds a newly signed-in account and says whether it is new. Signing in to an account that
    /// is already connected refreshes it in place instead of adding it twice.
    pub fn add(&mut self, account: Account) -> bool {
        let entry = Entry {
            account,
            state: AccountState::Connected,
        };
        match self
            .entries
            .iter_mut()
            .find(|e| e.account.id == entry.account.id)
        {
            Some(existing) => {
                *existing = entry;
                false
            }
            None => {
                self.entries.push(entry);
                true
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn list(&self) -> &[Entry] {
        &self.entries
    }

    /// The accounts whose sign-in Google is asked about; one needing sign-in waits for the user.
    pub fn connected(&self) -> Vec<Account> {
        self.entries
            .iter()
            .filter(|e| e.state == AccountState::Connected)
            .map(|e| e.account.clone())
            .collect()
    }

    /// Takes what checking the accounts found and says whether any state changed. Only the
    /// account Google turned away needs sign-in; the others keep working (`MOD-01-F05`).
    pub fn apply(&mut self, checks: &[(String, Check)]) -> bool {
        let mut changed = false;
        for (id, check) in checks {
            if *check != Check::SignInNeeded {
                continue;
            }
            if let Some(entry) = self.entries.iter_mut().find(|e| e.account.id == *id) {
                changed |= entry.state != AccountState::NeedsSignIn;
                entry.state = AccountState::NeedsSignIn;
            }
        }
        changed
    }

    /// One account per line: Google's account ID, the e-mail address, what was granted and the
    /// state, separated by tabs. Nothing secret is in it; tokens stay in the secret store.
    pub fn parse(text: &str) -> Self {
        let entries = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t');
                let (id, email, grants, state) = (
                    fields.next()?,
                    fields.next()?,
                    fields.next()?,
                    fields.next()?,
                );
                let granted = |name| grants.split(' ').any(|g| g == name);
                let state = [AccountState::Connected, AccountState::NeedsSignIn]
                    .into_iter()
                    .find(|s| s.key() == state)?;
                Some(Entry {
                    account: Account {
                        id: id.to_string(),
                        email: email.to_string(),
                        workspace: granted("workspace"),
                        contacts: granted("contacts"),
                        directory: granted("directory"),
                    },
                    state,
                })
            })
            .filter(|entry| !entry.account.id.is_empty())
            .collect();
        Accounts { entries }
    }

    pub fn to_text(&self) -> String {
        self.entries
            .iter()
            .map(|Entry { account, state }| {
                let grants: Vec<&str> = [
                    (account.workspace, "workspace"),
                    (account.contacts, "contacts"),
                    (account.directory, "directory"),
                ]
                .into_iter()
                .filter_map(|(has, name)| has.then_some(name))
                .collect();
                format!(
                    "{}\t{}\t{}\t{}\n",
                    account.id,
                    account.email,
                    grants.join(" "),
                    state.key()
                )
            })
            .collect()
    }

    /// The accounts remembered from the last run; none when the file is missing or unreadable.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|text| Accounts::parse(&text))
            .unwrap_or_default()
    }

    /// Written aside and then moved over the file, so a crash mid-way does not forget them all.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let aside = path.with_extension("new");
        std::fs::write(&aside, self.to_text())?;
        std::fs::rename(aside, path)
    }
}

/// `$XDG_CONFIG_HOME/calendar-buddy/accounts`, or under `~/.config` when that is not set.
pub fn default_path() -> Option<PathBuf> {
    Some(crate::settings::config_dir()?.join("accounts"))
}

/// What asking Google about one connected account's sign-in found (`MOD-01-F04`, `F05`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    /// Google still accepts the sign-in.
    Works,
    /// Google revoked or ended the app's access (`invalid_grant`), or the secret store holds no
    /// sign-in for the account.
    SignInNeeded,
    /// Google could not be reached or did not answer; the account stays connected.
    Failed(String),
}

/// Why no account could be checked; every account stays as it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    /// The Google OAuth client file is missing or unreadable (`DEC-77`).
    ClientFile(PathBuf),
    /// No secret store runs.
    NoSecretStore,
    /// The secret store stayed locked, e.g. the user declined to unlock it (`DEC-52`).
    SecretStore(String),
}

impl CheckError {
    /// What the app says above the calendar, which still shows what it has.
    pub fn message(&self) -> String {
        let until = "Your calendars are shown as they were, but nothing synchronizes or changes";
        match self {
            CheckError::ClientFile(path) => format!(
                "The Google OAuth client file {} is missing or unreadable. {until} until it is \
                 back.",
                path.display()
            ),
            CheckError::NoSecretStore => format!(
                "No secret store is running, and your sign-ins are kept only there. {until} \
                 until it runs."
            ),
            CheckError::SecretStore(reason) => {
                format!("The secret store is locked ({reason}). {until} until it is unlocked.")
            }
        }
    }
}

/// How often Google is asked whether the connected accounts still work, so a revoked one is
/// noticed while the app runs.
pub const CHECK_EVERY: Duration = Duration::from_secs(10 * 60);
/// How soon an account Google did not answer about is asked about again.
pub const RETRY_AFTER: Duration = Duration::from_secs(60);

/// When to check the accounts again after a check; `None` waits for the user's "Try again", so a
/// locked secret store does not ask to be unlocked over and over.
pub fn next_check(result: &Result<Vec<(String, Check)>, CheckError>) -> Option<Duration> {
    match result {
        Err(_) => None,
        Ok(checks) if checks.iter().any(|(_, c)| matches!(c, Check::Failed(_))) => {
            Some(RETRY_AFTER)
        }
        Ok(_) => Some(CHECK_EVERY),
    }
}

/// What the app says about an account Google turned away (`MOD-01-F05`).
pub fn needs_sign_in(email: &str) -> String {
    format!(
        "{email} needs sign-in: Google ended Calendar Buddy's access to it. Your other accounts \
         keep working."
    )
}

/// The action that signs in to an account again (`MOD-01-F06`).
pub const SIGN_IN_AGAIN: &str = "Sign in again";

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

    fn signed_in(id: &str, email: &str) -> Account {
        Account {
            id: id.into(),
            email: email.into(),
            ..account(false, true, false)
        }
    }

    fn listed(accounts: &Accounts) -> Vec<(&str, AccountState)> {
        accounts
            .list()
            .iter()
            .map(|entry| (entry.account.email.as_str(), entry.state))
            .collect()
    }

    #[test]
    fn a_second_account_is_listed_beside_the_first() {
        let mut accounts = Accounts::default();
        assert!(accounts.is_empty());
        assert!(accounts.add(signed_in("1", "me@gmail.com")));
        assert!(accounts.add(Account {
            workspace: true,
            ..signed_in("2", "me@work.example.com")
        }));
        assert_eq!(
            listed(&accounts),
            [
                ("me@gmail.com", AccountState::Connected),
                ("me@work.example.com", AccountState::Connected),
            ]
        );
    }

    #[test]
    fn five_accounts_are_connected_at_the_same_time() {
        let mut accounts = Accounts::default();
        for n in 1..=5 {
            assert!(accounts.add(signed_in(&n.to_string(), &format!("user{n}@example.com"))));
        }
        assert_eq!(accounts.list().len(), 5);
        assert!(
            accounts
                .list()
                .iter()
                .all(|entry| entry.state == AccountState::Connected)
        );
    }

    #[test]
    fn signing_in_to_a_connected_account_refreshes_it_without_a_duplicate() {
        let mut accounts = Accounts::default();
        accounts.add(signed_in("1", "me@gmail.com"));
        accounts.add(signed_in("2", "me@work.example.com"));
        accounts.entries[0].state = AccountState::NeedsSignIn;

        // The same Google account, now with another address and contacts no longer granted.
        let again = Account {
            contacts: false,
            ..signed_in("1", "renamed@gmail.com")
        };
        assert!(!accounts.add(again.clone()));

        assert_eq!(
            listed(&accounts),
            [
                ("renamed@gmail.com", AccountState::Connected),
                ("me@work.example.com", AccountState::Connected),
            ]
        );
        assert_eq!(accounts.list()[0].account, again);
    }

    #[test]
    fn the_accounts_and_their_states_are_saved_and_read_back() {
        let mut accounts = Accounts::default();
        accounts.add(signed_in("1", "me@gmail.com"));
        accounts.add(Account {
            workspace: true,
            directory: true,
            ..signed_in("2", "me@work.example.com")
        });
        accounts.add(account(true, false, false));
        accounts.entries[1].state = AccountState::NeedsSignIn;

        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-accounts-{}", std::process::id()));
        let path = dir.join("accounts");
        accounts.save(&path).unwrap();
        let read = Accounts::load(&path);
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(read.list(), accounts.list());
        assert_eq!(
            accounts.to_text().lines().nth(1),
            Some("2\tme@work.example.com\tworkspace contacts directory\tneeds-sign-in")
        );
    }

    #[test]
    fn a_missing_file_or_a_broken_line_forgets_only_what_cannot_be_read() {
        assert!(Accounts::load(Path::new("/nonexistent/calendar-buddy/accounts")).is_empty());
        let read = Accounts::parse(
            "1\tme@gmail.com\tcontacts\tconnected\nnoise\n2\tx@y.z\t\tunknown-state\n",
        );
        assert_eq!(listed(&read), [("me@gmail.com", AccountState::Connected)]);
    }

    #[test]
    fn only_the_account_google_turned_away_needs_sign_in() {
        let mut accounts = Accounts::default();
        accounts.add(signed_in("1", "me@gmail.com"));
        accounts.add(signed_in("2", "me@work.example.com"));
        accounts.add(signed_in("3", "third@example.com"));

        let changed = accounts.apply(&[
            ("1".into(), Check::Works),
            ("2".into(), Check::SignInNeeded),
            ("3".into(), Check::Failed("offline".into())),
        ]);

        assert!(changed);
        assert_eq!(
            listed(&accounts),
            [
                ("me@gmail.com", AccountState::Connected),
                ("me@work.example.com", AccountState::NeedsSignIn),
                ("third@example.com", AccountState::Connected),
            ]
        );
        // An account waiting for sign-in is not asked about again.
        assert_eq!(
            accounts
                .connected()
                .iter()
                .map(|a| a.id.as_str())
                .collect::<Vec<_>>(),
            ["1", "3"]
        );
        assert!(!accounts.apply(&[("2".into(), Check::SignInNeeded)]));
    }

    #[test]
    fn signing_in_again_restores_the_account_in_its_place() {
        let mut accounts = Accounts::default();
        accounts.add(signed_in("1", "me@gmail.com"));
        accounts.add(signed_in("2", "me@work.example.com"));
        accounts.apply(&[("1".into(), Check::SignInNeeded)]);

        assert!(!accounts.add(signed_in("1", "me@gmail.com")));

        assert_eq!(
            listed(&accounts),
            [
                ("me@gmail.com", AccountState::Connected),
                ("me@work.example.com", AccountState::Connected),
            ]
        );
    }

    #[test]
    fn a_failed_check_is_retried_soon_and_a_locked_store_waits_for_the_user() {
        let works = Ok(vec![("1".to_string(), Check::Works)]);
        let offline = Ok(vec![
            ("1".to_string(), Check::Works),
            ("2".to_string(), Check::Failed("offline".into())),
        ]);
        let locked = Err(CheckError::SecretStore("unlocking it was declined".into()));
        assert_eq!(next_check(&works), Some(CHECK_EVERY));
        assert_eq!(next_check(&offline), Some(RETRY_AFTER));
        assert_eq!(next_check(&locked), None);
    }

    #[test]
    fn account_problems_explain_themselves() {
        assert!(needs_sign_in("me@gmail.com").starts_with("me@gmail.com needs sign-in"));
        let locked = CheckError::SecretStore("unlocking it was declined".into()).message();
        assert!(locked.contains("locked") && locked.contains("nothing synchronizes"));
        let path = PathBuf::from("/home/u/.config/calendar-buddy/google-oauth-client.json");
        assert!(
            CheckError::ClientFile(path)
                .message()
                .contains("/home/u/.config/calendar-buddy/google-oauth-client.json")
        );
    }

    #[test]
    fn each_state_has_its_label() {
        assert_eq!(AccountState::Connected.label(), "Connected");
        assert_eq!(AccountState::NeedsSignIn.label(), "Needs sign-in");
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
