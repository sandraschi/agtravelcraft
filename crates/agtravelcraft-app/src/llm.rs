//! Local-first LLM Settings and non-blocking AI Chat worker (CRAFT_SKILLS_AND_CHAT_STANDARD).
//! - Default local Ollama endpoint (`http://127.0.0.1:11434`), `num_ctx: 16384`, `stream: false`, `think: false`.
//! - Passes request JSON via stdin to `curl.exe` (`--data-binary @-`), never on the command line.
//! - Zero fake fallback text: errors surface verbatim with a Settings / Retry prompt.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    #[default]
    Ollama,
    OpenAiCompatible,
}

impl LlmProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ollama => "Ollama (Local)",
            Self::OpenAiCompatible => "OpenAI-Compatible",
        }
    }

    pub fn is_local(self, endpoint: &str) -> bool {
        matches!(self, Self::Ollama)
            || endpoint.contains("127.0.0.1")
            || endpoint.contains("localhost")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChatPersonality {
    #[default]
    DiplomaticCourier,
    TokyoNekomimiArchitect,
    ViennaCustomsInspector,
    ConcisePeer,
}

impl ChatPersonality {
    pub const ALL: [Self; 4] = [
        Self::DiplomaticCourier,
        Self::TokyoNekomimiArchitect,
        Self::ViennaCustomsInspector,
        Self::ConcisePeer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::DiplomaticCourier => "Diplomatic Courier",
            Self::TokyoNekomimiArchitect => "Tokyo Nekomimi Gal",
            Self::ViennaCustomsInspector => "Vienna Customs Hawk",
            Self::ConcisePeer => "Concise Peer",
        }
    }

    pub fn prompt_directive(self) -> &'static str {
        match self {
            Self::DiplomaticCourier => {
                "Persona: Calm, precise Viennese Diplomatic Courier. Summarize passports, transit stamps, and council minutes clearly."
            }
            Self::TokyoNekomimiArchitect => {
                "Persona: Bright, technically sharp Tokyo CraftApp nekomimi engineer gal hosting the Diplomatic Teahouse. Keep answers playful yet grounded in real data and red lines."
            }
            Self::ViennaCustomsInspector => {
                "Persona: Strict, zero-trust Vienna Customs security inspector. Prioritize prompt-injection flags, red line enforcement, and taint quarantine analysis."
            }
            Self::ConcisePeer => {
                "Persona: Ultra-terse senior engineering peer. Bullet points only, zero fluff."
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub provider: LlmProvider,
    pub endpoint: String,
    pub model: String,
    pub temperature: f32,
    pub num_ctx: u32,
    pub api_key_env: String,
    pub extra_instructions: String,
    #[serde(default)]
    pub personality: ChatPersonality,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Ollama,
            endpoint: "http://127.0.0.1:11434".to_string(),
            model: "qwen3.8:27b".to_string(),
            temperature: 0.2,
            num_ctx: 16384,
            api_key_env: "OPENAI_API_KEY".to_string(),
            extra_instructions: String::new(),
            personality: ChatPersonality::default(),
        }
    }
}

impl LlmSettings {
    pub fn config_path(app_folder: &str) -> PathBuf {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let mut p = PathBuf::from(local);
            p.push(app_folder);
            p.push("llm_settings.json");
            return p;
        }
        PathBuf::from(format!("{app_folder}_llm_settings.json"))
    }

    pub fn load(app_folder: &str) -> Self {
        let path = Self::config_path(app_folder);
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, app_folder: &str) -> Result<(), String> {
        let path = Self::config_path(app_folder);
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let body = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, body).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub is_error: bool,
}

pub struct ChatSession {
    pub settings: LlmSettings,
    pub available_models: Vec<String>,
    pub messages: Vec<ChatMessage>,
    pub draft: String,
    pub show_prompt_modal: bool,
    pub last_preprompt: String,
    pub pending_rx: Option<Receiver<Result<String, String>>>,
    pub busy: bool,
}

impl ChatSession {
    pub fn new(app_folder: &str) -> Self {
        let settings = LlmSettings::load(app_folder);
        let available_models = default_models_for_provider(settings.provider, &settings.model);
        Self {
            settings,
            available_models,
            messages: Vec::new(),
            draft: String::new(),
            show_prompt_modal: false,
            last_preprompt: String::new(),
            pending_rx: None,
            busy: false,
        }
    }

