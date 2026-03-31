use std::fs;
use std::io::{self, Write};
use std::io::{Read, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::{Parser, Subcommand};
use astra_core::engine::CodexEngine;
use astra_core::migrate::detect::Language;
use astra_core::migrate::orchestrate::MigrationConfig;
use astra_core::migrate;
use astra_core::model::{CodexModel, GroqModel, OllamaModel, GeminiModel, OpenRouterModel, TavilySearch};
use astra_core::teams::{TeamManager, TeamRole};
use astra_core::tracker::SessionTracker;

use crossterm::style::Stylize;

#[derive(Parser)]
#[command(name = "astra")]
#[command(about = "A conversational CLI for understanding and migrating your codebase")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Free-form prompt to run as a one-shot command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
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

    /// Enable the Gemini language model
    #[arg(long)]
    use_gemini: bool,

    /// Gemini model name override
    #[arg(long)]
    gemini_model: Option<String>,

    /// Enable the OpenRouter unified API
    #[arg(long)]
    use_openrouter: bool,

    /// OpenRouter model name override
    #[arg(long)]
    openrouter_model: Option<String>,

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
    pub clean: bool,

    /// Auto-fix compiler errors after migration
    #[arg(long)]
    pub fix: bool,

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

    /// Start Model Context Protocol server for Cursor / Claude
    #[arg(long)]
    mcp: bool,

    /// Enable agentic mode: the AI can autonomously read/write files, run commands, etc.
    #[arg(long)]
    agent: bool,

    /// Auto-approve all agent actions (file writes, edits, command execution) without prompting.
    #[arg(long, name = "auto-approve")]
    auto_approve: bool,
}

#[derive(Clone, Subcommand)]
enum Commands {
    /// Astra Teams: task assignment and productivity tracking
    Team {
        #[command(subcommand)]
        action: TeamAction,
    },
    /// Manage global Astra configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Authentication/session commands for Astra CLI
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Clone, Subcommand)]
enum ConfigAction {
    /// Set a configuration value (e.g. config set user "Jeremy")
    Set {
        key: String,
        value: String,
    },
}

#[derive(Clone, Subcommand)]
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
    Sync {
        #[arg(long)]
        cloud: bool,
    },
    /// Show current team context for the signed-in user
    Status,
}

#[derive(Clone, Subcommand)]
enum AuthAction {
    /// Sign in to Astra locally (stores profile + session token)
    Login {
        user: String,
        #[arg(long, default_value = "local")]
        provider: String,
    },
    /// Open a browser form and auto sign-in from terminal callback
    LoginWeb {
        #[arg(long, default_value = "local")]
        provider: String,
    },
    /// Sign out and clear session token
    Logout,
    /// Show current auth session
    Status,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(env_path) = args.env.clone() {
        load_env_file(env_path);
    } else {
        let default_env = PathBuf::from(".env");
        if default_env.exists() {
            load_env_file(default_env);
        }
    }
    let root = args
        .root
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let mut engine = CodexEngine::with_root(root.clone());

