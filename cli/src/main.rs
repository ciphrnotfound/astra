use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use codex_core::engine::CodexEngine;
use codex_core::migrate::detect::Language;
use codex_core::migrate::orchestrate::MigrationConfig;
use codex_core::migrate;
use codex_core::model::GroqModel;
use codex_core::teams::TeamManager;

#[derive(Parser)]
#[command(name = "codex")]
#[command(about = "A conversational CLI for understanding and migrating your codebase")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Free-form prompt to run as a one-shot command
    prompt: Vec<String>,

    /// Root directory of the project
    #[arg(long)]
    root: Option<PathBuf>,

    /// Enable the Groq language model
    #[arg(long)]
    use_groq: bool,

    /// Groq model name override
    #[arg(long)]
    groq_model: Option<String>,

    /// Path to .env file for API keys
    #[arg(long)]
    env: Option<PathBuf>,

    // ── Migration flags ──────────────────────────────────────────────

    /// Migrate a source directory: --migrate <source-dir>
    #[arg(long)]
    migrate: Option<PathBuf>,

    /// Source language: --from ts|py|go|rs|java|js
    #[arg(long)]
    from: Option<String>,

    /// Target language: --to ts|py|go|rs|java|js
    #[arg(long)]
    to: Option<String>,

    /// Output directory for migrated code
    #[arg(long)]
    output: Option<PathBuf>,

    /// Use AI-assisted translation during migration
    #[arg(long)]
    ai: bool,

    // ── Quick-action flags ───────────────────────────────────────────

    /// Index the codebase
    #[arg(long)]
    index: bool,

    /// Show codebase summary
    #[arg(long)]
    summary: bool,

    /// Show memory
    #[arg(long)]
    memory: bool,

    /// Show files grouped by language
    #[arg(long, name = "files-by-lang")]
    files_by_lang: bool,

    /// Show codebase health dashboard
    #[arg(long)]
    health: bool,

    /// Time travel debugging: find the commit that introduced a bug
    #[arg(long)]
    bisect: Option<String>,

    /// Scan codebase for security vulnerabilities
    #[arg(long)]
    security_scan: bool,

    /// Switch persona vibe (e.g., nigerian-pidgin, professional, brutal)
    #[arg(long)]
    vibe: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Codex Teams: task assignment and productivity tracking
    Team {
        #[command(subcommand)]
        action: TeamAction,
    },
}

#[derive(Subcommand)]
enum TeamAction {
    /// Initialize a new team for this project
    Init { name: String, api_key: String },
    /// Admin: Assign a task to a developer
    Assign {
        task_id: String,
        developer: String,
        description: String,
    },
    /// Developer: Start working on a task (starts timer & saves git state)
    Start { task_id: String, developer: String },
    /// Developer: Finish a task (stops timer & diffs code changes)
    Finish { task_id: String, developer: String },
    /// Admin: Generate an end-of-week productivity report
    Report,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(env_path) = args.env.clone() {
        load_env_file(env_path);
    }
    let root = args
        .root
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let mut engine = CodexEngine::with_root(root.clone());

    if let Some(vibe_name) = &args.vibe {
        engine.set_persona(codex_core::persona::Persona::from_vibe(vibe_name));
    }

    let persona = codex_core::persona::Persona::load(&root);

    if let Some(byok_key) = persona.api_key.clone() {
        std::env::set_var("GROQ_API_KEY", &byok_key);
        let groq = GroqModel::from_env(persona.model.clone())?;
        engine.set_model(Box::new(groq));
        println!(
            "Using persona BYOK model {} for LLM features.",
            persona
                .model
                .unwrap_or_else(|| "llama-3.1-8b-instant".to_string())
        );
    } else if args.use_groq || std::env::var("GROQ_API_KEY").is_ok() {
        let groq = GroqModel::from_env(args.groq_model.clone())?;
        engine.set_model(Box::new(groq));
    }

    // ── Handle Team Subcommand ───────────────────────────────────────
    if let Some(Commands::Team { action }) = args.command {
        let team_mgr = TeamManager::new(&root);
        match action {
            TeamAction::Init { name, api_key } => {
                team_mgr.init_team(&name, &api_key)?;
                println!("✅ Team '{}' initialized successfully.", name);
            }
            TeamAction::Assign {
                task_id,
                developer,
                description,
            } => {
                team_mgr.assign_task(&task_id, &description, &developer)?;
                println!(
                    "📌 Task '{}' assigned to {}: {}",
                    task_id, developer, description
                );
            }
            TeamAction::Start { task_id, developer } => {
                team_mgr.start_task(&task_id, &developer)?;
                println!(
                    "🚀 {} started working on task '{}'. Timer and Git tracking active.",
                    developer, task_id
                );
            }
            TeamAction::Finish { task_id, developer } => {
                let session = team_mgr.finish_task(&task_id, &developer)?;
                let duration = session.end_time.unwrap_or(0).saturating_sub(session.start_time);
                let hours = duration / 3600;
                let mins = (duration % 3600) / 60;
                println!("✅ Task '{}' completed by {}.", task_id, developer);
                println!(
                    "📊 Time logged: {}h {}m. Code changed: +{} -{} lines.",
                    hours, mins, session.lines_added, session.lines_deleted
                );
            }
            TeamAction::Report => {
                let report = team_mgr.generate_report()?;
                println!("{}", report);
            }
        }
        return Ok(());
    }

    // ── Implicit Team Tracking ───────────────────────────────────────
    let team_mgr = TeamManager::new(&root);
    let mut implicit_session = false;
    let mut current_user = String::new();
    let mut implicit_task_id = String::new();
    