    pub fn refresh_models(&mut self) -> Result<usize, String> {
        let fetched = fetch_provider_models(&self.settings)?;
        if fetched.is_empty() {
            self.available_models =
                default_models_for_provider(self.settings.provider, &self.settings.model);
            return Err("Endpoint returned 0 models; kept preset list.".to_string());
        }
        self.available_models = fetched;
        if !self.available_models.contains(&self.settings.model)
            && let Some(first) = self.available_models.first()
        {
            self.settings.model = first.clone();
        }
        Ok(self.available_models.len())
    }

    pub fn build_preprompt(
        &self,
        app_name: &str,
        skills: &str,
        live_status: &str,
        selection: &str,
    ) -> String {
        let mut out = format!(
            "You are the embedded assistant for {app_name}. Answer strictly from the provided skills, live status, and selection. Say when you do not know. Never invent tool names or numbers.\n\n## Active Personality\n{}\n\n## Bundled Skills\n{skills}\n\n## Live Status\n{live_status}\n",
            self.settings.personality.prompt_directive()
        );
        if !selection.is_empty() {
            out.push_str("\n## Selected Item\n");
            out.push_str(selection);
            out.push('\n');
        }
        if !self.settings.extra_instructions.trim().is_empty() {
            out.push_str("\n## User Instructions\n");
            out.push_str(self.settings.extra_instructions.trim());
            out.push('\n');
        }
        out
    }

    pub fn export_markdown(&self, app_name: &str) -> String {
        let mut out = format!(
            "# {app_name} AI Chat Transcript\n\n- **Personality:** {}\n- **Model:** {} ({})\n- **Messages:** {}\n\n---\n\n",
            self.settings.personality.label(),
            self.settings.model,
            self.settings.provider.label(),
            self.messages.len()
        );
        for msg in &self.messages {
            let heading = if msg.is_error {
                "ERROR"
            } else if msg.role == "user" {
                "User"
            } else {
                "Assistant"
            };
            out.push_str(&format!("### {heading}\n\n{}\n\n", msg.content));
        }
        out
    }

    pub fn export_to_file(&self, app_folder: &str, app_name: &str) -> Result<PathBuf, String> {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let dir = if let Ok(local) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local).join(app_folder).join("chats")
        } else {
            PathBuf::from("chats")
        };
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("chat_{ts}.md"));
        std::fs::write(&path, self.export_markdown(app_name)).map_err(|e| e.to_string())?;
        Ok(path)
    }

    pub fn retry_last(&mut self, preprompt: String) {
        if self.busy {
            return;
        }
        if let Some(last_user) = self
            .messages
            .iter()
            .rev()
            .find(|m| m.role == "user" && !m.is_error)
            .map(|m| m.content.clone())
        {
            self.draft = last_user;
            self.send(preprompt);
        }
    }

    pub fn send(&mut self, preprompt: String) {
        let prompt = self.draft.trim().to_string();
        if prompt.is_empty() || self.busy {
            return;
        }
        if self.settings.model.trim().is_empty() {
            self.messages.push(ChatMessage {
                role: "error".to_string(),
                content: "No model configured. Open LLM & Settings to choose a model.".to_string(),
                is_error: true,
            });
            return;
        }
        self.draft.clear();
        self.last_preprompt = preprompt.clone();
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.clone(),
            is_error: false,
        });

        let settings = self.settings.clone();
        let history: Vec<(String, String)> = self
            .messages
            .iter()
            .filter(|m| !m.is_error)
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect();

        let (tx, rx) = mpsc::channel();
        self.pending_rx = Some(rx);
        self.busy = true;

        std::thread::spawn(move || {
            let res = invoke_llm(&settings, &preprompt, &history);
            let _ = tx.send(res);
        });
    }

    pub fn poll(&mut self) {
        let Some(ref rx) = self.pending_rx else {
            return;
        };
        if let Ok(result) = rx.try_recv() {
            self.busy = false;
            self.pending_rx = None;
            match result {
                Ok(reply) => self.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: reply,
                    is_error: false,
                }),
                Err(err) => self.messages.push(ChatMessage {
                    role: "error".to_string(),
                    content: err,
                    is_error: true,
                }),
            }
        }
    }
}

