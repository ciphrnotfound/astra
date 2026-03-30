use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{get_global_config_path, load_global_config};
use crate::teams::{Session, TeamManager};

#[derive(Serialize)]
struct SupabaseSession {
    task_id: String,
    developer: String,
    start_time: u64,
    end_time: u64,
    lines_added: usize,
    lines_deleted: usize,
    prompts_asked: serde_json::Value,
    files_touched: serde_json::Value,
}

#[derive(Serialize)]
struct SupabaseTeamSnapshot {
    repo_path: String,
    team_name: String,
    members: serde_json::Value,
    tasks: serde_json::Value,
    sessions: serde_json::Value,
    updated_at: u64,
}

#[derive(Serialize)]
struct SupabaseAuthProfile {
    user_name: String,
    provider: String,
    last_login_at: u64,
}

fn supabase_credentials() -> Result<(String, String)> {
    let config = load_global_config();
    let url = config
        .supabase_url
        .ok_or_else(|| anyhow!("Supabase URL not configured (run `astra config set supabase_url <url>`)"))?;
    let key = config
        .supabase_key
        .ok_or_else(|| anyhow!("Supabase key not configured (run `astra config set supabase_key <key>`)"))?;
    Ok((url, key))
}

pub fn sync_offline_queue() -> Result<()> {
    let mut path = get_global_config_path();
    path.set_file_name("sync_queue.json");

    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let queue: Vec<Session> = serde_json::from_str(&content).unwrap_or_default();

    if queue.is_empty() {
        return Ok(());
    }

    let (url, key) = supabase_credentials()?;

    let endpoint = format!("{}/rest/v1/astra_sessions", url.trim_end_matches('/'));
    
    let mut payload = Vec::new();
    for session in &queue {
        if let Some(end) = session.end_time {
            payload.push(SupabaseSession {
                task_id: session.task_id.clone(),
                developer: session.developer.clone(),
                start_time: session.start_time,
                end_time: end,
                lines_added: session.lines_added,
                lines_deleted: session.lines_deleted,
                prompts_asked: serde_json::to_value(&session.prompts_asked).unwrap_or(serde_json::Value::Null),
                files_touched: serde_json::to_value(&session.files_touched).unwrap_or(serde_json::Value::Null),
            });
        }
    }

    if payload.is_empty() {
        return Ok(());
    }

    let client = Client::new();
    let res = client.post(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&payload)
        .send()?;

    if res.status().is_success() {
        // Clear the queue
        fs::write(&path, "[]")?;
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        Err(anyhow!("Supabase sync failed with status {}: {}", status, body))
    }
}

pub fn sync_team_state(repo_path: &Path) -> Result<()> {
    let team_mgr = TeamManager::new(repo_path);
    let state = team_mgr.load_state()?;
    if state.team_name.is_empty() {
        return Ok(());
    }

    let (url, key) = supabase_credentials()?;
    let table = std::env::var("ASTRA_SUPABASE_TEAM_TABLE")
        .unwrap_or_else(|_| "astra_team_states".to_string());
    let endpoint = format!(
        "{}/rest/v1/{}?on_conflict=repo_path",
        url.trim_end_matches('/'),
        table
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let payload = vec![SupabaseTeamSnapshot {
        repo_path: repo_path.to_string_lossy().to_string(),
        team_name: state.team_name,
        members: serde_json::to_value(state.members).unwrap_or(serde_json::Value::Null),
        tasks: serde_json::to_value(state.tasks).unwrap_or(serde_json::Value::Null),
        sessions: serde_json::to_value(state.sessions).unwrap_or(serde_json::Value::Null),
        updated_at: now,
    }];

    let client = Client::new();
    let res = client
        .post(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .header("Prefer", "resolution=merge-duplicates,return=minimal")
        .json(&payload)
        .send()?;

    if res.status().is_success() {
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        Err(anyhow!(
            "Supabase team sync failed with status {}: {}",
            status,
            body
        ))
    }
}

pub fn sync_auth_profile(user_name: &str, provider: &str) -> Result<()> {
    let (url, key) = supabase_credentials()?;
    let table = std::env::var("ASTRA_SUPABASE_AUTH_TABLE")
        .unwrap_or_else(|_| "astra_users".to_string());
    let endpoint = format!(
        "{}/rest/v1/{}?on_conflict=user_name",
        url.trim_end_matches('/'),
        table
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let payload = vec![SupabaseAuthProfile {
        user_name: user_name.to_string(),
        provider: provider.to_string(),
        last_login_at: now,
    }];
    let client = Client::new();
    let res = client
        .post(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .header("Prefer", "resolution=merge-duplicates,return=minimal")
        .json(&payload)
        .send()?;

    if res.status().is_success() {
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        Err(anyhow!(
            "Supabase auth profile sync failed with status {}: {}",
            status,
            body
        ))
    }
}
