use codex_core::engine::CodexEngine;

pub fn start_tui() {
    let mut engine = CodexEngine::new();
    let response = engine
        .handle_input("tui startup")
        .unwrap_or_else(|_| "failed to start engine".to_string());
    println!("codex TUI placeholder: {}", response);
}
