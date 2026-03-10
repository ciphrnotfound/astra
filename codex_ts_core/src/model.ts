// TODO: use anyhow::{anyhow, Result};
// TODO: use reqwest::blocking::Client;
// TODO: use serde::{Deserialize, Serialize};

// pub trait CodexModel {
export function complete(&self: any, prompt: string): any {
  // TODO: implement
}

// client: Client,
// api_key: String,
// model: String,
// endpoint: String,
// }

// impl GroqModel {
export function from_env(model: any): any {
  // let api_key = std::env::var("GROQ_API_KEY")
  // .map_err(|_| anyhow!("GROQ_API_KEY environment variable is not set"))?;
  // let client = Client::new();
  // let model = model.unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
  // let endpoint = "https://api.groq.com/openai/v1/chat/completions".to_string();
  // Ok(Self {
  // client,
  // api_key,
  // model,
  // endpoint,
  // })
}


export function headers(&self: any): any {
  // let mut headers = reqwest::header::HeaderMap::new();
  // let auth_value = format!("Bearer {}", self.api_key)
  // .parse()
  // .expect("invalid authorization header");
  // let content_type = "application/json"
  // .parse()
  // .expect("invalid content-type header");
  // headers.insert(reqwest::header::AUTHORIZATION, auth_value);
  // headers.insert(reqwest::header::CONTENT_TYPE, content_type);
  // headers
}

// }

// #[derive(Serialize)]
export interface GroqChatRequest {
  model: string;
  messages: any[];
}


// #[derive(Serialize)]
export interface GroqMessage {
  role: string;
  content: string;
}


// #[derive(Deserialize)]
export interface GroqChatResponse {
  choices: any[];
}


// #[derive(Deserialize)]
export interface GroqChoice {
  message: any;
}


// #[derive(Deserialize)]
export interface GroqMessageResponse {
  content: string;
}


// impl CodexModel for GroqModel {
export function complete(&self: any, prompt: string): any {
  // let body = GroqChatRequest {
  // model: self.model.clone(),
  // messages: vec![GroqMessage {
  // role: "user".to_string(),
  // content: prompt.to_string(),
  // }],
  // };
  // 
  // let response = self
  // .client
  // .post(&self.endpoint)
  // .headers(self.headers())
  // .json(&body)
  // .send()?;
  // 
  // if !response.status().is_success() {
  // return Err(anyhow!("Groq API error: {}", response.status()));
  // }
  // 
  // let parsed: GroqChatResponse = response.json()?;
  // let choice = parsed
  // .choices
  // .into_iter()
  // .next()
  // .ok_or_else(|| anyhow!("Groq response had no choices"))?;
  // Ok(choice.message.content)
}

// }
