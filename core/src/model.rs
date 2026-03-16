use std::fmt::Write as FmtWrite;
use std::thread;
use std::time::Duration;
use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub trait CodexModel {
    fn complete(&self, prompt: &str) -> Result<String>;
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

pub struct TavilySearch {
    client: Client,
    api_key: String,
}

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
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

impl CodexModel for GroqModel {
    fn complete(&self, prompt: &str) -> Result<String> {
        let body = GroqChatRequest {
            model: self.model.clone(),
            messages: vec![GroqMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let mut delay = Duration::from_secs(2);
        let mut attempts = 0;
        let max_attempts = 5;

        loop {
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
                eprintln!("  ⚠ Rate limited (429). Retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                thread::sleep(delay);
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
