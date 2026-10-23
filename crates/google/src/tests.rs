use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use super::*;
use calendar_core::accounts::{AccountState, Accounts, Check, CheckError};

use crate::fake::{
    ACCOUNT_ID, CLIENT_ID, EMAIL, FakeGoogle, FakeStore, OTHER_EMAIL, OTHER_ID, Reply, param,
};

const WAIT: Duration = Duration::from_secs(10);

fn client() -> Client {
    Client {
        id: CLIENT_ID.into(),
        secret: "test-secret".into(),
    }
}

fn connect_to(google: &FakeGoogle, store: &FakeStore) -> Result<Account, ConnectError> {
    let cancel = AtomicBool::new(false);
    connect(
        &client(),
        &google.endpoints,
        google,
        store,
        None,
        &cancel,
        WAIT,
    )
}

/// A client file pointing at the test client, in a directory of the test's own.
fn client_file(name: &str) -> (PathBuf, PathBuf) {
    let dir = temp_dir(name);
    let path = dir.join(CLIENT_FILE);
    std::fs::write(
        &path,
        format!(r#"{{"installed":{{"client_id":"{CLIENT_ID}","client_secret":"test-secret"}}}}"#),
    )
    .unwrap();
    (dir, path)
}

/// A personal and a Workspace account, both connected.
fn two_accounts(google: &FakeGoogle, store: &FakeStore) -> Accounts {
    let mut accounts = Accounts::default();
    accounts.add(connect_to(google, store).unwrap());
    accounts.add(connect_to(google, store).unwrap());
    accounts
}

fn states(accounts: &Accounts) -> Vec<(&str, AccountState)> {
    accounts
        .list()
        .iter()
        .map(|e| (e.account.email.as_str(), e.state))
        .collect()
}

fn scopes(url: &Url) -> Vec<String> {
    param(url, "scope")
        .unwrap()
        .split(' ')
        .map(str::to_string)
        .collect()
}

/// A temporary directory for one test's files.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "calendar-buddy-google-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_pkce_challenge_is_s256_of_the_verifier() {
    // base64url(SHA-256(verifier)) without padding, computed independently with Python's hashlib.
    assert_eq!(
        challenge("dBjftJeZ4CVP-mJ92K1jtYctgdhgRPnpTyUyX4KfiG0"),
        "AW1ZxLZh-fO3E6MAgh001kPmVqtAwKkoRqzQxMJJSw0"
    );
    assert_eq!(random().len(), 43);
    assert_ne!(random(), random());
}

#[test]
fn the_authorization_request_asks_for_calendar_and_contacts_with_pkce() {
    let url = authorization_url(
        &client(),
        &Endpoints::google(),
        "http://127.0.0.1:4711",
        &SCOPES,
        "the-challenge",
        "the-state",
        None,
    );
    assert_eq!(url.host_str(), Some("accounts.google.com"));
    assert_eq!(param(&url, "client_id").as_deref(), Some(CLIENT_ID));
    assert_eq!(
        param(&url, "redirect_uri").as_deref(),
        Some("http://127.0.0.1:4711")
    );
    assert_eq!(param(&url, "response_type").as_deref(), Some("code"));
    assert_eq!(
        param(&url, "code_challenge").as_deref(),
        Some("the-challenge")
    );
    assert_eq!(
        param(&url, "code_challenge_method").as_deref(),
        Some("S256")
    );
    assert_eq!(param(&url, "state").as_deref(), Some("the-state"));
    assert_eq!(
        scopes(&url),
        [
            "openid",
            "email",
            "profile",
            "https://www.googleapis.com/auth/calendar",
            "https://www.googleapis.com/auth/contacts.readonly",
            "https://www.googleapis.com/auth/contacts.other.readonly",
        ]
    );
    assert_eq!(param(&url, "login_hint"), None);
    // Google's account chooser, so another account than the browser's current one can be picked.
    assert_eq!(
        param(&url, "prompt").as_deref(),
        Some("select_account consent")
    );
}