    // Enable agent mode if --agent flag is passed
    if args.agent {
        engine.set_agent_mode(true);
    }
    if args.auto_approve {
        engine.set_auto_approve(true);
    }

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
    } else if args.use_gemini || std::env::var("GEMINI_API_KEY").is_ok() {
        let model_name = args.gemini_model.clone().or_else(|| persona.model.clone());
        let gemini = GeminiModel::from_env(model_name)?;
        engine.set_model(Box::new(gemini));
    } else if args.use_openrouter || std::env::var("OPENROUTER_API_KEY").is_ok() {
        let model_name = args.openrouter_model.clone().or_else(|| persona.model.clone());
        let openrouter = OpenRouterModel::from_env(model_name)?;
        engine.set_model(Box::new(openrouter));
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

    // ── Handle Config Subcommand ─────────────────────────────────────
    if let Some(Commands::Config { action }) = args.command.clone() {
        match action {
            ConfigAction::Set { key, value } => {
                let mut cfg = astra_core::config::load_global_config();
                match key.as_str() {
                    "user" => {
                        cfg.user = Some(value.clone());
                        println!("✅ Set global user to '{}'", value);
                    }
                    _ => {
                        eprintln!("❌ Unknown config key: {}", key);
                        std::process::exit(1);
                    }
                }
                if let Err(e) = astra_core::config::save_global_config(&cfg) {
                    eprintln!("❌ Failed to save config: {}", e);
                }
            }
        }
        return Ok(());
    }

    if let Some(Commands::Auth { action }) = args.command.clone() {
        let mut cfg = astra_core::config::load_global_config();
        match action {
            AuthAction::Login { user, provider } => {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let token = format!("astra_{}_{}", user.replace(' ', "_"), ts);
                cfg.user = Some(user.clone());
                cfg.auth_user = Some(user.clone());
                cfg.auth_provider = Some(provider.clone());
                cfg.auth_token = Some(token.clone());
                astra_core::config::save_global_config(&cfg)?;
                println!("✅ Signed in as '{}' via {}.", user, provider);
                println!("🔐 Session token: {}", token);
                if let Err(e) = astra_core::supabase::sync_auth_profile(&user, &provider) {
                    eprintln!("⚠️ Auth profile was saved locally but cloud sync failed: {}", e);
                }
            }
            AuthAction::LoginWeb { provider } => {
                let user = run_web_login_flow()?;
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let token = format!("astra_{}_{}", user.replace(' ', "_"), ts);
                cfg.user = Some(user.clone());
                cfg.auth_user = Some(user.clone());
                cfg.auth_provider = Some(provider.clone());
                cfg.auth_token = Some(token.clone());
                astra_core::config::save_global_config(&cfg)?;
                println!("✅ Signed in as '{}' via {}.", user, provider);
                println!("🔐 Session token: {}", token);
                if let Err(e) = astra_core::supabase::sync_auth_profile(&user, &provider) {
                    eprintln!("⚠️ Auth profile was saved locally but cloud sync failed: {}", e);
                }
            }
            AuthAction::Logout => {
                cfg.auth_token = None;
                cfg.auth_provider = None;
                cfg.auth_user = None;
                astra_core::config::save_global_config(&cfg)?;
                println!("✅ Signed out.");
            }
            AuthAction::Status => {
                let user = cfg
                    .auth_user
                    .clone()
                    .or(cfg.user.clone())
                    .unwrap_or_else(|| "not set".to_string());
                let provider = cfg
                    .auth_provider
                    .clone()
                    .unwrap_or_else(|| "none".to_string());
                let has_token = cfg.auth_token.is_some();
                println!("👤 User: {}", user);
                println!("🔌 Provider: {}", provider);
                println!("🔐 Session active: {}", has_token);
            }
        }
        return Ok(());
    }

    // ── Handle Team Subcommand ───────────────────────────────────────
    if let Some(Commands::Team { action }) = args.command.clone() {
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
            TeamAction::Sync { cloud } => {
                if cloud {
                    let mut ok = true;
                    if let Err(e) = astra_core::supabase::sync_offline_queue() {
                        ok = false;
                        eprintln!("❌ Failed syncing session queue: {}", e);
                    } else {
                        println!("✅ Session queue synced to Supabase.");
                    }
                    if let Err(e) = astra_core::supabase::sync_team_state(&root) {
                        ok = false;
                        eprintln!("❌ Failed syncing team snapshot: {}", e);
                    } else {
                        println!("✅ Team snapshot synced to Supabase.");
                    }
                    if ok {
                        println!("✅ Cloud sync complete.");
                    }
                } else {
                    team_mgr.sync()?;
                    println!("✅ Team state synchronized with origin/astra-state successfully.");
                }
            }
            TeamAction::Status => {
                print_team_status(&team_mgr)?;
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
            let user = current_user_name();
            current_user = user.clone();
            let is_member = state
                .members
                .keys()
                .any(|name| member_matches_user(name, &user));
            if !is_member {
                let result = run_cli_logic(args, engine, persona.clone(), root.clone());
                return result;
            }
            
            if !state
                .sessions
                .iter()
                .any(|s| member_matches_user(&s.developer, &user) && s.end_time.is_none())
            {
                implicit_task_id = format!("auto-session-{}", std::process::id());
                let _ = team_mgr.assign_task_implicit(&implicit_task_id, "Implicit Astra Usage", &user);
                if team_mgr.start_task_implicit(&implicit_task_id, &user).is_ok() {
                    implicit_session = true;
                    println!("⏱️  [Astra Teams] Implicit time tracking started for {}.", user);
                }
            }
        }
    }

    let result = run_cli_logic(args, engine, persona.clone(), root.clone());

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

fn run_cli_logic(args: Args, mut engine: CodexEngine, persona: astra_core::persona::Persona, project_root: PathBuf) -> Result<()> {
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
            use_fix: args.fix,
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

        let mut search_holder: Option<TavilySearch> = None;
        if args.ai && std::env::var("TAVILY_API_KEY").is_ok() {
            search_holder = TavilySearch::from_env().ok();
        }
        let search_ref: Option<&(dyn astra_core::model::SearchProvider + Send + Sync)> = 
            search_holder.as_ref().map(|s| s as _);

        println!("codex ▸ Planning migration {} → {} ...", from_lang, to_lang);
        let result = migrate::run_migration(&config, model_ref, search_ref)?;

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

    if args.mcp {
        if let Err(e) = astra_core::mcp::run_mcp_server(&mut engine) {
            eprintln!("\u{274c} MCP Server error: {}", e);
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

    run_repl(&mut engine, &project_root)
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

fn current_user_name() -> String {
    let cfg = astra_core::config::load_global_config();
    cfg.user.unwrap_or_else(|| {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "local_dev".to_string())
    })
}

fn run_web_login_flow() -> Result<String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let addr = listener.local_addr()?;
    let url = format!("http://127.0.0.1:{}/", addr.port());
    println!("🔐 Open this link to sign in: {}", url);

    let mut captured_user: Option<String> = None;
    for stream in listener.incoming().take(6) {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Some(user) = handle_auth_connection(&mut stream)? {
            captured_user = Some(user);
            break;
        }
    }
    captured_user.ok_or_else(|| anyhow::anyhow!("Login timed out. Please run `astra auth login-web` again."))
}

fn handle_auth_connection(stream: &mut TcpStream) -> Result<Option<String>> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(v) = trimmed.strip_prefix("Content-Length:") {
            content_length = v.trim().parse::<usize>().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body_str = String::from_utf8_lossy(&body);

    if request_line.starts_with("GET / ") {
        let html = r#"<!doctype html><html><head><meta charset="utf-8"><title>Astra Sign In</title></head><body><h2>Astra Sign In</h2><form method="POST" action="/submit"><label>Name <input name="user" required /></label><br/><br/><button type="submit">Sign In</button></form></body></html>"#;
        write_http_response(stream, "200 OK", html, "text/html")?;
        return Ok(None);
    }

    if request_line.starts_with("POST /submit ") {
        let user = form_value(&body_str, "user").unwrap_or_default();
        if user.trim().is_empty() {
            write_http_response(stream, "400 Bad Request", "Missing user.", "text/plain")?;
            return Ok(None);
        }
        write_http_response(stream, "200 OK", "Sign-in complete. You can close this tab and return to terminal.", "text/plain")?;
        return Ok(Some(user));
    }

    write_http_response(stream, "404 Not Found", "Not found", "text/plain")?;
    Ok(None)
}

fn write_http_response(stream: &mut TcpStream, status: &str, body: &str, content_type: &str) -> Result<()> {
    let resp = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    stream.write_all(resp.as_bytes())?;
    Ok(())
}

fn form_value(body: &str, key: &str) -> Option<String> {
    for pair in body.split('&') {
        let (k, v) = pair.split_once('=')?;
        if k == key {
            return Some(url_decode(v));
        }
    }
    None
}

fn url_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &input[i + 1..i + 3];
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    out.push(val as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    out
}

fn member_matches_user(member: &str, user: &str) -> bool {
    let normalize = |value: &str| value.trim().trim_start_matches('@').to_ascii_lowercase();
    normalize(member) == normalize(user)
}

fn print_team_status(team_mgr: &TeamManager) -> Result<()> {
    let state = team_mgr.load_state()?;
    if state.team_name.is_empty() {
        println!("No team is initialized for this repo.");
        return Ok(());
    }
    let user = current_user_name();
    let is_member = state
        .members
        .keys()
        .any(|name| member_matches_user(name, &user));
    println!("👥 Team: {}", state.team_name);
    println!("👤 Current user: {} (member: {})", user, is_member);
    println!("👤 Members: {}", state.members.len());
    let my_open: Vec<_> = state
        .tasks
        .values()
        .filter(|t| {
            member_matches_user(&t.assignee, &user)
                && t.status != astra_core::teams::TaskStatus::Done
        })
        .collect();
    if my_open.is_empty() {
        println!("📌 Open tasks for you: none");
    } else {
        println!("📌 Open tasks for you: {}", my_open.len());
        for task in my_open.iter().take(5) {
            println!("   - [{}] {}", task.id, task.description);
        }
    }
    if let Some(active) = state
        .sessions
        .iter()
        .find(|s| member_matches_user(&s.developer, &user) && s.end_time.is_none())
    {
        println!("⏱️ Active session: {}", active.task_id);
    }
    Ok(())
}

fn run_repl(engine: &mut CodexEngine, root: &Path) -> Result<()> {
    println!("{}", "╭──────────────────────────────────────────────────╮".dark_grey());
    if engine.root().join(".astra").join("memory.json").exists() {
        // Check if agent mode is active
        println!("{} {} {}", "│".dark_grey(), "◈ Astra v0.1.0".white().bold(), "— Your codebase companion      │".dark_grey());
    } else {
        println!("{} {} {}", "│".dark_grey(), "◈ Astra v0.1.0".white().bold(), "— Your codebase companion      │".dark_grey());
    }
    println!("{}", "╰──────────────────────────────────────────────────╯".dark_grey());
    
    // ── Proactive Task Dashboard ────────────────────────────────────
    let team_mgr = TeamManager::new(root);
    if let Ok(state) = team_mgr.load_state() {
        let user = current_user_name();
        
        let my_tasks: Vec<_> = state
            .tasks
            .values()
            .filter(|t| {
                member_matches_user(&t.assignee, &user)
                    && t.status != astra_core::teams::TaskStatus::Done
            })
            .collect();
        
        if !my_tasks.is_empty() {
            println!("\n{} Welcome back, @{}!", "▸".cyan(), user);
            
            if let Some(active) = state
                .sessions
                .iter()
                .find(|s| member_matches_user(&s.developer, &user) && s.end_time.is_none())
            {
                if let Some(task) = state.tasks.get(&active.task_id) {
                    println!("{} You are currently working on: [{}] {}", "▸".cyan(), task.id.clone().yellow(), task.description);
                }
            } else {
                println!("{} You have {} pending tasks. Run `astra team start <task_id>` to begin.", "▸".cyan(), my_tasks.len());
                for task in my_tasks.iter().take(3) {
                    println!("    - [{}]: {}", task.id.clone().yellow(), task.description);
                }
            }
            println!("{}", "──────────────────────────────────────────────────".dark_grey());
        }
    }
    
    // ── Global First-Run Onboarding ──────────────────────────────────
    // This only runs ONCE ever — when ~/.astra/memory.json doesn't exist yet
    use astra_core::memory::MemoryStore;

    if MemoryStore::is_first_run() {
        println!("\n{}", "✨ Welcome to Astra — your AI-powered codebase companion!".yellow().bold());
        println!("{}", "Let me learn a bit about you. This will be saved globally so I remember you everywhere.\n".dark_grey());

        // 1. Name
        print!("{} What's your name? ", " ❯".green().bold());
        io::stdout().flush()?;
        let mut name_input = String::new();
        io::stdin().read_line(&mut name_input)?;
        let name_val = name_input.trim();
        if !name_val.is_empty() {
            engine.memory_mut().add_global("user-identity", format!("name: {}", name_val));
            println!("  {} Nice to meet you, {}!", "✓".green(), name_val);
        }

        // 2. Preferred language
        print!("{} What's your primary programming language? (e.g. rust, python, typescript) ", " ❯".green().bold());
        io::stdout().flush()?;
        let mut lang_input = String::new();
        io::stdin().read_line(&mut lang_input)?;
        let lang_val = lang_input.trim();
        if !lang_val.is_empty() {
            engine.memory_mut().add_global("user-preference", format!("language: {}", lang_val));
            println!("  {} Got it — {} is your go-to.", "✓".green(), lang_val);
        }

        // 3. Vibe / personality
        println!("{} What vibe should I use when talking to you?", " ❯".green().bold());
        println!("    {} professional, casual, nigerian-pidgin, brutal, or just press Enter for default", "Options:".dark_grey());
        print!("{} ", " ❯".green().bold());
        io::stdout().flush()?;
        let mut vibe_input = String::new();
        io::stdin().read_line(&mut vibe_input)?;
        let vibe_val = vibe_input.trim();
        if !vibe_val.is_empty() {
            engine.memory_mut().add_global("user-preference", format!("vibe: {}", vibe_val));
            let persona = astra_core::persona::Persona::from_vibe(vibe_val);
            let _ = persona.save(root);
            engine.set_persona(persona);
            println!("  {} Vibe set to: {}.", "✓".green(), vibe_val);
        }

        // 4. Reasoning depth
        println!("{} How much reasoning should I show?", " ❯".green().bold());
        println!("    {} concise (just answers), balanced (some reasoning), verbose (full chain-of-thought)", "Options:".dark_grey());
        print!("{} ", " ❯".green().bold());
        io::stdout().flush()?;
        let mut reason_input = String::new();
        io::stdin().read_line(&mut reason_input)?;
        let reason_val = reason_input.trim();
        if !reason_val.is_empty() {
            engine.memory_mut().add_global("user-preference", format!("reasoning: {}", reason_val));
            println!("  {} Reasoning level: {}.", "✓".green(), reason_val);
        }

        println!("\n{}", "🧠 Profile saved! I'll remember you across all projects.".magenta().bold());
        println!("{}\n", "──────────────────────────────────────────────────".dark_grey());
    } else if let Some(name) = engine.memory().user_name() {
        // Greet returning user by name
        println!("\n {} Welcome back, {}! 👋\n", "▸".cyan(), name);
    }
    
    // ── Per-Project Onboarding ───────────────────────────────────────
    // Only triggers when the project has no local memory yet
    let astra_file = root.join(".astra").join("memory.json");
    let legacy_file = root.join(".codex").join("memory.json");
    let is_fresh_project = !astra_file.exists() && !legacy_file.exists();

    if is_fresh_project {
        println!("{}", "📁 New project detected! Tell me about it:".yellow());

        print!("{} What is the goal of this project? (or press Enter to skip) ", " ❯".green().bold());
        io::stdout().flush()?;
        let mut goals_input = String::new();
        io::stdin().read_line(&mut goals_input)?;
        let goals_val = goals_input.trim();
        if !goals_val.is_empty() {
            let _ = engine.handle_input(&format!(":learn The core project goal is: {}", goals_val));
            println!("  {} Project goal saved.", "✓".green());
        }
        println!("{}\n", "──────────────────────────────────────────────────".dark_grey());
    }

    println!("{}", " Try `:index` to build the semantic graph, or ask:".dark_grey());
    println!("{}", "  ? what does this project do?".dark_grey());
    println!("{}", "  migrate core/src from rs to ts into ./out".dark_grey());
    println!();

    // ── Session Tracker ─────────────────────────────────────────────
    let tracker = SessionTracker::new(root);

    // ── Auto-Index on Startup ───────────────────────────────────────
    let astra_index = root.join(".astra").join("index.json");
    let legacy_index = root.join(".codex").join("index.json");
    if !astra_index.exists() && !legacy_index.exists() {
        println!("{}", "⚙\u{fe0f} No index found. Auto-indexing workspace in the background...".yellow());
        match engine.handle_input(":index") {
            Ok(res) => println!("{}", res.dark_grey()),
            Err(e) => println!("{} {}", "⚠\u{fe0f} Auto-index failed:".red(), e),
        }
        println!();
    }

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
            tracker.finish_session(engine.memory_mut());
            break;
        }

        match engine.handle_input(trimmed) {
            Ok(reply) => {
                println!("\n{} {}\n", "◈ Astra ❯".magenta().bold(), reply);
                let user = current_user_name();
                let _ = team_mgr.log_prompt(&user, trimmed);
            }
            Err(e) => {
                eprintln!("\n{} {}\n", " ❌ Error ❯".red().bold(), e);
            }
        }
    }

    Ok(())
}
