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

/// Computes a unique project identifier based on the absolute path.
pub fn get_global_project_id(project_root: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let abs_path = project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());
    let path_str = abs_path.to_string_lossy().to_string();

    let name = abs_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown_project");
    
    let mut hasher = DefaultHasher::new();
    path_str.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());

    // Format: name_hash (e.g., cli_codex_a1b2c3d4)
    format!("{}_{}", name, hash.chars().take(8).collect::<String>())
}

/// Returns the global brain directory for the current project.
/// `~/.astra/brain/<project_id>/`
pub fn get_global_brain_path(project_root: &std::path::Path) -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
        
    let project_id = get_global_project_id(project_root);
    home.join(".astra").join("brain").join(project_id)
}
