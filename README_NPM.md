# Astra CLI 🚀

Astra is an AI-powered codebase architect that helps you analyze, migrate, and orchestrate complex refactors across multiple languages.

## Installation

```bash
npm install -g astra-cli
```

On install, Astra downloads a small prebuilt binary for your platform into `~/.astra/bin/<version>/`.

## Quick Start

```bash
# In your project root
astra --env .env
```

## Troubleshooting

- If install is blocked by a corporate proxy/firewall, set `ASTRA_DOWNLOAD_BASE_URL` to your internal release host.
- If you prefer to compile locally (or no prebuilt binary exists yet), set `ASTRA_BUILD_FROM_SOURCE=1` and ensure Rust is installed.

## Releasing

1. Bump `package.json` version and tag the same version:
   - Tag format must be `v<package.json version>` (example: `v0.1.0`)
2. Push the tag to GitHub:
   - The workflow builds and uploads binaries named:
     - `astra-cli-<version>-win32-x64.exe`
     - `astra-cli-<version>-linux-x64.bin`
     - `astra-cli-<version>-darwin-x64.bin`
     - `astra-cli-<version>-darwin-arm64.bin`
3. Publish to npm:
   - `npm publish`

## Features

- **Semantic Indexing**: Astra builds a deep graph of your codebase.
- **Cross-Language Migration**: Autonomous agents can refactor Rust to TypeScript, Go to Python, etc.
- **Persistent Memory**: Astra remembers your project goals and architecture.
- **Global OS Layer**: Astra is no longer localized; its brain lives at `~/.astra/brain/` for cross-project intelligence.
- **Agentic Semantic Tools**: Astra can now autonomously query its own Temporal Graph (`:owners`, `:coupling`, `:why`) to provide zero-guesswork answers.
- **Team Delegation**: Assign tasks to virtual engineers like `!Sam`.

---

Built with Rust & ❤️ by Ciphr. Later Nerds.
