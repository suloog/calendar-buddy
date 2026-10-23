//! Signing in to Google (`MOD-01`): the OAuth 2.0 flow for installed apps with PKCE (S256), the
//! system browser and a loopback redirect to `127.0.0.1` on a random port.

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use calendar_core::accounts::{Account, ConnectError};
use calendar_core::platform::{Browser, SecretStore, SecretStoreError};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use url::Url;

#[cfg(test)]
mod fake;

/// The client file's name in the app's config directory (`DEC-77`).
pub const CLIENT_FILE: &str = "google-oauth-client.json";

const CALENDAR: &str = "https://www.googleapis.com/auth/calendar";
const CONTACTS: &str = "https://www.googleapis.com/auth/contacts.readonly";
const OTHER_CONTACTS: &str = "https://www.googleapis.com/auth/contacts.other.readonly";
const DIRECTORY: &str = "https://www.googleapis.com/auth/directory.readonly";
/// What every account is asked for; the directory is asked afterwards, of Workspace accounts only.
const SCOPES: [&str; 6] = [
    "openid",
    "email",
    "profile",
    CALENDAR,
    CONTACTS,
    OTHER_CONTACTS,
];

/// How long the app waits for the browser to come back before it stops listening.
pub const SIGN_IN_WAIT: Duration = Duration::from_secs(5 * 60);
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const NETWORK_CHECK: Duration = Duration::from_secs(5);

/// Where the sign-in talks to; tests point it at a fake Google.
#[derive(Clone, Debug)]
pub struct Endpoints {
    pub auth: String,
    pub token: String,
    pub userinfo: String,
}

impl Endpoints {
    pub fn google() -> Self {
        Endpoints {
            auth: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
            userinfo: "https://openidconnect.googleapis.com/v1/userinfo".into(),
        }
    }
}

/// The OAuth client, from the file downloaded from Google Cloud Console (`DEC-77`); a desktop
/// client's secret is not confidential (`DEC-10`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub id: String,
    pub secret: String,
}

impl Client {
    pub fn read(path: &Path) -> Result<Client, ConnectError> {
        #[derive(Deserialize)]
        struct File {
            installed: Installed,
        }
        #[derive(Deserialize)]
        struct Installed {
            client_id: String,
            client_secret: String,
        }
        let unreadable = || ConnectError::ClientFile(path.to_path_buf());
        let text = std::fs::read_to_string(path).map_err(|_| unreadable())?;
        let file: File = serde_json::from_str(&text).map_err(|_| unreadable())?;
        Ok(Client {
            id: file.installed.client_id,
            secret: file.installed.client_secret,
        })
    }
}

/// `$XDG_CONFIG_HOME/calendar-buddy/google-oauth-client.json` (default `~/.config/…`).
pub fn client_file_path() -> PathBuf {
    calendar_core::settings::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config/calendar-buddy"))
        .join(CLIENT_FILE)
}

/// What connecting checks before the browser opens: the client file, a running secret store and
/// the network.
pub fn prepare(
    client_file: &Path,
    store: &dyn SecretStore,
    endpoints: &Endpoints,
) -> Result<Client, ConnectError> {
    let client = Client::read(client_file)?;
    store.check().map_err(store_error)?;
    if !reachable(&endpoints.token) {
        return Err(ConnectError::Offline);
    }
    Ok(client)
}

