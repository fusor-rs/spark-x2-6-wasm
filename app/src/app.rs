use fusor::{JsInputs, Signal, signal};
use serde::{Deserialize, Serialize};

use crate::examples::{EXAMPLES, Example};
use crate::markdown::{self, Block};

/// Who wrote a message.
#[derive(Clone, Copy, PartialEq)]
enum Role {
    User,
    Model,
}

impl Role {
    fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Model => "model",
        }
    }
}

/// A message of the conversation as the page shows it, a reply's reasoning apart.
#[derive(Clone, PartialEq)]
struct Entry {
    index: usize,
    role: Role,
    text: String,
    reasoning: Option<String>,
}

impl Entry {
    fn new(index: usize, role: Role, text: String) -> Self {
        Self {
            index,
            role,
            text,
            reasoning: None,
        }
    }

    fn reasons(&self) -> bool {
        self.reasoning.is_some()
    }

    fn is_user(&self) -> bool {
        self.role == Role::User
    }

    /// A reply's Markdown, drawn.
    fn blocks(&self) -> Vec<Block> {
        markdown::blocks(&self.text)
    }

    fn reasoning(&self) -> String {
        // Shown only for a reply that reasons.
        self.reasoning.clone().unwrap_or_default()
    }
}

/// The runtime's sampling: `spark_x2_6::sampling::Sampling`.
#[derive(Serialize)]
struct Sampling {
    temperature: f64,
    top_p: f64,
    top_k: u32,
    presence_penalty: f64,
}

/// The runtime's message, `spark_x2_6::session::Message`, but its seed, which `web/app.js` draws.
#[derive(Serialize)]
struct Message {
    text: String,
    thinking: bool,
    max_tokens: u32,
    /// None for the model card's settings.
    sampling: Option<Sampling>,
}

// The runtime build and the bundle, once loaded.
#[derive(Clone, PartialEq, Deserialize)]
struct Loaded {
    build: String,
    threads: u32,
    bundle_bytes: f64,
    load_ms: f64,
    // Every part came from this device's storage.
    from_device: bool,
}

impl Loaded {
    fn summary(&self) -> String {
        let threads = match self.threads {
            1 => "1 thread".to_string(),
            count => format!("{count} threads"),
        };
        let source = if self.from_device {
            "model loaded from this device"
        } else {
            "model downloaded"
        };
        format!(
            "{} · {threads} · {:.0} MB bundle · {source} in {:.1} s",
            self.build,
            self.bundle_bytes / 1e6,
            self.load_ms / 1000.0
        )
    }
}

/// What `web/app.js` measures while a reply is written: the delay from Send to its first token
/// (`first_ms`), the tokens written and the time since that first one (`writing_ms`), the
/// conversation's tokens and the prompt's, which each turn reads whole, and the network requests
/// made since the model loaded.
#[derive(Clone, Default, PartialEq, Deserialize)]
struct Metrics {
    first_ms: Option<f64>,
    tokens: u32,
    writing_ms: f64,
    context_tokens: u32,
    prompt_tokens: u32,
    requests: u32,
}

impl Metrics {
    fn first_token(&self) -> String {
        self.first_ms
            .map_or("–".into(), |ms| format!("{:.2} s", ms / 1000.0))
    }

    fn speed(&self) -> String {
        if self.tokens < 2 || self.writing_ms == 0.0 {
            return "–".into();
        }
        format!(
            "{:.1}/s",
            f64::from(self.tokens - 1) * 1000.0 / self.writing_ms
        )
    }

    fn tokens(&self) -> String {
        self.tokens.to_string()
    }

    fn context(&self) -> String {
        self.context_tokens.to_string()
    }

    fn read(&self) -> String {
        format!("{} read this turn", self.prompt_tokens)
    }

    fn requests(&self) -> String {
        self.requests.to_string()
    }
}

// What `web/app.js` reports, one JSON object per `engine` event.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Report {
    Loading {
        fraction: f64,
        detail: String,
    },
    Ready {
        loaded: Loaded,
    },
    // The reply so far.
    Reply {
        status: String,
        reasoning: Option<String>,
        answer: String,
        metrics: Metrics,
    },
    // The reply is written, or stopped.
    Done {
        status: String,
        metrics: Metrics,
    },
    // Something failed; the engine can take the next message.
    Error {
        message: String,
    },
    // The model could not load, or the engine died: nothing more can run in this page.
    Broken {
        message: String,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Idle,
    Replying,
    Broken,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Replying => "replying",
            Self::Broken => "broken",
        }
    }
}

/// The longest reply the page asks for unless told otherwise.
const MAX_TOKENS: u32 = 2048;

