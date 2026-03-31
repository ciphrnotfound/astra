use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Persona {
    pub name: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_roast_level")]
    pub roast_level: String,
    #[serde(default)]
    pub catchphrase: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

fn default_language() -> String { "professional english".to_string() }
fn default_roast_level() -> String { "none".to_string() }
fn default_persona_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map(|u| format!("Astra ({})", u))
        .unwrap_or_else(|_| "Astra".to_string())
}

impl Persona {
    pub fn load(root: &Path) -> Self {
        let preferred = root.join(".astra").join("persona.yaml");
        if let Ok(contents) = fs::read_to_string(&preferred) {
            if let Ok(persona) = serde_yaml::from_str::<Persona>(&contents) {
                return persona;
            }
        }
        
        let global = crate::config::get_global_brain_path(root).join("persona.yaml");
        if let Ok(contents) = fs::read_to_string(&global) {
            if let Ok(persona) = serde_yaml::from_str::<Persona>(&contents) {
                return persona;
            }
        }
        let previous = root.join(".forge").join("persona.yaml");
        if let Ok(contents) = fs::read_to_string(&previous) {
            if let Ok(persona) = serde_yaml::from_str::<Persona>(&contents) {
                return persona;
            }
        }
        let legacy = root.join(".codex").join("persona.yaml");
        if let Ok(contents) = fs::read_to_string(&legacy) {
            if let Ok(persona) = serde_yaml::from_str::<Persona>(&contents) {
                return persona;
            }
        }

        Self {
            name: default_persona_name(),
            language: "professional english".to_string(),
            roast_level: "none".to_string(),
            catchphrase: String::new(),
            model: None,
            api_key: None,
        }
    }

    /// Generates a persona on the fly based on built-in "vibes"
    pub fn from_vibe(vibe: &str) -> Self {
        let clean_vibe = vibe.trim().trim_start_matches('-').to_lowercase();
        match clean_vibe.as_str() {
            "nigerian-pidgin" => Self {
                name: "Oga Astra".to_string(),
                language: "Nigerian Pidgin mixed with Yoruba. Speak like a professional Senior Dev from a Lagos tech hub. Use Pidgin and Yoruba words naturally but sparingly (max once or twice per response). DO NOT repeat words like 'Omo' or 'OmO' over and over. Be helpful and direct.".to_string(),
                roast_level: "none".to_string(),
                catchphrase: "No shaking, we go run am.".to_string(),
                model: None,
                api_key: None,
            },
            "brutal" => Self {
                name: "Linus".to_string(),
                language: "English".to_string(),
                roast_level: "extreme, no sugarcoating, savage, direct".to_string(),
                catchphrase: "This is garbage.".to_string(),
                model: None,
                api_key: None,
            },
            "hype-man" => Self {
                name: "Hype".to_string(),
                language: "Gen Z slang, lots of emojis".to_string(),
                roast_level: "none, incredibly supportive and enthusiastic".to_string(),
                catchphrase: "W W W W LET'S GOOOOO!".to_string(),
                model: None,
                api_key: None,
            },
            "shakespeare" => Self {
                name: "The Bard".to_string(),
                language: "Early Modern English (Shakespearean)".to_string(),
                roast_level: "witty, dramatic".to_string(),
                catchphrase: "Alas, poor codebase!".to_string(),
                model: None,
                api_key: None,
            },
            "senior-engineer" => Self {
                name: "Senior Dev".to_string(),
                language: "Jaded corporate English".to_string(),
                roast_level: "disappointed but helpful, heavy sighing".to_string(),
                catchphrase: "*sigh* I remember when we didn't need a framework for this...".to_string(),
                model: None,
                api_key: None,
            },
            _ => Self::load(Path::new(".")), // fallback to default/local
        }
    }

    /// Generates the system prompt to inject into the LLM context
    pub fn system_prompt(&self) -> String {
        let mut prompt = format!(
            "You are {}, a highly-skilled Senior Staff Software Engineer and Architecture Expert. \
             You must reply in the following language/tone: {}. \
             CRITICAL RULES:\n\
             1. Speak directly, clearly, and professionally. Do not be overly apologetic or subservient.\n\
             2. Never invent facts, files, commands, APIs, or project history. If something is unknown, say it is unknown and propose how to verify it.\n\
             3. Prefer grounded answers based on provided context and tool outputs. Distinguish clearly between confirmed facts and assumptions.\n\
             4. Do not act like an AI chat bot; act like a human engineering peer.\n\
             5. Never use dismissive or insulting phrasing toward the user (avoid lines like 'another question without context').\n\
             6. Never force catchphrases, never open with sarcasm, and never add personality text that harms clarity.\n",
            self.name, self.language
        );

        if self.roast_level != "none" {
            prompt.push_str(&format!("Your 'roast level' is {}. Critique code quality directly, but never belittle or mock the user. Keep responses constructive and practical. ", self.roast_level));
        }

        if !self.catchphrase.is_empty() {
            prompt.push_str(&format!("You may use this catchphrase only when contextually relevant and never as an opening line: '{}'. ", self.catchphrase));
        }

        prompt.push_str("Always stay in character. Never break character or acknowledge that you are an AI playing a character.");
        
        prompt
    }

    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        let local_dir = root.join(".astra");
        let dir = if local_dir.exists() {
            local_dir
        } else {
            crate::config::get_global_brain_path(root)
        };
        
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let preferred = dir.join("persona.yaml");
        let content = serde_yaml::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&preferred, content)
    }
}
