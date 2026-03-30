use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AstraConfig {
    pub user: Option<String>,
    pub supabase_url: Option<String>,
    pub supabase_key: Option<String>,
    #[serde(default)]
    pub auth_user: Option<String>,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub auth_provider: Option<String>,
}

pub fn get_global_config_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    home.join(".astra").join("config.json")
}

pub fn load_global_config() -> AstraConfig {
    let path = get_global_config_path();
    if let Ok(contents) = fs::read_to_string(path) {
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        AstraConfig::default()
    }
}

pub fn save_global_config(config: &AstraConfig) -> Result<()> {
    let path = get_global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(config)?;
    fs::write(path, contents)?;
    Ok(())
}
