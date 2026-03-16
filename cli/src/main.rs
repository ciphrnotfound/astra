use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use astra_core::engine::CodexEngine;
use astra_core::migrate::detect::Language;
use astra_core::migrate::orchestrate::MigrationConfig;
use astra_core::migrate;
use astra_core::model::{CodexModel, GroqModel, OllamaModel, TavilySearch};
use astra_core::teams::{TeamManager, TeamRole};

use crossterm::style::Stylize;

#[derive(Parser)]
#[command(name = "astra")]
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

    /// Enable the Ollama language model
    #[arg(long)]
    use_ollama: bool,

    /// Ollama model name override
    #[arg(long)]
    ollama_model: Option<String>,

    /// Ollama API endpoint override
    #[arg(long)]
    ollama_url: Option<String>,

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

    /// Clean up migrated code using semantic cleanup engine
    #[arg(long)]
    clean: bool,

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

    /// Run predictive refactoring analysis
    #[arg(long)]
    predict: bool,

    /// Install git pre-commit hook
    #[arg(long)]
    hook: bool,

    /// Start watch mode (monitors file changes in real time)
    #[arg(long)]
    watch: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Astra Teams: task assignment and productivity tracking
    Team {
        #[command(subcommand)]
        action: TeamAction,
    },
}

#[derive(Subcommand)]
enum TeamAction {
    /// Initialize a new team for this project
    Init { name: String, admin_key: Option<String> },
    /// Admin: Add a team member with a role and key
    AddMember {
        name: String,
        role: String,
        admin_key: String,
        member_key: Option<String>,
    },
    /// Admin: Assign a task to a developer
    Assign {
        task_id: String,
        developer: String,
        description: String,
        admin_key: String,
    },
    /// Developer: Start working on a task (starts timer & saves git state)
    Start {
        task_id: String,
        developer: String,
        member_key: String,
    },
    /// Developer: Finish a task (stops timer & diffs code changes)
    Finish {
        task_id: String,
        developer: String,
        member_key: String,
    },
    /// Admin: Generate an end-of-week productivity report
    Report { admin_key: String },
    /// Sync team state with the distributed Git branch (astra-state)
    Sync,
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
        engine.set_persona(astra_core::persona::Persona::from_vibe(vibe_name));
    }

    let persona = astra_core::persona::Persona::load(&root);

    if let Some(byok_key) = persona.api_key.clone() {
        std::env::set_var("GROQ_API_KEY", &byok_key);
        let groq = GroqModel::from_env(persona.model.clone())?;
        engine.set_model(Box::new(groq));
        println!(
            "Using persona BYOK model {} for LLM features.",
            persona
                .model
                .clone()
                .unwrap_or_else(|| "llama-3.1-8b-instant".to_string())
        );
    } else if let Some(ollama_url) = std::env::var("OLLAMA_URL").ok().or(args.ollama_url.clone()) {
        let model = args.ollama_model.clone().or_else(|| std::env::var("OLLAMA_MODEL").ok());
        let ollama = OllamaModel::from_env(model, Some(ollama_url))?;
        engine.set_model(Box::new(ollama));
    } else if args.use_groq || std::env::var("GROQ_API_KEY").is_ok() {
        let model_name = args.groq_model.clone().or_else(|| persona.model.clone());
        let groq = GroqModel::from_env(model_name)?;
        engine.set_model(Box::new(groq));
    }

    if std::env::var("TAVILY_API_KEY").is_ok() {
        if let Ok(search) = TavilySearch::from_env() {
            engine.set_search(Box::new(search));
        }
    }

    // ── Handle Team Subcommand ───────────────────────────────────────
    if let Some(Commands::Team { action }) = args.command {
        let team_mgr = TeamManager::new(&root);
        match action {
            TeamAction::Init { name, admin_key } => {
                let key = team_mgr.init_team(&name, admin_key.as_deref())?;
                println!("✅ Team '{}' initialized successfully.", name);
                println!("🔑 Admin key: {}", key);
            }
            TeamAction::AddMember {
                name,
                role,
                member_key,
                admin_key,
            } => {
                let parsed_role = parse_team_role(&role)?;
                let key = team_mgr.add_member(
                    &admin_key,
                    &name,
                    parsed_role,
                    member_key.as_deref(),
                )?;
                println!("✅ Added member '{}' with role {}.", name, role);
                println!("🔑 Member key: {}", key);
            }
            TeamAction::Assign {
                task_id,
                developer,
                description,
                admin_key,
            } => {
                team_mgr.assign_task(&admin_key, &task_id, &description, &developer)?;
                println!(
                    "📌 Task '{}' assigned to {}: {}",
                    task_id, developer, description
                );
            }
            TeamAction::Start {
                task_id,
                developer,
                member_key,
            } => {
                team_mgr.start_task(&member_key, &task_id, &developer)?;
                println!(
                    "🚀 {} started working on task '{}'. Timer and Git tracking active.",
                    developer, task_id
                );
            }
            TeamAction::Finish {
                task_id,
                developer,
                member_key,
            } => {
                let session = team_mgr.finish_task(&member_key, &task_id, &developer)?;
                let duration = session.end_time.unwrap_or(0).saturating_sub(session.start_time);
                let hours = duration / 3600;
                let mins = (duration % 3600) / 60;
                println!("✅ Task '{}' completed by {}.", task_id, developer);
                println!(
                    "📊 Time logged: {}h {}m. Code changed: +{} -{} lines.",
                    hours, mins, session.lines_added, session.lines_deleted
                );
            }
            TeamAction::Report { admin_key } => {
                let report = team_mgr.generate_report(&admin_key)?;
                println!("{}", report);
            }
            TeamAction::Sync => {
                team_mgr.sync()?;
                println!("✅ Team state synchronized with origin/astra-state successfully.");
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
        if !state.team_name.is_empty() && state.members.is_empty() {
            let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "local_dev".to_string());
            current_user = user.clone();
            
            // Auto-assign and start a generic "Astra Usage" task if they aren't already working on one
            if !state.sessions.iter().any(|s| s.developer == user && s.end_time.is_none()) {
                implicit_task_id = format!("auto-session-{}", std::process::id());
                let _ = team_mgr.assign_task_implicit(&implicit_task_id, "Implicit Astra Usage", &user);
                if team_mgr.start_task_implicit(&implicit_task_id, &user).is_ok() {
                    implicit_session = true;
                    println!("⏱️  [Astra Teams] Implicit time tracking started for {}.", user);
                }
            }
        }
    }

    let result = run_cli_logic(args, engine, persona.clone());

    if implicit_session {
        // Find the active implicit task and finish it
        if let Ok(state) = team_mgr.load_state() {
            if let Some(session) = state.sessions.iter().find(|s| s.developer == current_user && s.end_time.is_none() && s.task_id == implicit_task_id) {
                if let Ok(finished) = team_mgr.finish_task_implicit(&session.task_id, &current_user) {
                    let dur = finished.end_time.unwrap_or(0).saturating_sub(finished.start_time);
                    println!("\n⏱️  [Astra Teams] Session ended. Time logged: {}s. Code changed: +{} -{} lines.", 
                        dur, finished.lines_added, finished.lines_deleted);
                }
            }
        }
    }

    result
}

