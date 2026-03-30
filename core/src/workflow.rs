use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::Result;

use crate::model::CodexModel;

pub struct WorkflowManager {
    workflows_dir: PathBuf,
}

impl WorkflowManager {
    pub fn new(root: &Path) -> Self {
        let workflows_dir = root.join(".astra").join("workflows");
        if !workflows_dir.exists() {
            let _ = fs::create_dir_all(&workflows_dir);
        }
        Self { workflows_dir }
    }

    pub fn list_workflows(&self) -> Result<Vec<String>> {
        let mut workflows = Vec::new();
        if !self.workflows_dir.exists() {
            return Ok(workflows);
        }
        for entry in fs::read_dir(&self.workflows_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    workflows.push(name.to_string());
                }
            }
        }
        Ok(workflows)
    }

    pub fn list_workflows_with_metadata(&self) -> Result<Vec<(String, String)>> {
        let mut workflows = Vec::new();
        if !self.workflows_dir.exists() {
            return Ok(workflows);
        }
        for entry in fs::read_dir(&self.workflows_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "ps1") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    let description = self.extract_description(&path).unwrap_or_else(|_| "Custom automation tool.".to_string());
                    workflows.push((name.to_string(), description));
                }
            }
        }
        Ok(workflows)
    }

    fn extract_description(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)?;
        for line in content.lines().take(10) {
            let trimmed = line.trim();
            if trimmed.to_lowercase().starts_with("# description:") {
                return Ok(trimmed[14..].trim().to_string());
            }
            if trimmed.starts_with("# ") && !trimmed.contains("Description:") && trimmed.len() > 10 {
                 // Fallback: use first long comment line if no formal description found
                 return Ok(trimmed[2..].trim().to_string());
            }
        }
        Ok("Custom automation tool.".to_string())
    }

    pub fn generate_workflow(&self, description: &str, model: &dyn CodexModel) -> Result<String> {
        let prompt = format!(
            "You are an AI assistant capable of writing self-extending tools. \
            The user has requested a new automated workflow tool with the following description:\n\n\
            \"{}\"\n\n\
            Since the user is likely on Windows, write a standalone PowerShell context script (.ps1) \
            that permanently serves this function. It should be highly reliable, well commented, and \
            capable of running in the background if necessary (e.g. for a watcher).\n\n\
            IMPORTANT: Prepend a single line comment at the top: `# Description: <one-line summary of what this tool does>`\n\n\
            Return ONLY the raw PowerShell script code. Do not include markdown fences (```ps1), \
            explanations, or any extra text. Just the clear script code.",
            description
        );

        let code_raw = model.complete(&prompt)?;
        let code = Self::strip_markdown_fences(&code_raw);

        // Derive a clean, snake_case name for the workflow
        let name_prompt = format!(
            "Based on the following script description: \"{}\"\n\n\
            Provide a short, lower_snake_case filename for this tool (e.g. auto_commit_watcher). \
            Return ONLY the name, nothing else.",
            description
        );
        let name_raw = model.complete(&name_prompt).unwrap_or_else(|_| "custom_tool".to_string());
        let mut name = Self::strip_markdown_fences(&name_raw).to_ascii_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        if name.is_empty() {
            name = format!("workflow_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
        }

        let file_path = self.workflows_dir.join(format!("{}.ps1", name));
        fs::write(&file_path, code.as_bytes())?;

        Ok(format!("Generated workflow '{}' securely saved to {:?}", name, file_path))
    }

    pub fn execute_workflow(&self, name: &str, args: Vec<String>) -> Result<String> {
        let file_path = self.workflows_dir.join(format!("{}.ps1", name));
        let exists = file_path.exists();
        
        if !exists {
            return Ok(format!("Workflow '{}' not found in .astra/workflows/", name));
        }

        // We run PowerShell explicitly
        let mut command = Command::new("powershell");
        command.arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&file_path);

        // Append any extra arguments
        for arg in args {
            command.arg(arg);
        }

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // If it's meant to be a watcher, it might keep running in the background. 
        // We'll give it a tiny delay to catch immediate startup errors, then return.
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    Ok(format!("Workflow '{}' completed successfully.", name))
                } else {
                    Ok(format!("Workflow '{}' exited with status {}.", name, status))
                }
            }
            Ok(None) => {
                Ok(format!("Workflow '{}' is now running in the background successfully.", name))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to interact with workflow process: {}", e))
            }
        }
    }

    fn strip_markdown_fences(text: &str) -> String {
        let mut s = text.trim().to_string();
        if s.starts_with("```") {
            if let Some(pos) = s.find('\n') {
                s = s[pos + 1..].to_string();
            }
        }
        if s.ends_with("```") {
            if let Some(pos) = s.rfind("```") {
                s = s[..pos].to_string();
            }
        }
        s.trim().to_string()
    }
}