#[test]
fn the_client_file_is_read_as_downloaded_from_google_cloud_console() {
    let dir = temp_dir("client-file");
    let path = dir.join(CLIENT_FILE);
    std::fs::write(
        &path,
        r#"{"installed":{"client_id":"test-client.apps.googleusercontent.com",
        "project_id":"calendar-buddy","auth_uri":"https://accounts.google.com/o/oauth2/auth",
        "token_uri":"https://oauth2.googleapis.com/token","client_secret":"test-secret",
        "redirect_uris":["http://localhost"]}}"#,
    )
    .unwrap();
    let read = Client::read(&path);
    std::fs::write(&path, "{ not json").unwrap();
    let unreadable = Client::read(&path);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(read, Ok(client()));
    assert_eq!(unreadable, Err(ConnectError::ClientFile(path.clone())));
    assert_eq!(
        Client::read(&dir.join("missing.json")),
        Err(ConnectError::ClientFile(dir.join("missing.json")))
    );
}

#[test]
fn the_client_file_lives_in_the_config_directory() {
    assert!(
        client_file_path().ends_with("calendar-buddy/google-oauth-client.json"),
        "{}",
        client_file_path().display()
    );
}

#[test]
fn a_missing_client_file_stops_connecting_before_anything_else() {
    let google = FakeGoogle::start(None, &[]);
    let missing = PathBuf::from("/nonexistent/calendar-buddy/google-oauth-client.json");
    let store = FakeStore {
        not_running: true,
        ..FakeStore::default()
    };
    assert_eq!(
        prepare(&missing, &store, &google.endpoints),
        Err(ConnectError::ClientFile(missing))
    );
    assert!(google.opened().is_empty());
}

#[test]
fn without_a_secret_store_connecting_stops_before_the_browser() {
    let dir = temp_dir("no-store");
    let path = dir.join(CLIENT_FILE);
    std::fs::write(
        &path,
        format!(r#"{{"installed":{{"client_id":"{CLIENT_ID}","client_secret":"test-secret"}}}}"#),
    )
    .unwrap();
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore {
        not_running: true,
        ..FakeStore::default()
    };
    let result = prepare(&path, &store, &google.endpoints);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(result, Err(ConnectError::NoSecretStore));
    assert!(google.opened().is_empty());
}

#[test]
fn offline_signing_in_is_refused_and_works_again_once_the_network_is_back() {
    let dir = temp_dir("offline");
    let path = dir.join(CLIENT_FILE);
    std::fs::write(
        &path,
        format!(r#"{{"installed":{{"client_id":"{CLIENT_ID}","client_secret":"test-secret"}}}}"#),
    )
    .unwrap();
    // Google's port, with nothing answering on it.
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let endpoints = Endpoints {
        auth: format!("http://127.0.0.1:{port}/auth"),
        token: format!("http://127.0.0.1:{port}/token"),
        userinfo: format!("http://127.0.0.1:{port}/userinfo"),
        revoke: format!("http://127.0.0.1:{port}/revoke"),
    };
    let store = FakeStore::default();
    let offline = prepare(&path, &store, &endpoints);
    // The network is back: Google answers, even if only to say the request is wrong.
    let google = TcpListener::bind(("127.0.0.1", port)).unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = google.accept().unwrap();
        let _ = stream.read(&mut [0; 1024]);
        let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n");
    });
    let again = prepare(&path, &store, &endpoints);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(offline, Err(ConnectError::Offline));
    assert_eq!(again, Ok(client()));
}

#[test]
fn a_personal_account_is_connected_and_its_refresh_token_kept_in_the_secret_store() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    let store = FakeStore::default();

    let account = connect_to(&google, &store).unwrap();

    assert_eq!(
        account,
        Account {
            id: ACCOUNT_ID.into(),
            email: EMAIL.into(),
            workspace: false,
            contacts: true,
            directory: false,
        }
    );
    assert!(account.unavailable().is_empty());
    assert_eq!(
        *store.saved.lock().unwrap(),
        [(ACCOUNT_ID.to_string(), "refresh-code-1".to_string())]
    );
    // One consent screen, without the directory, and the redirect to the loopback address.
    let opened = google.opened();
    assert_eq!(opened.len(), 1);
    assert!(!scopes(&opened[0]).iter().any(|s| s.contains("directory")));
    assert!(
        param(&opened[0], "redirect_uri")
            .unwrap()
            .starts_with("http://127.0.0.1:")
    );
}

