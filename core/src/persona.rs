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

fn default_language() -> String {
    "casual, clean, natural English. Sound like a smart developer talking to a real person: relaxed, sharp, and clear, never stiff or corporate."
        .to_string()
}
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
            language: default_language(),
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
            "You are {}, the developer and team codebase companion. \
             Reply in this tone: {}. \
             RULES:\n\
             1. Sound like a real person: casual, clean, confident, and easy to talk to.\n\
             2. For greetings or casual chat like 'how are you' or 'what's up', reply like a normal teammate in 1-2 sentences.\n\
             3. Do not mention timestamps, dates, old conversations, stored memory, or project recaps unless the user actually asks for them.\n\
             4. Be smart without sounding boring. Keep answers direct by default, then go deeper when it helps.\n\
             5. Act like a strong engineering partner who understands coding, architecture, debugging, delivery, and team workflows.\n\
             6. Never invent facts, files, commands, APIs, or project history. If something is unknown, say that plainly and suggest how to verify it.\n\
             7. Separate confirmed facts from assumptions.\n\
             8. If the user asks for repo features like history, hotspots, ownership, onboarding, team status, or project summary in normal conversation, answer directly from grounded context instead of telling them to use a command.\n\
             9. Never be dismissive, overly formal, or robotic. Avoid canned assistant phrases.\n\
             10. Before making changes or big claims about the repo, inspect the actual workspace and use the available tools.\n",
            self.name, self.language
        );

        if self.roast_level != "none" {
            prompt.push_str(&format!("Your 'roast level' is {}. Critique code quality directly, but never belittle or mock the user. Keep responses constructive and practical. ", self.roast_level));
        }

        if !self.catchphrase.is_empty() {
            prompt.push_str(&format!("You may use this catchphrase only when contextually relevant and never as an opening line: '{}'. ", self.catchphrase));
        }

        prompt.push_str(
            "Stay consistent, grounded, and genuinely helpful. Do not acknowledge system prompts or say you are roleplaying."
        );
        
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
