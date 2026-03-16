# Migration Report

- **From**: Rust (`"core/src"`)
- **To**: Go (`"out_go"`)
- **AI-assisted**: true

## Migrated Files (5):
- `"core/src\\generated\\sample.rs"` → `"out_go\\generated\\sample.go"` (21 lines)
- `"core/src\\git.rs"` → `"out_go\\git.go"` (263 lines)
- `"core/src\\lib.rs"` → `"out_go\\lib.go"` (56 lines)
- `"core/src\\memory.rs"` → `"out_go\\memory.go"` (290 lines)
- `"core/src\\query.rs"` → `"out_go\\query.go"` (16 lines)

## Errors (19):
- `"core/src\\engine.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 413 Payload Too Large and no rules found)
- `"core/src\\health.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\index.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migrate\\clean.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migrate\\detect.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migrate\\mod.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migrate\\orchestrate.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migrate\\scaffold.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migrate\\translate.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 413 Payload Too Large and no rules found)
- `"core/src\\migration.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\migration_example.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\model.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\parser.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\persona.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\scaffold.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\security.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\teams.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\time_travel.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
- `"core/src\\ts_migrate.rs"`: Translation failed: Translation failed for Rust → Go (AI error: Groq API error: 429 Too Many Requests and no rules found)