#[derive(JsInputs)]
struct App {
    text: Signal<String>,
    thinking: Signal<bool>,
    // The model card's settings for the turn, unless the fields below are used.
    card_sampling: Signal<bool>,
    temperature: Signal<f64>,
    top_p: Signal<f64>,
    top_k: Signal<u32>,
    presence_penalty: Signal<f64>,
    max_tokens: Signal<u32>,
    // `web/app.js` acts on every new count: `sends` (Send) sends `message` to the engine, `stops`
    // stops the reply, `resets` starts a new conversation.
    #[js]
    message: Signal<String>,
    #[js]
    sends: Signal<u32>,
    #[js]
    stops: Signal<u32>,
    #[js]
    resets: Signal<u32>,
    phase: Signal<Phase>,
    fraction: Signal<f64>,
    detail: Signal<String>,
    status: Signal<String>,
    entries: Signal<Vec<Entry>>,
    metrics: Signal<Metrics>,
    loaded: Signal<Option<Loaded>>,
}

impl App {
    fn new() -> Self {
        Self {
            text: signal(String::new()),
            thinking: signal(true),
            card_sampling: signal(true),
            temperature: signal(0.7),
            top_p: signal(0.95),
            top_k: signal(40),
            presence_penalty: signal(0.0),
            max_tokens: signal(MAX_TOKENS),
            message: signal(String::new()),
            sends: signal(0),
            stops: signal(0),
            resets: signal(0),
            phase: signal(Phase::Idle),
            fraction: signal(0.0),
            detail: signal("Starting…".into()),
            status: signal("Write a message, or start from an example.".into()),
            entries: signal(Vec::new()),
            metrics: signal(Metrics::default()),
            loaded: signal(None),
        }
    }

    fn examples(&self) -> Vec<Example> {
        EXAMPLES.to_vec()
    }

    fn pick(&self, example: &Example) {
        self.text.set(example.text.into());
    }

    fn send(&self) {
        let text = self.text.get();
        if text.trim().is_empty() {
            return self.status.set("Write a message first.".into());
        }
        let sampling = (!self.card_sampling.get()).then(|| Sampling {
            temperature: self.temperature.get(),
            top_p: self.top_p.get(),
            top_k: self.top_k.get(),
            presence_penalty: self.presence_penalty.get(),
        });
        let message = Message {
            text: text.clone(),
            thinking: self.thinking.get(),
            max_tokens: self.max_tokens.get(),
            sampling,
        };
        self.message
            .set(serde_json::to_string(&message).expect("a message serializes"));
        self.entries.update(|entries| {
            let index = entries.len();
            entries.push(Entry::new(index, Role::User, text));
            entries.push(Entry::new(index + 1, Role::Model, String::new()));
        });
        self.text.set(String::new());
        self.metrics.set(Metrics::default());
        self.phase.set(Phase::Replying);
        let status = if self.loaded.get().is_some() {
            "Reading the conversation…"
        } else {
            "Replying once the model is ready."
        };
        self.status.set(status.into());
        self.sends.update(|count| *count += 1);
    }

    fn stop(&self) {
        self.stops.update(|count| *count += 1);
    }

    fn reset(&self) {
        self.entries.set(Vec::new());
        self.metrics.set(Metrics::default());
        self.status.set("A new conversation.".into());
        self.resets.update(|count| *count += 1);
    }

    fn receive(&self, event: web_sys::Event) {
        let json = fusor::js::event_detail::<String>(&event).expect("web/app.js sends strings");
        let report = serde_json::from_str(&json).expect("web/app.js sends engine reports");
        match report {
            Report::Loading { fraction, detail } => {
                self.fraction.set(fraction);
                self.detail.set(detail);
            }
            Report::Ready { loaded } => self.loaded.set(Some(loaded)),
            Report::Reply {
                status,
                reasoning,
                answer,
                metrics,
            } => {
                self.entries.update(|entries| {
                    let reply = entries.last_mut().expect("a reply follows its message");
                    reply.text = answer;
                    reply.reasoning = reasoning;
                });
                self.status.set(status);
                self.metrics.set(metrics);
            }
            Report::Done { status, metrics } => {
                self.status.set(status);
                self.metrics.set(metrics);
                self.phase.set(Phase::Idle);
            }
            Report::Error { message } => {
                self.status.set(format!("error: {message}"));
                self.phase.set(Phase::Idle);
            }
            Report::Broken { message } => {
                self.detail.set(message);
                self.phase.set(Phase::Broken);
            }
        }
    }

    fn summary(&self) -> String {
        self.loaded
            .get()
            .map_or(String::new(), |loaded| loaded.summary())
    }

    fn loading(&self) -> bool {
        self.loaded.get().is_none() && self.phase.get() != Phase::Broken
    }

    // The model never loaded: likely a bundle that does not match the runtime.
    fn failed_to_load(&self) -> bool {
        self.loaded.get().is_none()
    }

    fn idle(&self) -> bool {
        self.phase.get() == Phase::Idle
    }

    fn send_label(&self) -> &'static str {
        match self.phase.get() {
            Phase::Replying => "Replying…",
            _ if self.loaded.get().is_none() => "Send now, replies when ready",
            _ => "Send",
        }
    }

    fn empty(&self) -> bool {
        self.entries.get().is_empty()
    }
}

fusor::template!("web/index.html");
