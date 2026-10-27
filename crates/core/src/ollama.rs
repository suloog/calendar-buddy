//! Events from a description (`MOD-10`): whether the function is on, where Ollama runs and which
//! model it uses, as the settings keep them (`MOD-10-F19`, `F20`, `U05`), and whether Ollama
//! answers there and has the model (`MOD-10-F21`, `U04`). The only traffic this adds goes to the
//! address set (`NFR-06`, `MOD-10-NFR-02`).

use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use url::{Host, Url};

use crate::texts;

/// Ollama on this computer, on its own port.
pub const DEFAULT_ADDRESS: &str = "http://localhost:11434";
/// The model that did best in the tests (`SRC-08-21`).
pub const DEFAULT_MODEL: &str = "gemma4:e4b";

/// How long a check waits for Ollama; it runs on this computer by default.
const TIMEOUT: Duration = Duration::from_secs(3);

/// The settings of events from a description.
#[derive(Clone, Debug, PartialEq)]
pub struct Ollama {
    /// The function is on, as by default (`MOD-10-F19`).
    pub on: bool,
    pub address: String,
    pub model: String,
}

impl Default for Ollama {
    fn default() -> Self {
        Ollama {
            on: true,
            address: DEFAULT_ADDRESS.to_string(),
            model: DEFAULT_MODEL.to_string(),
        }
    }
}

/// The address typed, when it is one Ollama can be asked at: `http` or `https` with a host; without
/// a slash at its end.
pub fn address(typed: &str) -> Option<String> {
    let typed = typed.trim();
    let url = Url::parse(typed).ok()?;
    let usable = matches!(url.scheme(), "http" | "https")
        && url.host().is_some()
        && url.query().is_none()
        && url.fragment().is_none();
    usable.then(|| typed.trim_end_matches('/').to_string())
}

/// Whether the address is this computer: `localhost`, `127.0.0.1` or `::1` (`MOD-10-U05`).
pub fn is_local(address: &str) -> bool {
    let Ok(url) = Url::parse(address) else {
        return false;
    };
    match url.host() {
        Some(Host::Domain(name)) => name.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip == Ipv4Addr::LOCALHOST,
        Some(Host::Ipv6(ip)) => ip == Ipv6Addr::LOCALHOST,
        None => false,
    }
}

/// Whether the description box can be used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Availability {
    /// Not asked yet: the box is shown as usable until Ollama says otherwise (`DEC-217`).
    #[default]
    Unknown,
    Available,
    /// Nothing answers at the address.
    NotRunning,
    /// Ollama answers but does not have the model.
    ModelMissing,
}

impl Availability {
    /// Why the box is unavailable, naming the address or the model (`MOD-10-U04`).
    pub fn why(self, ollama: &Ollama) -> Option<String> {
        match self {
            Availability::Unknown | Availability::Available => None,
            Availability::NotRunning => Some(texts::ollama_not_available(&ollama.address)),
            Availability::ModelMissing => Some(texts::model_not_installed(&ollama.model)),
        }
    }
}

/// Asks Ollama at the address set for its version, then for its models (`MOD-10` technical
/// notes); nothing else is sent, and only there: never through a proxy the environment names, nor
/// to where a redirect points (`MOD-10-NFR-02`). Blocks for as long as Ollama takes: never on the
/// UI thread.
pub fn check(ollama: &Ollama) -> Availability {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .timeout_global(Some(TIMEOUT))
        .build()
        .into();
    let get = |path: &str| -> Option<String> {
        let mut response = agent.get(format!("{}{path}", ollama.address)).call().ok()?;
        response.body_mut().read_to_string().ok()
    };
    if get("/api/version").is_none() {
        return Availability::NotRunning;
    }
    let Some(tags) = get("/api/tags") else {
        return Availability::NotRunning;
    };
    if has_model(&tags, &ollama.model) {
        Availability::Available
    } else {
        Availability::ModelMissing
    }
}

/// Whether Ollama's list of models holds the model; a name without a tag is Ollama's `latest`.
fn has_model(tags: &str, model: &str) -> bool {
    let Ok(tags) = serde_json::from_str::<serde_json::Value>(tags) else {
        return false;
    };
    let model = model.trim();
    let wanted = if model.contains(':') {
        model.to_string()
    } else {
        format!("{model}:latest")
    };
    (tags["models"].as_array().into_iter().flatten())
        .filter_map(|m| m["name"].as_str().or(m["model"].as_str()))
        .any(|name| name == wanted)
}

