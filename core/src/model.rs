use std::fmt::Write as FmtWrite;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub trait CodexModel {
    fn complete(&self, prompt: &str) -> Result<String>;
    /// Chat-style completion with separate system and user messages.
    /// Default implementation falls back to `complete` with concatenation.
    fn complete_chat(&self, system: &str, user: &str) -> Result<String> {
        self.complete(&format!("{}\n\n{}", system, user))
    }
}

pub trait SearchProvider {
    fn search(&self, query: &str) -> Result<String>;
}

pub struct GroqModel {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

pub struct OllamaModel {
    client: Client,
    model: String,
    endpoint: String,
}

pub struct GeminiModel {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

pub struct OpenRouterModel {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

pub struct TavilySearch {
    client: Client,
    api_key: String,
}

static GROQ_LAST_REQUEST_AT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

impl TavilySearch {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("TAVILY_API_KEY")
            .map_err(|_| anyhow!("TAVILY_API_KEY environment variable is not set"))?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }
}

#[derive(Serialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    search_depth: String,
    include_raw_content: bool,
    max_results: i32,
}

#[derive(Deserialize)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

impl SearchProvider for TavilySearch {
    fn search(&self, query: &str) -> Result<String> {
        let body = TavilySearchRequest {
            api_key: self.api_key.clone(),
            query: query.to_string(),
            search_depth: "basic".to_string(),
            include_raw_content: false,
            max_results: 5,
        };

        let response = self
            .client
            .post("https://api.tavily.com/search")
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!("Tavily API error: {}", response.status()));
        }

        let parsed: TavilySearchResponse = response.json()?;
        let mut summary = String::new();
        for res in parsed.results {
            let _ = writeln!(
                &mut summary,
                "### {}\nURL: {}\n{}\n",
                res.title, res.url, res.content
            );
        }
        Ok(summary)
    }
}

impl GroqModel {
    pub fn from_env(model: Option<String>) -> Result<Self> {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| anyhow!("GROQ_API_KEY environment variable is not set"))?;
        let client = Client::new();
        let model = model.unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        let endpoint = "https://api.groq.com/openai/v1/chat/completions".to_string();
        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("Bearer {}", self.api_key)
            .parse()
            .expect("invalid authorization header");
        let content_type = "application/json"
            .parse()
            .expect("invalid content-type header");
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);
        headers.insert(reqwest::header::CONTENT_TYPE, content_type);
        headers
    }

    fn wait_for_request_slot(&self) {
        let min_interval_ms = std::env::var("GROQ_MIN_REQUEST_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(2500);
        let min_interval = Duration::from_millis(min_interval_ms);
        let gate = GROQ_LAST_REQUEST_AT.get_or_init(|| Mutex::new(None));

        loop {
            let sleep_for = {
                let mut last = gate.lock().expect("request gate lock poisoned");
                match *last {
                    Some(prev) => {
                        let elapsed = prev.elapsed();
                        if elapsed >= min_interval {
                            *last = Some(Instant::now());
                            Duration::ZERO
                        } else {
                            min_interval - elapsed
                        }
                    }
                    None => {
                        *last = Some(Instant::now());
                        Duration::ZERO
                    }
                }
            };
            if sleep_for.is_zero() {
                break;
            }
            thread::sleep(sleep_for);
        }
    }

    fn retry_after_delay(response: &reqwest::blocking::Response) -> Option<Duration> {
        response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(Duration::from_secs)
    }
}

impl OllamaModel {
    pub fn from_env(model: Option<String>, endpoint: Option<String>) -> Result<Self> {
        let model = model
            .or_else(|| std::env::var("OLLAMA_MODEL").ok())
            .unwrap_or_else(|| "llama3.1:8b".to_string());
        let endpoint = endpoint
            .or_else(|| std::env::var("OLLAMA_URL").ok())
            .unwrap_or_else(|| "http://localhost:11434/api/generate".to_string());
        Ok(Self {
            client: Client::new(),
            model,
            endpoint,
        })
    }
}

impl GeminiModel {
    pub fn from_env(model: Option<String>) -> Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow!("GEMINI_API_KEY environment variable is not set"))?;
        let client = Client::new();
        let model = model.unwrap_or_else(|| "gemini-2.5-flash".to_string());
        let endpoint = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key);
        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }
}

impl OpenRouterModel {
    pub fn from_env(model: Option<String>) -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow!("OPENROUTER_API_KEY environment variable is not set"))?;
        let client = Client::new();
        // Default to a stable Free Router (2026 choice)
        let model = model.unwrap_or_else(|| "google/gemini-2.0-flash-001:free".to_string());
        let endpoint = "https://openrouter.ai/api/v1/chat/completions".to_string();
        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }
}

#[derive(Serialize)]
struct GroqChatRequest {
    model: String,
    messages: Vec<GroqMessage>,
}

#[derive(Serialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GroqChatResponse {
    choices: Vec<GroqChoice>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqMessageResponse,
}

#[derive(Deserialize)]
struct GroqMessageResponse {
    content: String,
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
}

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Option<Vec<OpenRouterChoice>>,
    error: Option<OpenRouterError>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterResponseMessage,
}