/// Signs in to one account in the browser, keeps its refresh token in the secret store and
/// returns it. Workspace accounts are then asked for the directory, so personal accounts never
/// see that permission (`MOD-01-F03`).
pub fn connect(
    client: &Client,
    endpoints: &Endpoints,
    browser: &dyn Browser,
    store: &dyn SecretStore,
    cancel: &AtomicBool,
    wait: Duration,
) -> Result<Account, ConnectError> {
    let first = authorize(client, endpoints, browser, &SCOPES, None, cancel, wait)?;
    let mut granted = first.scope.clone();
    if !has(&granted, CALENDAR) {
        return Err(ConnectError::CalendarNotGranted);
    }
    let user = user_info(endpoints, &first.access_token)?;
    let mut refresh_token = first.refresh_token;
    let workspace = user.hd.is_some();
    if workspace {
        let hint = Some(user.email.as_str());
        match authorize(client, endpoints, browser, &[DIRECTORY], hint, cancel, wait) {
            // `login_hint` only pre-selects the account: another one could have been picked.
            Ok(more)
                if user_info(endpoints, &more.access_token).is_ok_and(|u| u.sub == user.sub) =>
            {
                granted = format!("{granted} {}", more.scope);
                // The new token covers both grants (`include_granted_scopes`).
                refresh_token = more.refresh_token.or(refresh_token);
            }
            Err(_) if cancel.load(Ordering::Relaxed) => return Err(ConnectError::Cancelled),
            // Connected without the directory; colleague suggestions stay unavailable (`DEC-52`).
            _ => {}
        }
    }
    let refresh_token =
        refresh_token.ok_or_else(|| ConnectError::Failed("Google sent no refresh token".into()))?;
    let account = Account {
        id: user.sub,
        email: user.email,
        workspace,
        contacts: has(&granted, CONTACTS) && has(&granted, OTHER_CONTACTS),
        directory: has(&granted, DIRECTORY),
    };
    if cancel.load(Ordering::Relaxed) {
        return Err(ConnectError::Cancelled);
    }
    store.save(&account, &refresh_token).map_err(store_error)?;
    Ok(account)
}

fn has(granted: &str, scope: &str) -> bool {
    granted.split_whitespace().any(|g| g == scope)
}

fn store_error(error: SecretStoreError) -> ConnectError {
    match error {
        SecretStoreError::NotRunning => ConnectError::NoSecretStore,
        SecretStoreError::Failed(reason) => ConnectError::SecretStore(reason),
    }
}

fn failed(error: std::io::Error) -> ConnectError {
    ConnectError::Failed(error.to_string())
}

/// Whether Google's token service answers at all, through a proxy if the system has one; no
/// network means no sign-in.
fn reachable(endpoint: &str) -> bool {
    agent(NETWORK_CHECK).get(endpoint).call().is_ok()
}

fn user_info(endpoints: &Endpoints, access_token: &str) -> Result<User, ConnectError> {
    json(
        agent(HTTP_TIMEOUT)
            .get(&endpoints.userinfo)
            .header("Authorization", format!("Bearer {access_token}"))
            .call(),
    )
}

/// Google's token answer; not `Debug`, so the tokens never reach a log (`NFR-07`).
#[derive(Deserialize)]
struct Tokens {
    access_token: String,
    refresh_token: Option<String>,
    #[serde(default)]
    scope: String,
}

#[derive(Deserialize)]
struct User {
    sub: String,
    email: String,
    /// The Workspace domain; absent on personal accounts.
    hd: Option<String>,
}

/// One trip through the browser: Google's consent screen for `scopes`, the code back on the
/// loopback port, and the code exchanged for tokens.
fn authorize(
    client: &Client,
    endpoints: &Endpoints,
    browser: &dyn Browser,
    scopes: &[&str],
    login_hint: Option<&str>,
    cancel: &AtomicBool,
    wait: Duration,
) -> Result<Tokens, ConnectError> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(failed)?;
    let redirect = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().map_err(failed)?.port()
    );
    let verifier = random();
    let state = random();
    let url = authorization_url(
        client,
        endpoints,
        &redirect,
        scopes,
        &challenge(&verifier),
        &state,
        login_hint,
    );
    browser
        .open(url.as_str())
        .map_err(|_| ConnectError::Failed("the browser could not be opened".into()))?;
    let code = wait_for_code(&listener, &state, cancel, wait)?;
    json(agent(HTTP_TIMEOUT).post(&endpoints.token).send_form([
        ("code", code.as_str()),
        ("client_id", &client.id),
        ("client_secret", &client.secret),
        ("redirect_uri", &redirect),
        ("grant_type", "authorization_code"),
        ("code_verifier", &verifier),
    ]))
}

