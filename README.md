<div align="center">

<img src="https://your-logo-url.com/logo.png" alt="Astra" width="80" />

# Astra

**The Codebase Operating System**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![GitHub Stars](https://img.shields.io/github/stars/ciphrnotfound/astra?style=flat)](https://github.com/ciphrnotfound/astra/stargazers)
[![Discord](https://img.shields.io/badge/Discord-Join-5865F2.svg)](https://discord.gg/your-invite)

[Website](https://astra.sh) · [Documentation](https://docs.astra.sh) · [Changelog](https://github.com/ciphrnotfound/astra/releases) · [Discord](https://discord.gg/your-invite)

</div>

---

## What is Astra?

Astra is a persistent intelligence layer that lives in your terminal and understands your entire codebase — permanently.

Every other tool forgets you the moment you close the terminal. Astra never does.

It builds a living semantic graph of your codebase, remembers every decision ever made, traces bugs through time, protects your security, onboards your team, and gets smarter with every single commit. Free. Forever. Offline.

```bash
$ astra "why did we switch from REST to GraphQL in the user service?"

▸ In March you merged PR #47 — over-fetching issues on mobile.
  Decision made by @sarah. Still 2 places using the old REST path.
  Want me to clean those up?
```

---

## Features

### 🧠 Core Intelligence
- **System 2 Reasoning** — dedicated thought loop that architects solutions before executing
- **Search-Repair Loop** — automatically searches for current syntax when uncertain
- **Vibe Switching** — modular personas (Architect, Brutal, Nigerian Pidgin, Doge)

### 🏗️ Migration & Transformation
- **Mass Migration Engine** — translates entire codebases between languages idiomatically
- **Semantic Cleanup** — post-migration engine that auto-detects and fixes structural drift
- **Project Scaffolding** — generates consistent boilerplates for your entire team

### 🛡️ Security & Time Travel
- **Security Hunter** — scans every file for secrets, SQL injection, unencrypted HTTP
- **Semantic Bisect** (`:bisect`) — finds the exact commit that introduced a bug using natural language

### 👥 Team Operating System
- **Distributed Task Manager** — syncs team tasks across machines via a hidden git branch
- **Productivity Metrics** — tracks developer velocity, time per task, AI prompt history
- **Cloud Sync** — optional real-time sync via Supabase

### 🎓 Onboarding & Learning
- **Onboarding Manager** — acts as a programming tutor for junior developers
- **Learning Phases** — sets up structured learning paths and unlocks new levels via AI code review

### 🔮 Predictive Health
- **Predictive Refactoring** — forecasts hot files, technical debt, and cross-language drift
- **Semantic Cartographer** (`:graph`) — generates a full GraphViz dependency map of your codebase
- **Health Dashboard** — live scores for Code Quality, Test Health, Security Surface, Team Velocity

### 🔌 IDE & Ecosystem
- **MCP Support** — native bridge for Cursor and Claude Desktop
- **Context Injection** — auto-writes `.cursorrules` so AI editors know what you're working on
- **Watch Mode** — real-time monitoring that triggers checks as you type
- **Git Hook Sentry** — blocks commits that lower health scores or introduce security risks

---

## Installation

```bash
# Install via script (recommended)
curl -fsSL https://astra.sh/install | sh

# Or with Cargo
cargo install astra-cli

# Verify installation
astra --version
```

**Supported platforms:** macOS · Linux · Windows (WSL)

---

## Quick Start

```bash
# Navigate to your project
cd your-project

# Initialize Astra
astra init

# Index your codebase
astra :index

# Check codebase health
astra :health

# Ask anything
astra "what is the most dangerous file in this codebase"

# Time travel debug
astra :bisect "when did the payment bug get introduced"

# Capture a bug, tie it to Git evidence, and queue a verified fix
astra :fix-bug "checkout returns 500 when the card is expired"
astra :issues
astra :issue astra-issue-<id>

# Migrate to another language
astra migrate --from typescript --to rust
```

---

## Codex + Claude Code + Cursor coworker mode

Astra can be the persistent project brain while multiple coding agents act as workers. Configure all three project-scoped MCP clients in one step:

```text
:cowork init
```

Then queue work without launching it accidentally:

```text
:delegate codex implement OAuth login with tests
:delegate claude review the checkout architecture
:delegate cursor build the settings UI
:jobs
```

To explicitly launch an installed headless worker CLI, use `:dispatch`:

```text
:dispatch codex fix the flaky authentication tests
```

Workers share compact context, durable project decisions, job state, changed-file reports, and verification evidence through Astra MCP. Source files and full chat transcripts are not copied into memory.

## Issue-to-fix workflow

Use `:fix-bug` when the input is a bug report rather than a known file edit. Astra records the report under `.astra/issues`, captures the current Git HEAD and branch, finds likely files and recent commit evidence, and creates a cowork job with a hard reproduction gate. No production file is changed during intake. A Codex, Claude Code, Cursor, or other MCP worker must first create a failing regression test or replay command, then implement and verify the fix. Use `:issue <id>` or the `astra_issue_status` MCP tool to inspect the evidence and worker state.

---

## Configuration

Astra works out of the box with Ollama (no API key needed). For cloud models, add your key:

```bash
# Use Ollama locally (free, offline, recommended)
astra config set model ollama/deepseek-coder

# Or bring your own key
astra config set api-key YOUR_GROQ_KEY
astra config set api-key YOUR_OPENAI_KEY

# Set your vibe
astra vibe nigerian-pidgin
astra vibe brutal
astra vibe architect
```

---

## Health Dashboard

```
╔══════════════════════════════════════════════════════╗
║            CODEBASE HEALTH REPORT                    ║
╠══════════════════════════════════════════════════════╣
║  Code Quality        99/100  ██████████  ━           ║
║  Security Surface    90/100  ██████████  ▲ (+12)     ║
║  Cross-Lang Drift    75/100  ███████░░░  ▲ (+75)     ║
║  Test Health          4/100  ░░░░░░░░░░  ━           ║
║  Git Health          42/100  ████░░░░░░  ▼ (-8)      ║
║  Team Velocity        0/100  ░░░░░░░░░░  ━           ║
╠══════════════════════════════════════════════════════╣
║  TOP FIXES THIS WEEK                                 ║
╠══════════════════════════════════════════════════════╣
║  1. 6,681 uncommitted changes — commit or stash      ║
║  2. Test coverage critical — only 4/100              ║
║  3. Low commit frequency — commit smaller, more often║
╚══════════════════════════════════════════════════════╝
```

---

## Supported Languages

| Language | Migration | Analysis | Security |
|----------|-----------|----------|----------|
| TypeScript | ✅ | ✅ | ✅ |
| JavaScript | ✅ | ✅ | ✅ |
| Rust | ✅ | ✅ | ✅ |
| Go | ✅ | ✅ | ✅ |
| Python | ✅ | ✅ | ✅ |
| Java | ✅ | ✅ | ✅ |
| Kotlin | 🔜 | ✅ | ✅ |

---

## Philosophy

> *"Every other AI tool is a smart stranger. Astra is the colleague who's been there since day one."*

Astra doesn't write your code. It **understands** it. The distinction matters.

Writing code is easy. Understanding why it exists, what it connects to, what it will break, who built it, and what it will look like in six months — that's hard. That's what Astra does.

---

## Contributing

Astra is open source and contributions are welcome.

```bash
git clone https://github.com/ciphrnotfound/astra
cd astra
cargo build
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Roadmap

- [x] Core CLI + conversational interface
- [x] Codebase indexing + language detection
- [x] Cross-language migration engine
- [x] Semantic memory system
- [x] Health dashboard
- [x] Time travel debugging (`:bisect`)
- [x] Security hunter
- [x] Team task OS
- [x] MCP integration
- [x] Vibe/persona system
- [ ] Cross-repo intelligence
- [ ] Global machine-wide graph
- [ ] Codebase simulation
- [ ] Astra Cloud dashboard
- [ ] VS Code extension

---

## License

MIT © [ciphr](https://github.com/ciphrnotfound)

---

<div align="center">

Built with ❤️ and Rust · [astra.sh](https://astra.sh)

</div>