#[derive(Deserialize)]
struct OpenRouterResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenRouterError {
    message: String,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "safetySettings")]
    safety_settings: Vec<GeminiSafetySetting>,
}

#[derive(Serialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

impl CodexModel for GroqModel {
    fn complete(&self, prompt: &str) -> Result<String> {
        self.complete_chat("You are a helpful coding assistant.", prompt)
    }

    fn complete_chat(&self, system: &str, user: &str) -> Result<String> {
        let body = GroqChatRequest {
            model: self.model.clone(),
            messages: vec![
                GroqMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                GroqMessage {
                    role: "user".to_string(),
                    content: user.to_string(),
                },
            ],
        };

        let mut delay = Duration::from_secs(3);
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            self.wait_for_request_slot();
            let response = self
                .client
                .post(&self.endpoint)
                .headers(self.headers())
                .json(&body)
                .send()?;

            let status = response.status();
            
            if status.is_success() {
                let parsed: GroqChatResponse = response.json()?;
                return parsed
                    .choices
                    .first()
                    .map(|c| c.message.content.clone())
                    .ok_or_else(|| anyhow!("Groq API returned no choices"));
            }

            if status.as_u16() == 429 && attempts < max_attempts {
                attempts += 1;
                let retry_delay = Self::retry_after_delay(&response).unwrap_or(delay);
                eprintln!(
                    "  ⚠ Rate limited (429). Retrying in {:?} (attempt {}/{})",
                    retry_delay, attempts, max_attempts
                );
                thread::sleep(retry_delay);
                delay *= 2;
                continue;
            }

            let err_text = response.text().unwrap_or_default();
            return Err(anyhow!("Groq API error: {} - {}", status, err_text));
        }
    }
}

impl CodexModel for OllamaModel {
    fn complete(&self, prompt: &str) -> Result<String> {
        let body = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };
        let response = self
            .client
            .post(&self.endpoint)
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", response.status()));
        }

        let parsed: OllamaGenerateResponse = response.json()?;
        Ok(parsed.response)
    }
}

impl CodexModel for GeminiModel {
    fn complete(&self, prompt: &str) -> Result<String> {
        let body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            safety_settings: vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
            ],
        };

        let mut attempts = 0;
        let max_attempts = 5;

        loop {
            let response = self
                .client
                .post(&self.endpoint)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            let status = response.status();
            
            if status.is_success() {
                let parsed: GeminiResponse = response.json()?;
                if let Some(candidates) = parsed.candidates {
                    if let Some(first) = candidates.into_iter().next() {
                        if let Some(part) = first.content.parts.into_iter().next() {
                            return Ok(part.text);
                        }
                    }
                }
                return Err(anyhow!("Gemini API returned no content"));
            }

            let err_text = response.text().unwrap_or_default();
            
            // Handle 429 Too Many Requests or RESOURCE_EXHAUSTED
            let is_rate_limited = status.as_u16() == 429 
                || err_text.contains("RESOURCE_EXHAUSTED")
                || err_text.contains("quota exceeded")
                || err_text.contains("429");

            if is_rate_limited && attempts < max_attempts {
                attempts += 1;
                // Default backoff: 30s + exponential
                let mut delay_secs = 30 + (10 * attempts);
                
                // Try parsing specific retryDelay from the error string if present (e.g., "retryDelay": "31s")
                if let Some(start_idx) = err_text.find("\"retryDelay\": \"") {
                    let substring = &err_text[start_idx + 15..];
                    if let Some(end_idx) = substring.find("\"") {
                         let inner = &substring[..end_idx];
                         let numeric_part = inner.trim_end_matches('s');
                         if let Ok(parsed_delay) = numeric_part.parse::<f64>() {
                            delay_secs = (parsed_delay as u64) + 2;
                        }
                    }
                }
                
                eprintln!(
                    "  ⚠ Rate limited (Gemini 429). Sleeping for {}s (attempt {}/{})",
                    delay_secs, attempts, max_attempts
                );
                thread::sleep(Duration::from_secs(delay_secs));
                continue;
            }

            return Err(anyhow!("Gemini API error: {} - {}", status, err_text));
        }
    }
}

impl CodexModel for OpenRouterModel {
    fn complete(&self, prompt: &str) -> Result<String> {
        self.complete_chat("You are a helpful coding assistant.", prompt)
    }

    fn complete_chat(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let body = OpenRouterRequest {
            model: self.model.clone(),
            messages: vec![
                OpenRouterMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenRouterMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/ciphrnotfound/cli_codex")
            .header("X-Title", "Astra CLI")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        let status = response.status();
        let parsed: OpenRouterResponse = response.json()?;

        if !status.is_success() {
            let err_msg = parsed
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| format!("Unknown error (HTTP {})", status));
            return Err(anyhow::anyhow!("OpenRouter API error: {}", err_msg));
        }

        if let Some(choices) = parsed.choices {
            if let Some(first) = choices.into_iter().next() {
                if let Some(content) = first.message.content {
                    return Ok(content);
                }
            }
        }
        
        Err(anyhow::anyhow!("OpenRouter API returned no content. (This model might be offline or currently returning empty completions)"))
    }
}