enum MigrationModel {
    Groq(GroqModel),
    Ollama(OllamaModel),
}

fn run_cli_logic(args: Args, mut engine: CodexEngine, persona: astra_core::persona::Persona) -> Result<()> {
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

        // Phase 6: Pre-migration research
        let mut knowledge = None;
        if args.ai && engine.has_search() {
             println!("Researching latest {} syntax and best practices...", to_lang);
             knowledge = engine.research_language(to_lang).ok();
        }

        let config = MigrationConfig {
            source_dir,
            output_dir,
            from_lang,
            to_lang,
            use_ai: args.ai,
            use_clean: args.clean,
            knowledge,
        };

        let mut model_holder: Option<MigrationModel> = None;
        if args.ai {
            if args.use_ollama
                || std::env::var("OLLAMA_URL").is_ok()
                || std::env::var("OLLAMA_MODEL").is_ok()
            {
                model_holder = Some(MigrationModel::Ollama(OllamaModel::from_env(
                    args.ollama_model.clone(),
                    args.ollama_url.clone(),
                )?));
            } else {
                let model_name = args.groq_model.clone().or(persona.model.clone());
                model_holder = Some(MigrationModel::Groq(GroqModel::from_env(model_name)?));
            }
        }
        let model_ref: Option<&(dyn CodexModel + Send + Sync)> = model_holder.as_ref().map(|m| {
            match m {
                MigrationModel::Groq(g) => g as &(dyn CodexModel + Send + Sync),
                MigrationModel::Ollama(o) => o as &(dyn CodexModel + Send + Sync),
            }
        });

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
    if args.predict {
        let response = engine.handle_input(":predict")?;
        println!("{}", response);
        return Ok(());
    }
    if args.hook {
        let response = engine.handle_input(":hook")?;
        println!("{}", response);
        return Ok(());
    }
    if args.watch {
        let root = args.root.clone().unwrap_or_else(|| PathBuf::from("."));
        println!("\u{1f440} Astra Watch Mode \u{2014} monitoring {} for changes...", root.display());
        println!("Press Ctrl+C to stop.\n");

        // Index first
        let _ = engine.handle_input(":index");

        match astra_core::watch::start_watcher(&root) {
            Ok((_watcher, rx)) => {
                loop {
                    match rx.recv() {
                        Ok(alert) => {
                            println!("{}", alert);
                            let warnings = astra_core::watch::handle_file_change(
                                engine.index_mut(),
                                &alert,
                            );
                            for w in warnings {
                                println!("{}", w);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(e) => {
                eprintln!("\u{274c} Failed to start watcher: {}", e);
            }
        }
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

fn parse_team_role(role: &str) -> Result<TeamRole> {
    match role.to_ascii_lowercase().as_str() {
        "admin" => Ok(TeamRole::Admin),
        "member" => Ok(TeamRole::Member),
        _ => Err(anyhow::anyhow!("Unknown role: {}", role)),
    }
}

fn run_repl(engine: &mut CodexEngine) -> Result<()> {
    println!("{}", "╭──────────────────────────────────────────────────╮".dark_grey());
    println!("{} {} {}", "│".dark_grey(), "🤖 Astra v0.1.0".cyan().bold(), "— Your codebase companion      │".dark_grey());
    println!("{}", "╰──────────────────────────────────────────────────╯".dark_grey());
    println!("{}", " Try `:index` to build the semantic graph, or ask:".dark_grey());
    println!("{}", "  ? what does this project do?".dark_grey());
    println!("{}", "  migrate core/src from rs to ts into ./out".dark_grey());
    println!();

    let mut input = String::new();

    loop {
        input.clear();
        print!("{} ", " You ❯".green().bold());
        io::stdout().flush()?;

        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        match engine.handle_input(trimmed) {
            Ok(reply) => {
                println!("\n{} {}\n", " Astra ❯".magenta().bold(), reply);
            }
            Err(e) => {
                eprintln!("\n{} {}\n", " ❌ Error ❯".red().bold(), e);
            }
        }
    }

    Ok(())
}