#[test]
fn a_workspace_account_is_asked_for_the_directory_on_its_own() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[Reply::Consent(&[]), Reply::Consent(&[])],
    );
    let store = FakeStore::default();

    let account = connect_to(&google, &store).unwrap();

    assert!(account.workspace && account.contacts && account.directory);
    let opened = google.opened();
    assert_eq!(opened.len(), 2);
    assert_eq!(scopes(&opened[1]), [DIRECTORY]);
    assert_eq!(param(&opened[1], "login_hint").as_deref(), Some(EMAIL));
    assert_eq!(
        param(&opened[1], "include_granted_scopes").as_deref(),
        Some("true")
    );
    // The account is already picked; no chooser on the second screen.
    assert_eq!(param(&opened[1], "prompt").as_deref(), Some("consent"));
    // The second token covers both grants.
    assert_eq!(
        *store.saved.lock().unwrap(),
        [(ACCOUNT_ID.to_string(), "refresh-code-2".to_string())]
    );
}

#[test]
fn unticked_contacts_or_directory_still_connect_the_account_without_those_features() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[
            Reply::Consent(&[OTHER_CONTACTS]),
            Reply::Consent(&[DIRECTORY]),
        ],
    );
    let account = connect_to(&google, &FakeStore::default()).unwrap();
    assert!(!account.contacts && !account.directory);
    assert_eq!(account.unavailable().len(), 2);

    // Cancelling the second screen also leaves the account connected without the directory.
    let google = FakeGoogle::start(
        Some("example.com"),
        &[Reply::Consent(&[]), Reply::Error("access_denied")],
    );
    let store = FakeStore::default();
    let account = connect_to(&google, &store).unwrap();
    assert!(account.contacts && !account.directory);
    assert_eq!(
        *store.saved.lock().unwrap(),
        [(ACCOUNT_ID.to_string(), "refresh-code-1".to_string())]
    );
}

#[test]
fn another_account_picked_on_the_directory_screen_is_not_mixed_in() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[Reply::Consent(&[]), Reply::OtherAccount],
    );
    let store = FakeStore::default();
    let account = connect_to(&google, &store).unwrap();
    assert!(account.contacts && !account.directory);
    assert_eq!(account.id, ACCOUNT_ID);
    // The first account's own token, not the other account's directory-only one.
    assert_eq!(
        *store.saved.lock().unwrap(),
        [(ACCOUNT_ID.to_string(), "refresh-code-1".to_string())]
    );
}

#[test]
fn a_personal_account_is_connected_beside_a_workspace_one() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[
            Reply::Consent(&[]),
            Reply::Consent(&[]),
            Reply::OtherAccount,
        ],
    );
    let store = FakeStore::default();
    let mut accounts = Accounts::default();

    assert!(accounts.add(connect_to(&google, &store).unwrap()));
    // The second account is picked in Google's account chooser.
    assert!(accounts.add(connect_to(&google, &store).unwrap()));

    let listed: Vec<_> = accounts
        .list()
        .iter()
        .map(|e| (e.account.email.as_str(), e.account.workspace, e.state))
        .collect();
    assert_eq!(
        listed,
        [
            (EMAIL, true, AccountState::Connected),
            (OTHER_EMAIL, false, AccountState::Connected),
        ]
    );
    // Each account keeps its own refresh token.
    assert_eq!(
        *store.saved.lock().unwrap(),
        [
            (ACCOUNT_ID.to_string(), "refresh-code-2".to_string()),
            (OTHER_ID.to_string(), "refresh-code-3".to_string()),
        ]
    );
}

#[test]
fn signing_in_to_a_connected_account_again_keeps_one_account() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[]), Reply::Consent(&[])]);
    let store = FakeStore::default();
    let mut accounts = Accounts::default();

    assert!(accounts.add(connect_to(&google, &store).unwrap()));
    assert!(!accounts.add(connect_to(&google, &store).unwrap()));

    assert_eq!(accounts.list().len(), 1);
    // Saved again under the same account ID.
    assert_eq!(
        *store.saved.lock().unwrap(),
        [(ACCOUNT_ID.to_string(), "refresh-code-2".to_string())]
    );
}

