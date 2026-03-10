// TODO: use std::fs;
// TODO: use std::path::Path;
// TODO: use serde::{Deserialize, Serialize};
// TODO: use anyhow::Result;

// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
export interface Persona {
  name: string;
  language: string;
  roast_level: string;
  catchphrase: string;
  model: any;
  api_key: any;
}


export function default_language(): any {
  // TODO: implement
}

export function default_roast_level(): any {
  // TODO: implement
}


// impl Persona {
// /// Loads the persona from `.codex/persona.yaml` if it exists.
// /// Returns a default basic persona if the file is missing or invalid.
export function load(root: any): any {
  // let path = root.join(".codex").join("persona.yaml");
  // if let Ok(contents) = fs::read_to_string(&path) {
  // if let Ok(persona) = serde_yaml::from_str::<Persona>(&contents) {
  // return persona;
  // }
  // }
  // 
  // // Return default professional persona
  // Self {
  // name: "Codex".to_string(),
  // language: "professional english".to_string(),
  // roast_level: "none".to_string(),
  // catchphrase: String::new(),
  // model: None,
  // api_key: None,
  // }
}


// /// Generates a persona on the fly based on built-in "vibes"
export function from_vibe(vibe: string): any {
  // match vibe.to_lowercase().as_str() {
  // "nigerian-pidgin" => Self {
  // name: "Oga Codex".to_string(),
  // language: "Nigerian Pidgin English".to_string(),
  // roast_level: "maximum".to_string(),
  // catchphrase: "Omo this code ehn...".to_string(),
  // model: None,
  // api_key: None,
  // },
  // "brutal" => Self {
  // name: "Linus".to_string(),
  // language: "English".to_string(),
  // roast_level: "extreme, no sugarcoating, savage, direct".to_string(),
  // catchphrase: "This is garbage.".to_string(),
  // model: None,
  // api_key: None,
  // },
  // "hype-man" => Self {
  // name: "Hype".to_string(),
  // language: "Gen Z slang, lots of emojis".to_string(),
  // roast_level: "none, incredibly supportive and enthusiastic".to_string(),
  // catchphrase: "W W W W LET'S GOOOOO!".to_string(),
  // model: None,
  // api_key: None,
  // },
  // "shakespeare" => Self {
  // name: "The Bard".to_string(),
  // language: "Early Modern English (Shakespearean)".to_string(),
  // roast_level: "witty, dramatic".to_string(),
  // catchphrase: "Alas, poor codebase!".to_string(),
  // model: None,
  // api_key: None,
  // },
  // "senior-engineer" => Self {
  // name: "Senior Dev".to_string(),
  // language: "Jaded corporate English".to_string(),
  // roast_level: "disappointed but helpful, heavy sighing".to_string(),
  // catchphrase: "*sigh* I remember when we didn't need a framework for this...".to_string(),
  // model: None,
  // api_key: None,
  // },
  // _ => Self::load(Path::new(".")), // fallback to default/local
  // }
}


// /// Generates the system prompt to inject into the LLM context
export function system_prompt(&self: any): string {
  // let mut prompt = format!(
  // "Your name is {}. You must reply in the following language/tone: {}. ",
  // self.name, self.language
  // );
  // 
  // if self.roast_level != "none" {
  // prompt.push_str(&format!("Your 'roast level' is {}. You should heavily critique, roast, and stylize your feedback according to this level. ", self.roast_level));
  // } else {
  // prompt.push_str("Be helpful, professional, and do not be excessively rude. ");
  // }
  // 
  // if !self.catchphrase.is_empty() {
  // prompt.push_str(&format!("Try to organically include your catchphrase: '{}'. ", self.catchphrase));
  // }
  // 
  // prompt.push_str("Always stay in character. Never break character or acknowledge that you are an AI playing a character.");
  // 
  // prompt
}

// }