/// The settings' section being edited: what is typed in its fields until it is entered, and
/// whether the address typed was refused (`US-10-01` edge cases).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Form {
    pub address: String,
    pub model: String,
    pub wrong_address: bool,
}

impl Form {
    pub fn of(ollama: &Ollama) -> Self {
        Form {
            address: ollama.address.clone(),
            model: ollama.model.clone(),
            wrong_address: false,
        }
    }

    /// Enter in the address, or leaving it: an address replaces the one set, anything else is
    /// refused and the one set kept. Says whether the settings changed.
    pub fn enter_address(&mut self, ollama: &mut Ollama) -> bool {
        match address(&self.address) {
            Some(address) => {
                self.wrong_address = false;
                self.address = address.clone();
                let changed = ollama.address != address;
                ollama.address = address;
                changed
            }
            None => {
                self.wrong_address = true;
                false
            }
        }
    }

    /// Enter in the model, or leaving it: a name replaces the one set; an empty field gets the
    /// one set back. Says whether the settings changed.
    pub fn enter_model(&mut self, ollama: &mut Ollama) -> bool {
        let model = self.model.trim();
        if model.is_empty() {
            self.model = ollama.model.clone();
            return false;
        }
        let changed = ollama.model != model;
        ollama.model = model.to_string();
        self.model = ollama.model.clone();
        changed
    }

    /// Both fields entered, as when the settings close. Says whether the settings changed.
    pub fn enter(&mut self, ollama: &mut Ollama) -> bool {
        let address = self.enter_address(ollama);
        self.enter_model(ollama) || address
    }
}

/// The side panel's description box: what is typed, kept while Ollama goes away and comes back
/// (`MOD-10` *Constraints and edge cases*), and whether it can be used.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DescriptionBox {
    pub field: String,
    pub availability: Availability,
}

impl DescriptionBox {
    /// Why the box cannot be used now; `None` when it can.
    pub fn unavailable(&self, ollama: &Ollama) -> Option<String> {
        self.availability.why(ollama)
    }
}

/// A fake Ollama for the tests: on `127.0.0.1`, with the models it is given, recording every
/// request it receives; stopped, it closes every connection unanswered.
#[cfg(any(test, feature = "fake"))]
pub mod fake {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct State {
        models: Vec<String>,
        stopped: bool,
        requests: Vec<String>,
    }

    pub struct FakeOllama {
        /// Where it answers, e.g. `http://localhost:40123`.
        pub address: String,
        state: Arc<Mutex<State>>,
    }