fn authorization_url(
    client: &Client,
    endpoints: &Endpoints,
    redirect: &str,
    scopes: &[&str],
    challenge: &str,
    state: &str,
    login_hint: Option<&str>,
) -> Url {
    let mut url = Url::parse(&endpoints.auth).expect("a valid authorization endpoint");
    url.query_pairs_mut()
        .append_pair("client_id", &client.id)
        .append_pair("redirect_uri", redirect)
        .append_pair("response_type", "code")
        .append_pair("scope", &scopes.join(" "))
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state)
        // A refresh token every time, so the account stays signed in (`MOD-01-F04`).
        .append_pair("access_type", "offline");
    match login_hint {
        Some(hint) => url
            .query_pairs_mut()
            .append_pair("prompt", "consent")
            .append_pair("login_hint", hint)
            .append_pair("include_granted_scopes", "true"),
        // Google's account chooser, so a second account can be picked in a browser already
        // signed in to the first (`MOD-01-F02`).
        None => url
            .query_pairs_mut()
            .append_pair("prompt", "select_account consent"),
    };
    url
}

/// 32 random bytes, base64url: a PKCE code verifier or a `state`.
fn random() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the system's random source");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// The PKCE S256 challenge of a verifier.
fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Listens on the loopback port until the browser brings this sign-in's answer, the user
/// cancels in the app, or the wait runs out.
fn wait_for_code(
    listener: &TcpListener,
    state: &str,
    cancel: &AtomicBool,
    wait: Duration,
) -> Result<String, ConnectError> {
    listener.set_nonblocking(true).map_err(failed)?;
    let deadline = Instant::now() + wait;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(ConnectError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(ConnectError::TimedOut);
        }
        match listener.accept() {
            Ok((stream, _)) => {
                if let Some(answer) = answer(stream, state) {
                    return answer;
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => return Err(failed(error)),
        }
    }
}

const DONE_PAGE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
     Connection: close\r\n\r\n<!doctype html><title>Calendar Buddy</title>\
     <p>You can close this tab and go back to Calendar Buddy.</p>";
const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";

/// Reads one request to the loopback port; `None` when it is not this sign-in's redirect (a
/// favicon, a stray visit).
fn answer(mut stream: TcpStream, state: &str) -> Option<Result<String, ConnectError>> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    while !request.windows(4).any(|w| w == b"\r\n\r\n") && request.len() < 16 * 1024 {
        let read = stream.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
    }
    let request = String::from_utf8_lossy(&request);
    let target = request.lines().next()?.split_whitespace().nth(1)?;
    let url = Url::parse(&format!("http://127.0.0.1{target}")).ok()?;
    let param = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    let result = if param("state").as_deref() != Some(state) {
        None
    } else if let Some(code) = param("code") {
        Some(Ok(code))
    } else {
        param("error").map(|error| Err(redirect_error(error, param("error_description"))))
    };
    let page = if result.is_some() {
        DONE_PAGE
    } else {
        NOT_FOUND
    };
    let _ = stream.write_all(page.as_bytes());
    result
}

/// `access_denied` is the user cancelling on Google's screen; anything else is Google's reason.
fn redirect_error(error: String, description: Option<String>) -> ConnectError {
    match (error.as_str(), description) {
        ("access_denied", _) => ConnectError::Cancelled,
        (_, Some(description)) => ConnectError::Refused(format!("{error} ({description})")),
        (_, None) => ConnectError::Refused(error),
    }
}

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(timeout))
        .build()
        .into()
}

/// Google's JSON answer; an error answer becomes Google's reason for refusing.
fn json<T: DeserializeOwned>(
    response: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
) -> Result<T, ConnectError> {
    let mut response = response.map_err(|error| ConnectError::Failed(error.to_string()))?;
    let status = response.status();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|error| ConnectError::Failed(error.to_string()))?;
    if !status.is_success() {
        return Err(ConnectError::Refused(
            google_reason(&body).unwrap_or_else(|| status.to_string()),
        ));
    }
    serde_json::from_str(&body)
        .map_err(|_| ConnectError::Failed("Google's answer could not be read".into()))
}

/// `{"error": "…", "error_description": "…"}` from the token service, or
/// `{"error": {"message": "…"}}` from Google's APIs.
fn google_reason(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let error = &value["error"];
    match (error.as_str(), value["error_description"].as_str()) {
        (Some(error), Some(description)) => Some(format!("{error} ({description})")),
        (Some(error), None) => Some(error.to_string()),
        _ => error["message"].as_str().map(str::to_string),
    }
}

#[cfg(test)]
mod tests;