#[test]
fn unticked_calendar_access_does_not_connect_the_account() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[CALENDAR])]);
    let store = FakeStore::default();
    assert_eq!(
        connect_to(&google, &store),
        Err(ConnectError::CalendarNotGranted)
    );
    assert!(store.saved.lock().unwrap().is_empty());
}

#[test]
fn cancelling_on_googles_screen_adds_no_account() {
    let google = FakeGoogle::start(None, &[Reply::Error("access_denied")]);
    let store = FakeStore::default();
    assert_eq!(connect_to(&google, &store), Err(ConnectError::Cancelled));
    assert!(store.saved.lock().unwrap().is_empty());
}

#[test]
fn a_closed_browser_ends_in_a_time_out_or_the_users_cancel() {
    let google = FakeGoogle::start(None, &[Reply::Nothing, Reply::Nothing]);
    let store = FakeStore::default();
    let cancel = AtomicBool::new(false);
    let short = Duration::from_millis(300);
    assert_eq!(
        connect(
            &client(),
            &google.endpoints,
            &google,
            &store,
            None,
            &cancel,
            short
        ),
        Err(ConnectError::TimedOut)
    );
    let cancel = AtomicBool::new(true);
    assert_eq!(
        connect(
            &client(),
            &google.endpoints,
            &google,
            &store,
            None,
            &cancel,
            WAIT
        ),
        Err(ConnectError::Cancelled)
    );
    assert!(store.saved.lock().unwrap().is_empty());
}

#[test]
fn googles_reason_is_shown_when_it_refuses_the_account() {
    let google = FakeGoogle::start(None, &[Reply::Error("admin_policy_enforced")]);
    assert_eq!(
        connect_to(&google, &FakeStore::default()),
        Err(ConnectError::Refused("admin_policy_enforced".into()))
    );

    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    google.refuse_tokens("org_internal");
    let store = FakeStore::default();
    assert_eq!(
        connect_to(&google, &store),
        Err(ConnectError::Refused(
            "org_internal (Refused by the fake)".into()
        ))
    );
    assert!(store.saved.lock().unwrap().is_empty());
}

#[test]
fn a_secret_store_that_does_not_save_leaves_the_account_unconnected() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    let store = FakeStore {
        locked: true,
        ..FakeStore::default()
    };
    assert_eq!(
        connect_to(&google, &store),
        Err(ConnectError::SecretStore("the collection is locked".into()))
    );
}

#[test]
fn a_stray_request_to_the_loopback_port_is_ignored() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for query in [
            "GET /favicon.ico",
            "GET /?code=x&state=other",
            "GET /?code=c&state=s",
        ] {
            let mut stream = TcpStream::connect(address).unwrap();
            write!(stream, "{query} HTTP/1.1\r\n\r\n").unwrap();
            let _ = stream.read_to_end(&mut Vec::new());
        }
    });
    assert_eq!(
        wait_for_code(&listener, "s", &AtomicBool::new(false), WAIT),
        Ok("c".to_string())
    );
}

#[test]
fn tc_01_03_connected_accounts_stay_connected_after_a_restart() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[
            Reply::Consent(&[]),
            Reply::Consent(&[]),
            Reply::OtherAccount,
        ],
    );
    let store = FakeStore::default();
    let (dir, client_file) = client_file("tc-01-03");
    let remembered = dir.join("accounts");
    two_accounts(&google, &store).save(&remembered).unwrap();
    let signed_in = google.opened().len();

    // The app starts again: the accounts are read back and Google is asked about each.
    let mut accounts = Accounts::load(&remembered);
    let checked = check(
        &client_file,
        &google.endpoints,
        &store,
        &accounts.connected(),
    );
    std::fs::remove_dir_all(&dir).unwrap();

    let checked = checked.unwrap();
    assert_eq!(
        checked,
        [
            (ACCOUNT_ID.to_string(), Check::Works),
            (OTHER_ID.to_string(), Check::Works),
        ]
    );
    assert!(!accounts.apply(&checked));
    assert_eq!(
        states(&accounts),
        [
            (EMAIL, AccountState::Connected),
            (OTHER_EMAIL, AccountState::Connected),
        ]
    );
    // No browser sign-in: the refresh tokens from the secret store were enough.
    assert_eq!(google.opened().len(), signed_in);
    assert_eq!(google.refreshed(), ["refresh-code-2", "refresh-code-3"]);
}

