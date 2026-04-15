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

pub trait EmbeddingProvider {
    fn get_embedding(&self, text: &str) -> Result<Vec<f32>>;
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

#[derive(Clone)]
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
        let config = crate::config::load_global_config();
        let api_key = config.tavily_api_key
            .or_else(|| std::env::var("TAVILY_API_KEY").ok())
            .ok_or_else(|| anyhow!("TAVILY_API_KEY is not set in config or environment"))?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
        Ok(Self {
            client,
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
        let config = crate::config::load_global_config();
        let api_key = config.groq_api_key
            .or_else(|| std::env::var("GROQ_API_KEY").ok())
            .ok_or_else(|| anyhow!("GROQ_API_KEY is not set in config or environment"))?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());
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
        let config = crate::config::load_global_config();
        let model = model
            .or_else(|| std::env::var("OLLAMA_MODEL").ok())
            .unwrap_or_else(|| "llama3.1:8b".to_string());
        let endpoint = endpoint
            .or_else(|| config.ollama_url)
            .or_else(|| std::env::var("OLLAMA_URL").ok())
            .unwrap_or_else(|| "http://localhost:11434/api/generate".to_string());
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());
        Ok(Self {
            client,
            model,
            endpoint,
        })
    }
}

static GEMINI_LAST_REQUEST_AT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

