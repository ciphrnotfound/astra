use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::model::CodexModel;
use crate::memory::{MemoryStore, MemoryEvent, LearningPhase};
use crate::migrate::detect::Language;

pub struct LearningManager<'a> {
    pub model: &'a (dyn CodexModel + Send + Sync),
    pub memory: &'a mut MemoryStore,
    pub root: PathBuf,
}

impl<'a> LearningManager<'a> {
    pub fn new(model: &'a (dyn CodexModel + Send + Sync), memory: &'a mut MemoryStore, root: PathBuf) -> Self {
        Self { model, memory, root }
    }

    pub fn start_learning(&mut self, language: Language, why: &str, how: &str) -> Result<String> {
        let learning_dir = self.root.join("learning").join(language.to_string().to_lowercase());
        fs::create_dir_all(&learning_dir)?;

        let phase1_dir = learning_dir.join("phase1");
        fs::create_dir_all(&phase1_dir)?;

        let phase = LearningPhase {
            language: language.to_string(),
            phase_number: 1,
            goal: format!("Learn the basics of {}: syntax, variables, and simple functions.", language),
            path: phase1_dir.to_string_lossy().to_string(),
            proficiency_notes: format!("Started learning {} because: {}", language, why),
        };

        // Create initial lesson file
        let lesson_path = phase1_dir.join("README.md");
        let lesson_content = format!(
            "# Learning {}: Phase 1\n\n\
            Welcome to your first phase of learning {}!\n\n\
            **Goal**: {}\n\n\
            **Your Task**: Create a simple file in this directory that demonstrates basic syntax.\n\n\
            _Astra is watching this folder. When you're ready, ask me to review your work!_",
            language, language, phase.goal
        );
        fs::write(lesson_path, lesson_content)?;

        self.memory.add_event(
            "learning",
            format!("Started learning {} (Phase 1)", language),
            MemoryEvent::LearningProgress { phase },
        );

        Ok(format!(
            "Excellent! I've set up a learning environment for you in `{}`.\n\n\
            I'll guide you through Phase 1: Basics. Check the README in that folder to get started.\n\n\
            How you want to learn: {}",
            phase1_dir.display(),
            how
        ))
    }

    pub fn evaluate_progress(&mut self) -> Result<String> {
        let current_phase = match self.memory.latest_event("learning") {
            Some(entry) => match &entry.event {
                Some(MemoryEvent::LearningProgress { phase }) => phase.clone(),
                _ => return Ok("You aren't currently in a learning phase. Say `I want to learn [language]` to start!".to_string()),
            },
            None => return Ok("You aren't currently in a learning phase. Say `I want to learn [language]` to start!".to_string()),
        };

        let phase_path = Path::new(&current_phase.path);
        if !phase_path.exists() {
            return Ok("It looks like your learning directory was moved or deleted.".to_string());
        }

        // Read files in the phase directory
        let mut files_content = String::new();
        for entry in fs::read_dir(phase_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e != "md").unwrap_or(true) {
                let content = fs::read_to_string(&path)?;
                files_content.push_str(&format!("\nFile: {:?}\n---\n{}\n---\n", path.file_name().unwrap(), content));
            }
        }

        if files_content.is_empty() {
            return Ok("I don't see any code files in your phase directory yet. Keep working on it!".to_string());
        }

        let prompt = format!(
            "You are a programming tutor. Evaluate the following code for a student learning {}.\n\n\
            Current Phase: {}\n\
            Phase Goal: {}\n\n\
            Student's Code:\n{}\n\n\
            If the code covers the goal well, output 'PASS' followed by a brief encouraging review.\n\
            If not, output 'FEEDBACK' followed by specific improvements needed.",
            current_phase.language, current_phase.phase_number, current_phase.goal, files_content
        );

        let evaluation = self.model.complete(&prompt)?;

        if evaluation.starts_with("PASS") {
            let next_phase_num = current_phase.phase_number + 1;
            let next_phase_dir = Path::new(&current_phase.path).parent().unwrap().join(format!("phase{}", next_phase_num));
            fs::create_dir_all(&next_phase_dir)?;

            let next_phase = LearningPhase {
                language: current_phase.language.clone(),
                phase_number: next_phase_num,
                goal: format!("Intermediate {} concepts: loops, collections, and error handling.", current_phase.language),
                path: next_phase_dir.to_string_lossy().to_string(),
                proficiency_notes: format!("Passed Phase {}. Review: {}", current_phase.phase_number, evaluation),
            };

            let lesson_path = next_phase_dir.join("README.md");
            let lesson_content = format!(
                "# Learning {}: Phase {}\n\n\
                Great job passing Phase {}!\n\n\
                **Goal**: {}\n\n\
                **Your Task**: Implement a small program that uses collections and handles errors.\n\n\
                _Ask me to review whenever you're ready to move to Phase {}!_",
                next_phase.language, next_phase.phase_number, current_phase.phase_number, next_phase.goal, next_phase_num + 1
            );
            fs::write(lesson_path, lesson_content)?;

            self.memory.add_event(
                "learning",
                format!("Advanced to Phase {} in {}", next_phase_num, current_phase.language),
                MemoryEvent::LearningProgress { phase: next_phase },
            );

            return Ok(format!(
                "🎉 **Phase {} Passed!**\n\n{}\n\nI've unlocked **Phase {}** for you in `{}`. Check the new README for your next challenge!",
                current_phase.phase_number,
                evaluation.trim_start_matches("PASS").trim(),
                next_phase_num,
                next_phase_dir.display()
            ));
        } else {
            return Ok(format!(
                "📝 **Review for Phase {}**\n\n{}",
                current_phase.phase_number,
                evaluation.trim_start_matches("FEEDBACK").trim()
            ));
        }
    }
}
