# Codex

> The CLI that knows your codebase better than you do

A conversational CLI tool that understands your entire codebase semantically across multiple programming languages, maintains local semantic memory, and can plan + execute cross-language refactors autonomously.

## 🚀 What Makes Codex Different

**Talk to your codebase like a colleague:**
```bash
$ codex "migrate our Express auth middleware to the Go service"
$ codex "why does the Python worker handle retries differently than the TS frontend?"
$ codex "refactor all database calls to use our new connection pool pattern"
```

**Key Features:**
- 🧠 **Local Semantic Memory** - Builds and maintains a persistent graph of your codebase
- 🔄 **Cross-Language Refactoring** - Migrate code between TypeScript ↔ Go ↔ Python ↔ Rust ↔ Java
- 💬 **Conversational Interface** - Natural language commands that actually execute
- 🔒 **Fully Local** - Your code never leaves your machine
- ⚡ **Zero Config** - `codex init` and you're ready to go

## 🏗️ Architecture

```
codex/
├── core/           # Rust engine - semantic analysis & refactor planning
├── cli/            # Command-line interface
├── lsp/            # LSP sidecar for editor integration
├── hooks/          # Git & CI hook runners
└── tui/            # Terminal UI for visualization
```

## 🛠️ Current Status

**Phase 1 (In Development):**
- [x] Basic CLI structure
- [x] Core engine foundation
- [ ] Multi-language AST parsing (TypeScript, Go, Python, Rust, Java)
- [ ] Semantic graph construction
- [ ] Cross-language construct mapping
- [ ] Refactor plan generation

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/yourusername/codex.git
cd codex
cargo build --release

# Initialize in your project
./target/release/codex-cli init

# Start exploring
./target/release/codex-cli "analyze this codebase"
```

## 🎯 Roadmap

**Phase 1 - Core Engine** (Current)
- Multi-language parsing and semantic analysis
- Cross-language construct mapping
- Basic refactor planning

**Phase 2 - Team Features**
- Team task assignment and tracking
- Collaborative refactoring
- Productivity metrics

**Phase 3 - Advanced Intelligence**
- Persistent memory across sessions
- Codebase health monitoring
- Predictive refactoring suggestions

## 🤝 Contributing

This project is currently in early development. Contributions welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## 📄 License

[License TBD - will be open source]

## 🔮 Vision

Codex aims to be the first tool that combines local semantic memory, cross-language AST intelligence, conversational UX, and autonomous multi-step execution. Think of it as having a senior engineer who has read every line of code you've ever written, available 24/7 in your terminal.

---

*Built with ❤️ in Rust*