impl GeminiModel {
    pub fn from_env(model: Option<String>) -> Result<Self> {
        let config = crate::config::load_global_config();
        let api_key = config.gemini_api_key
            .or_else(|| std::env::var("GEMINI_API_KEY").ok())
            .ok_or_else(|| anyhow!("GEMINI_API_KEY is not set in config or environment"))?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());
        let model = model.unwrap_or_else(|| "gemini-2.0-flash".to_string());
        let endpoint = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key);
        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }

    fn wait_for_request_slot(&self) {
        // Free tier is 15 RPM (4 seconds per request)
        let min_interval_ms = std::env::var("GEMINI_MIN_REQUEST_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(4100);
        let min_interval = Duration::from_millis(min_interval_ms);
        let gate = GEMINI_LAST_REQUEST_AT.get_or_init(|| Mutex::new(None));

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
}

impl OpenRouterModel {
    pub fn from_env(model: Option<String>) -> Result<Self> {
        let config = crate::config::load_global_config();
        let api_key = config.openrouter_api_key
            .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
            .ok_or_else(|| anyhow!("OPENROUTER_API_KEY is not set in config or environment"))?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());
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

#[derive(Serialize)]
struct GeminiEmbeddingRequest {
    model: String,
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiEmbeddingResponse {
    embedding: GeminiEmbeddingValue,
}

#[derive(Deserialize)]
struct GeminiEmbeddingValue {
    values: Vec<f32>,
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
        self.wait_for_request_slot();
        
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
        let max_attempts = 0; // Seamlessly fail over to Groq immediately

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

impl EmbeddingProvider for GeminiModel {
    fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.wait_for_request_slot();
        
        // Embeddings use the text-embedding-004 model regardless of the chat model
        let embed_endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key={}",
            self.api_key
        );

        let body = GeminiEmbeddingRequest {
            model: "models/text-embedding-004".to_string(),
            content: GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: text.to_string(),
                }],
            },
        };

        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            let response = self
                .client
                .post(&embed_endpoint)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            let status = response.status();
            
            if status.is_success() {
                let parsed: GeminiEmbeddingResponse = response.json()?;
                return Ok(parsed.embedding.values);
            }

            let err_text = response.text().unwrap_or_default();
            let is_rate_limited = status.as_u16() == 429 
                || err_text.contains("RESOURCE_EXHAUSTED")
                || err_text.contains("quota");

            if is_rate_limited && attempts < max_attempts {
                attempts += 1;
                thread::sleep(Duration::from_secs(10 * attempts as u64));
                continue;
            }

            return Err(anyhow!("Gemini Embedding API error: {} - {}", status, err_text));
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

// ──────────────────────────────────────────────────────────────────
// Automatic Failover: Gemini → Groq after N consecutive rate limits
// ──────────────────────────────────────────────────────────────────

static GEMINI_RATE_LIMIT_COUNT: OnceLock<Mutex<u32>> = OnceLock::new();

pub struct FallbackModel {
    primary: Box<dyn CodexModel + Send + Sync>,
    fallback: Option<Box<dyn CodexModel + Send + Sync>>,
    max_rate_limits: u32,
}

impl FallbackModel {
    /// Create a failover model. `primary` is used by default.
    /// After `max_rate_limits` consecutive rate limit errors,
    /// switches to `fallback` for all subsequent calls.
    pub fn new(primary: Box<dyn CodexModel + Send + Sync>, fallback: Option<Box<dyn CodexModel + Send + Sync>>, max_rate_limits: u32) -> Self {
        Self { primary, fallback, max_rate_limits }
    }

    fn rate_limit_count() -> u32 {
        let gate = GEMINI_RATE_LIMIT_COUNT.get_or_init(|| Mutex::new(0));
        *gate.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn increment_rate_limit() -> u32 {
        let gate = GEMINI_RATE_LIMIT_COUNT.get_or_init(|| Mutex::new(0));
        let mut count = gate.lock().unwrap_or_else(|e| e.into_inner());
        *count += 1;
        *count
    }

    fn reset_rate_limit() {
        let gate = GEMINI_RATE_LIMIT_COUNT.get_or_init(|| Mutex::new(0));
        let mut count = gate.lock().unwrap_or_else(|e| e.into_inner());
        *count = 0;
    }

    fn should_use_fallback(&self) -> bool {
        self.fallback.is_some() && Self::rate_limit_count() >= self.max_rate_limits
    }
}

impl CodexModel for FallbackModel {
    fn complete(&self, prompt: &str) -> Result<String> {
        if self.should_use_fallback() {
            if let Some(fb) = &self.fallback {
                eprintln!("  ⚡ Using Groq fallback (Gemini rate-limited {} times)", Self::rate_limit_count());
                return fb.complete(prompt);
            }
        }

        match self.primary.complete(prompt) {
            Ok(result) => {
                Self::reset_rate_limit();
                Ok(result)
            }
            Err(e) => {
                let msg = e.to_string().to_ascii_lowercase();
                if msg.contains("429") || msg.contains("resource_exhausted") || msg.contains("rate") || msg.contains("quota") {
                    let count = Self::increment_rate_limit();
                    eprintln!("  ⚠ Gemini rate limit #{}", count);
                    
                    if let Some(fb) = &self.fallback {
                        if count >= self.max_rate_limits {
                            eprintln!("  ⚡ Switching to Groq fallback permanently ({} rate limits hit)", count);
                        } else {
                            eprintln!("  ⚡ Using Groq fallback for this request");
                        }
                        return fb.complete(prompt);
                    }
                }
                Err(e)
            }
        }
    }

    fn complete_chat(&self, system: &str, user: &str) -> Result<String> {
        if self.should_use_fallback() {
            if let Some(fb) = &self.fallback {
                eprintln!("  ⚡ Using Groq fallback (Gemini rate-limited {} times)", Self::rate_limit_count());
                return fb.complete_chat(system, user);
            }
        }

        match self.primary.complete_chat(system, user) {
            Ok(result) => {
                Self::reset_rate_limit();
                Ok(result)
            }
            Err(e) => {
                let msg = e.to_string().to_ascii_lowercase();
                if msg.contains("429") || msg.contains("resource_exhausted") || msg.contains("rate") || msg.contains("quota") {
                    let count = Self::increment_rate_limit();
                    eprintln!("  ⚠ Gemini rate limit #{}", count);
                    
                    // Instantly use fallback so the user's request doesn't fail
                    if let Some(fb) = &self.fallback {
                        if count >= self.max_rate_limits {
                            eprintln!("  ⚡ Switching to Groq fallback permanently ({} rate limits hit)", count);
                        } else {
                            eprintln!("  ⚡ Using Groq fallback for this request");
                        }
                        return fb.complete_chat(system, user);
                    }
                }
                Err(e)
            }
        }
    }
}