    impl FakeOllama {
        /// Answers at `localhost` on a free port, with the models.
        pub fn start(models: &[&str]) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let state = Arc::new(Mutex::new(State {
                models: models.iter().map(|m| m.to_string()).collect(),
                ..State::default()
            }));
            let server = state.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    serve(stream, &server);
                }
            });
            FakeOllama {
                address: format!("http://localhost:{port}"),
                state,
            }
        }

        /// Ollama stops: nothing answers.
        pub fn stop(&self) {
            self.state.lock().unwrap().stopped = true;
        }

        /// Ollama starts again.
        pub fn resume(&self) {
            self.state.lock().unwrap().stopped = false;
        }

        /// Every request answered, as method and path.
        pub fn requests(&self) -> Vec<String> {
            self.state.lock().unwrap().requests.clone()
        }
    }

    /// An address where nothing answers: a port just freed.
    pub fn nothing_answers() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://localhost:{}", listener.local_addr().unwrap().port())
    }

    fn serve(mut stream: TcpStream, state: &Mutex<State>) {
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => return,
                Ok(read) => request.extend_from_slice(&buffer[..read]),
            }
        }
        let head = String::from_utf8_lossy(&request).to_string();
        let mut state = state.lock().unwrap();
        if state.stopped {
            return;
        }
        let line: Vec<&str> = head.split_whitespace().take(2).collect();
        state.requests.push(line.join(" "));
        let (status, body) = match line.get(1).copied() {
            Some("/api/version") => ("200 OK", r#"{"version":"0.12.6"}"#.to_string()),
            Some("/api/tags") => {
                let models: Vec<serde_json::Value> = (state.models.iter())
                    .map(|name| serde_json::json!({"name": name, "model": name}))
                    .collect();
                (
                    "200 OK",
                    serde_json::json!({ "models": models }).to_string(),
                )
            }
            _ => ("404 Not Found", "404 page not found".to_string()),
        };
        let _ = write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            body.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::fake::{FakeOllama, nothing_answers};
    use super::*;

    #[test]
    fn an_address_needs_http_or_https_and_a_host() {
        assert_eq!(
            address(" http://localhost:11434/ "),
            Some("http://localhost:11434".into())
        );
        assert_eq!(
            address("https://gpu-box.lan"),
            Some("https://gpu-box.lan".into())
        );
        for wrong in [
            "",
            "localhost:11434",
            "ftp://localhost",
            "http://",
            "not a url",
        ] {
            assert_eq!(address(wrong), None, "{wrong}");
        }
    }

    #[test]
    fn an_address_that_is_not_one_is_refused_and_the_previous_one_kept() {
        let mut settings = Ollama::default();
        let mut form = Form::of(&settings);
        form.address = "localhost 11434".into();
        assert!(!form.enter_address(&mut settings));
        assert!(form.wrong_address);
        assert_eq!(settings.address, DEFAULT_ADDRESS);

        form.address = "http://gpu-box.lan:11434/".into();
        assert!(form.enter_address(&mut settings));
        assert!(!form.wrong_address);
        assert_eq!(settings.address, "http://gpu-box.lan:11434");

        form.model = "  ".into();
        assert!(!form.enter_model(&mut settings));
        assert_eq!(form.model, DEFAULT_MODEL);
    }

    /// Where the parent of `the_check_ignores_a_proxy_set_in_the_environment` runs its fake.
    const UNDER_TEST: &str = "CALENDAR_BUDDY_OLLAMA_UNDER_TEST";

    /// The child half of the test below: run with a proxy in its environment, it checks Ollama
    /// at the address it is given. Does nothing when run on its own.
    #[test]
    fn checked_with_a_proxy_in_the_environment() {
        if let Some(address) = std::env::var_os(UNDER_TEST) {
            let settings = Ollama {
                address: address.into_string().unwrap(),
                ..Ollama::default()
            };
            assert_eq!(check(&settings), Availability::Available);
        }
    }

    #[test]
    fn the_check_ignores_a_proxy_set_in_the_environment() {
        // `MOD-10-NFR-02`: with `HTTP_PROXY` and the like set, the check still goes straight to
        // the address set; the proxy receives nothing. Run in a child process, so that the proxy
        // variables never reach this process's other tests.
        use std::process::Command;
        let ollama = FakeOllama::start(&[DEFAULT_MODEL]);
        let proxy = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy_url = format!("http://{}", proxy.local_addr().unwrap());
        proxy.set_nonblocking(true).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap());
        child.args([
            "--exact",
            "ollama::tests::checked_with_a_proxy_in_the_environment",
        ]);
        Command::env_remove(&mut child, "NO_PROXY");
        Command::env_remove(&mut child, "no_proxy");
        let proxies = [
            "HTTP_PROXY",
            "http_proxy",
            "HTTPS_PROXY",
            "https_proxy",
            "ALL_PROXY",
        ];
        let variables = (proxies.into_iter().map(|name| (name, proxy_url.as_str())))
            .chain([(UNDER_TEST, ollama.address.as_str())]);
        Command::envs(&mut child, variables);
        let output = child.output().unwrap();
        let printed = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{printed}");
        assert!(printed.contains("1 passed"), "{printed}");
        assert!(proxy.accept().is_err(), "the proxy was asked");
        assert_eq!(ollama.requests(), ["GET /api/version", "GET /api/tags"]);
    }

    #[test]
    fn a_model_without_a_tag_is_its_latest() {
        let tags = r#"{"models":[{"name":"llama3.2:latest"},{"name":"gemma4:e4b"}]}"#;
        assert!(has_model(tags, "gemma4:e4b"));
        assert!(has_model(tags, "llama3.2"));
        assert!(!has_model(tags, "gemma4"));
        assert!(!has_model("not json", "gemma4:e4b"));
    }

    #[test]
    fn ollama_is_asked_only_for_its_version_and_its_models() {
        let ollama = FakeOllama::start(&[DEFAULT_MODEL]);
        let settings = Ollama {
            address: ollama.address.clone(),
            ..Ollama::default()
        };
        assert_eq!(check(&settings), Availability::Available);
        assert_eq!(ollama.requests(), ["GET /api/version", "GET /api/tags"]);

        let away = Ollama {
            address: nothing_answers(),
            ..Ollama::default()
        };
        assert_eq!(check(&away), Availability::NotRunning);
    }
}