#[test]
fn revoked_access_marks_only_that_account_and_signing_in_again_restores_it() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[
            Reply::Consent(&[]),
            Reply::Consent(&[]),
            Reply::OtherAccount,
            Reply::Consent(&[]),
            Reply::Consent(&[]),
        ],
    );
    let store = FakeStore::default();
    let (dir, client_file) = client_file("revoked");
    let mut accounts = two_accounts(&google, &store);

    google.revoke(ACCOUNT_ID);
    let checked = check(
        &client_file,
        &google.endpoints,
        &store,
        &accounts.connected(),
    )
    .unwrap();
    assert_eq!(
        checked,
        [
            (ACCOUNT_ID.to_string(), Check::SignInNeeded),
            (OTHER_ID.to_string(), Check::Works),
        ]
    );
    assert!(accounts.apply(&checked));
    assert_eq!(
        states(&accounts),
        [
            (EMAIL, AccountState::NeedsSignIn),
            (OTHER_EMAIL, AccountState::Connected),
        ]
    );

    // "Sign in again" asks Google for that account, which is restored in its place.
    let cancel = AtomicBool::new(false);
    let again = connect(
        &client(),
        &google.endpoints,
        &google,
        &store,
        Some(EMAIL),
        &cancel,
        WAIT,
    )
    .unwrap();
    let first = &google.opened()[3];
    assert_eq!(param(first, "login_hint").as_deref(), Some(EMAIL));
    assert_eq!(param(first, "prompt").as_deref(), Some("consent"));
    assert!(!accounts.add(again));
    assert_eq!(
        states(&accounts),
        [
            (EMAIL, AccountState::Connected),
            (OTHER_EMAIL, AccountState::Connected),
        ]
    );
    let checked = check(
        &client_file,
        &google.endpoints,
        &store,
        &accounts.connected(),
    );
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        checked
            .unwrap()
            .iter()
            .all(|(_, check)| *check == Check::Works)
    );
}

#[test]
fn an_account_without_a_sign_in_in_the_secret_store_needs_sign_in() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("no-token");
    let account = connect_to(&google, &store).unwrap();
    store.saved.lock().unwrap().clear();

    let checked = check(&client_file, &google.endpoints, &store, &[account]);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(
        checked,
        Ok(vec![(ACCOUNT_ID.to_string(), Check::SignInNeeded)])
    );
    assert!(google.refreshed().is_empty());
}

#[test]
fn offline_the_accounts_stay_connected() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("offline-check");
    let mut accounts = Accounts::default();
    accounts.add(connect_to(&google, &store).unwrap());
    // Google's port, with nothing answering on it.
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let offline = Endpoints {
        token: format!("http://127.0.0.1:{port}/token"),
        ..google.endpoints.clone()
    };

    let checked = check(&client_file, &offline, &store, &accounts.connected());
    std::fs::remove_dir_all(&dir).unwrap();

    let checked = checked.unwrap();
    assert!(matches!(checked[0].1, Check::Failed(_)));
    assert!(!accounts.apply(&checked));
    assert_eq!(states(&accounts), [(EMAIL, AccountState::Connected)]);
}