    // If a team is initialized, start tracking implicitly
    if let Ok(state) = team_mgr.load_state() {
        if !state.team_name.is_empty() {
            let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "local_dev".to_string());
            current_user = user.clone();
            
            // Auto-assign and start a generic "Codex Usage" task if they aren't already working on one
            if !state.sessions.iter().any(|s| s.developer == user && s.end_time.is_none()) {
                implicit_task_id = format!("auto-session-{}", std::process::id());
                let _ = team_mgr.assign_task(&implicit_task_id, "Implicit Codex Usage", &user);
                if team_mgr.start_task(&implicit_task_id, &user).is_ok() {
                    implicit_session = true;
                    println!("⏱️  [Codex Teams] Implicit time tracking started for {}.", user);
                }
            }
        }
    }

    let result = run_cli_logic(args, engine);

    if implicit_session {
        // Find the active implicit task and finish it
        if let Ok(state) = team_mgr.load_state() {
            if let Some(session) = state.sessions.iter().find(|s| s.developer == current_user && s.end_time.is_none() && s.task_id == implicit_task_id) {
                if let Ok(finished) = team_mgr.finish_task(&session.task_id, &current_user) {
                    let dur = finished.end_time.unwrap_or(0).saturating_sub(finished.start_time);
                    println!("\n⏱️  [Codex Teams] Session ended. Time logged: {}s. Code changed: +{} -{} lines.", 
                        dur, finished.lines_added, finished.lines_deleted);
                }
            }
        }
    }

    result
}

fn run_cli_logic(args: Args, mut engine: CodexEngine) -> Result<()> {
    // ── Handle --migrate ─────────────────────────────────────────────
    if let Some(source_dir) = args.migrate {
        let from_str = args
            .from
            .as_deref()
            .unwrap_or_else(|| {
                eprintln!("Error: --from <language> is required with --migrate");
                std::process::exit(1);
            });
        let to_str = args
            .to
            .as_deref()
            .unwrap_or_else(|| {
                eprintln!("Error: --to <language> is required with --migrate");
                std::process::exit(1);
            });
        let output_dir = args
            .output
            .unwrap_or_else(|| {
                eprintln!("Error: --output <dir> is required with --migrate");
                std::process::exit(1);
            });

        let from_lang = Language::from_str_loose(from_str).unwrap_or_else(|| {
            eprintln!("Unknown source language: {}", from_str);
            std::process::exit(1);
        });
        let to_lang = Language::from_str_loose(to_str).unwrap_or_else(|| {
            eprintln!("Unknown target language: {}", to_str);
            std::process::exit(1);
        });

        let config = MigrationConfig {
            source_dir,
            output_dir,
            from_lang,
            to_lang,
            use_ai: args.ai,
        };

        let mut model_holder: Option<GroqModel> = None;
        if args.use_groq && args.ai {
            model_holder = Some(GroqModel::from_env(args.groq_model.clone())?);
        }
        let model_ref: Option<&(dyn codex_core::model::CodexModel + Send + Sync)> =
            model_holder.as_ref().map(|m| m as &(dyn codex_core::model::CodexModel + Send + Sync));

        println!("codex ▸ Planning migration {} → {} ...", from_lang, to_lang);
        let result = migrate::run_migration(&config, model_ref)?;

        println!("{}", result.plan_text);
        println!();
        println!("codex ▸ Executing migration ...");
        println!("{}", result.scaffold_log);
        println!("{}", result.summary());
        return Ok(());
    }

    // ── Handle quick-action flags ────────────────────────────────────
    if args.index {
        let response = engine.handle_input(":index")?;
        println!("{}", response);
        return Ok(());
    }
    if args.summary {
        let response = engine.handle_input(":summary")?;
        println!("{}", response);
        return Ok(());
    }
    if args.memory {
        let response = engine.handle_input(":memory")?;
        println!("{}", response);
        return Ok(());
    }
    if args.files_by_lang {
        let response = engine.handle_input(":files-by-lang")?;
        println!("{}", response);
        return Ok(());
    }
    if args.health {
        let response = engine.handle_input(":health")?;
        println!("{}", response);
        return Ok(());
    }
    if let Some(desc) = args.bisect {
        let cmd = format!(":bisect {}", desc);
        let response = engine.handle_input(&cmd)?;
        println!("{}", response);
        return Ok(());
    }
    if args.security_scan {
        let response = engine.handle_input(":security-scan")?;
        println!("{}", response);
        return Ok(());
    }

    // ── Handle free-form prompt ──────────────────────────────────────
    if !args.prompt.is_empty() {
        let prompt = args.prompt.join(" ");
        let response = engine.handle_input(&prompt)?;
        println!("{}", response);
        return Ok(());
    }

    run_repl(&mut engine)
}

fn load_env_file(path: PathBuf) {
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            std::env::set_var(key.trim(), value.trim());
        }
    }
}

fn run_repl(engine: &mut CodexEngine) -> Result<()> {
    println!("────────────────────────────────────────");
    println!(" codex v0.1.0 — conversational CLI");
    println!("────────────────────────────────────────");
    println!("Type what you want, like:");
    println!("  migrate core/src from rs to ts into ./out --ai");
    println!("  show project summary");
    println!("  what do you remember about this repo?");
    println!();

    let mut input = String::new();

    loop {
        input.clear();
        print!("\n› ");
        io::stdout().flush()?;

        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        let reply = engine.handle_input(trimmed)?;
        println!("𝙘𝙤𝙙𝙚𝙭 › {}", reply);
    }

    Ok(())
}