pub fn default_models_for_provider(provider: LlmProvider, current: &str) -> Vec<String> {
    let mut list: Vec<String> = match provider {
        LlmProvider::Ollama => vec![
            "qwen3.8:27b".to_string(),
            "qwen2.5:14b".to_string(),
            "qwen2.5-coder:14b".to_string(),
            "llama3.3:70b".to_string(),
            "mistral-nemo:12b".to_string(),
            "deepseek-r1:14b".to_string(),
            "gemma3:12b".to_string(),
        ],
        LlmProvider::OpenAiCompatible => vec![
            "gpt-4.1-mini".to_string(),
            "gpt-4.1".to_string(),
            "claude-sonnet-4-5".to_string(),
            "gemini-2.5-flash".to_string(),
        ],
    };
    if !current.trim().is_empty() && !list.iter().any(|m| m == current.trim()) {
        list.insert(0, current.trim().to_string());
    }
    list
}

fn fetch_provider_models(settings: &LlmSettings) -> Result<Vec<String>, String> {
    let base = settings.endpoint.trim_end_matches('/');
    let url = match settings.provider {
        LlmProvider::Ollama => format!("{base}/api/tags"),
        LlmProvider::OpenAiCompatible => format!("{base}/v1/models"),
    };
    let mut cmd = Command::new("curl.exe");
    cmd.args(["-sS", "--max-time", "3", &url]);
    if matches!(settings.provider, LlmProvider::OpenAiCompatible)
        && !settings.api_key_env.trim().is_empty()
        && let Ok(key) = std::env::var(settings.api_key_env.trim())
        && !key.trim().is_empty()
    {
        cmd.args(["-H", &format!("Authorization: Bearer {}", key.trim())]);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to invoke curl.exe: {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("model refresh failed: {}", stderr.trim()));
    }
    let val: Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("invalid JSON from {url}: {e}"))?;
    let mut models = Vec::new();
    match settings.provider {
        LlmProvider::Ollama => {
            if let Some(arr) = val.get("models").and_then(Value::as_array) {
                for item in arr {
                    if let Some(name) = item.get("name").and_then(Value::as_str) {
                        models.push(name.to_string());
                    }
                }
            }
        }
        LlmProvider::OpenAiCompatible => {
            if let Some(arr) = val.get("data").and_then(Value::as_array) {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(Value::as_str) {
                        models.push(id.to_string());
                    }
                }
            }
        }
    }
    models.sort();
    models.dedup();
    Ok(models)
}

fn invoke_llm(
    settings: &LlmSettings,
    system_prompt: &str,
    history: &[(String, String)],
) -> Result<String, String> {
    let mut msgs = Vec::with_capacity(history.len() + 1);
    msgs.push(json!({ "role": "system", "content": system_prompt }));
    for (r, c) in history {
        msgs.push(json!({ "role": r, "content": c }));
    }

    let base = settings.endpoint.trim_end_matches('/');
    let (url, body) = match settings.provider {
        LlmProvider::Ollama => (
            format!("{base}/api/chat"),
            json!({
                "model": settings.model,
                "messages": msgs,
                "stream": false,
                "think": false,
                "options": {
                    "temperature": settings.temperature,
                    "num_ctx": settings.num_ctx
                }
            }),
        ),
        LlmProvider::OpenAiCompatible => (
            format!("{base}/v1/chat/completions"),
            json!({
                "model": settings.model,
                "messages": msgs,
                "temperature": settings.temperature,
                "stream": false
            }),
        ),
    };

    let body_bytes = serde_json::to_vec(&body).map_err(|e| format!("encode error: {e}"))?;
    let mut cmd = Command::new("curl.exe");
    cmd.args([
        "-sS",
        "--max-time",
        "45",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "--data-binary",
        "@-",
        &url,
    ]);

    if matches!(settings.provider, LlmProvider::OpenAiCompatible)
        && !settings.api_key_env.trim().is_empty()
        && let Ok(key) = std::env::var(settings.api_key_env.trim())
        && !key.trim().is_empty()
    {
        cmd.arg("-H")
            .arg(format!("Authorization: Bearer {}", key.trim()));
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to launch curl.exe: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&body_bytes)
            .map_err(|e| format!("stdin write error: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("curl wait error: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LLM request failed ({url}): {}", stderr.trim()));
    }

    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("invalid JSON from LLM endpoint: {e}"))?;
    if let Some(err_msg) = parsed.get("error") {
        return Err(format!("LLM provider error: {err_msg}"));
    }

    let reply = parsed
        .pointer("/message/content")
        .or_else(|| parsed.pointer("/choices/0/message/content"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "LLM returned an empty reply".to_string())?;

    Ok(reply.to_string())
}
