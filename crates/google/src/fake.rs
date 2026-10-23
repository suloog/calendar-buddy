//! Fakes for the sign-in tests: Google (its consent screen, as seen through the browser, its token
//! service and user info) and the secret store. Nothing here leaves `127.0.0.1`.

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use calendar_core::accounts::Account;
use calendar_core::platform::{Browser, SecretStore, SecretStoreError};
use url::Url;

use crate::{Endpoints, challenge};

pub const CLIENT_ID: &str = "test-client.apps.googleusercontent.com";
pub const ACCOUNT_ID: &str = "107691503500061507151";
pub const EMAIL: &str = "owner@example.com";
/// Another account signed in to the same browser.
const OTHER_ID: &str = "100000000000000000001";

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
}

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
    let (status, json) = if head.starts_with("POST /token") {
        let form: HashMap<String, String> = url::form_urlencoded::parse(body.as_bytes())
            .into_owned()
            .collect();
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
        let hd = state
            .hd
            .map(|hd| format!(r#", "hd": "{hd}""#))
            .unwrap_or_default();
        (
            "200 OK",
            format!(r#"{{"sub": "{sub}", "email": "{EMAIL}", "name": "Owner"{hd}}}"#),
        )
    } else {
        (
            "401 Unauthorized",
            r#"{"error": {"code": 401, "message": "Invalid Credentials"}}"#.to_string(),
        )
    };
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
}