#[test]
fn a_locked_secret_store_leaves_every_account_as_it_was() {
    let google = FakeGoogle::start(None, &[]);
    let (dir, client_file) = client_file("locked");
    let account = Account {
        id: ACCOUNT_ID.into(),
        email: EMAIL.into(),
        workspace: false,
        contacts: true,
        directory: false,
    };
    let locked = FakeStore {
        locked: true,
        ..FakeStore::default()
    };
    let not_running = FakeStore {
        not_running: true,
        ..FakeStore::default()
    };

    let declined = check(
        &client_file,
        &google.endpoints,
        &locked,
        std::slice::from_ref(&account),
    );
    let none = check(
        &client_file,
        &google.endpoints,
        &not_running,
        std::slice::from_ref(&account),
    );
    let missing = dir.join("missing.json");
    let no_client = check(&missing, &google.endpoints, &locked, &[account]);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(
        declined,
        Err(CheckError::SecretStore("unlocking it was declined".into()))
    );
    assert_eq!(none, Err(CheckError::NoSecretStore));
    assert_eq!(no_client, Err(CheckError::ClientFile(missing)));
    assert!(google.refreshed().is_empty());
}

#[test]
fn tc_01_04_removing_an_account_deletes_its_local_data_and_withdraws_access() {
    let google = FakeGoogle::start(
        Some("example.com"),
        &[
            Reply::Consent(&[]),
            Reply::Consent(&[]),
            Reply::OtherAccount,
        ],
    );
    let store = FakeStore::default();
    let (dir, client_file) = client_file("tc-01-04");
    let remembered = dir.join("accounts");
    let mut accounts = two_accounts(&google, &store);
    accounts.save(&remembered).unwrap();
    assert!(google.has_access(ACCOUNT_ID));
    let asked = google.requests().len();

    // Remove and confirm: the credentials go from this computer first, then the app's access is
    // withdrawn at Google.
    let token = store.take(ACCOUNT_ID).unwrap();
    assert!(accounts.remove(ACCOUNT_ID));
    accounts.save(&remembered).unwrap();
    let withdrawn = revoke(&google.endpoints, &token.unwrap());
    let received = google.requests()[asked..].to_vec();

    let kept = std::fs::read_to_string(&remembered).unwrap();
    let others = check(
        &client_file,
        &google.endpoints,
        &store,
        &accounts.connected(),
    );
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(!kept.contains(ACCOUNT_ID) && !kept.contains(EMAIL));
    assert_eq!(states(&accounts), [(OTHER_EMAIL, AccountState::Connected)]);
    let saved = store.saved.lock().unwrap();
    assert_eq!(
        saved.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
        [OTHER_ID]
    );
    assert!(withdrawn);
    assert!(!google.has_access(ACCOUNT_ID));
    // Google was asked only to withdraw the access: nothing in the account itself changed.
    assert_eq!(received, ["POST /revoke"]);
    // The other account keeps working.
    assert!(google.has_access(OTHER_ID));
    assert_eq!(others, Ok(vec![(OTHER_ID.to_string(), Check::Works)]));
}

#[test]
fn without_network_removing_still_deletes_everything_locally() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    let store = FakeStore::default();
    connect_to(&google, &store).unwrap();
    // Google's port, with nothing answering on it.
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let offline = Endpoints {
        revoke: format!("http://127.0.0.1:{port}/revoke"),
        ..google.endpoints.clone()
    };

    let token = store.take(ACCOUNT_ID).unwrap();
    assert!(store.saved.lock().unwrap().is_empty());
    assert!(!revoke(&offline, &token.unwrap()));
}

#[test]
fn a_locked_secret_store_keeps_the_credentials_of_an_account_not_removed() {
    let google = FakeGoogle::start(None, &[Reply::Consent(&[])]);
    let mut store = FakeStore::default();
    connect_to(&google, &store).unwrap();
    store.locked = true;

    assert_eq!(
        store.take(ACCOUNT_ID),
        Err(SecretStoreError::Failed("unlocking it was declined".into()))
    );
    assert_eq!(store.saved.lock().unwrap().len(), 1);
    let none = FakeStore {
        not_running: true,
        ..FakeStore::default()
    };
    assert_eq!(none.take(ACCOUNT_ID), Err(SecretStoreError::NotRunning));
}

#[test]
fn an_account_without_credentials_is_removed_with_nothing_to_withdraw() {
    let store = FakeStore::default();
    assert_eq!(store.take(ACCOUNT_ID), Ok(None));
}
