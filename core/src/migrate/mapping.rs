use std::collections::HashMap;
use super::detect::Language;

pub struct LibraryMapping {
    pub source_lib: String,
    pub target_lang: Language,
    pub target_lib: String,
    pub import_path: String,
    pub notes: String,
}

pub struct LibraryRegistry {
    mappings: HashMap<(String, Language), LibraryMapping>,
}

impl LibraryRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            mappings: HashMap::new(),
        };
        registry.init_defaults();
        registry
    }

    fn init_defaults(&mut self) {
        // Rust Git2 -> Go go-git v5
        self.add(LibraryMapping {
            source_lib: "git2".to_string(),
            target_lang: Language::Go,
            target_lib: "go-git".to_string(),
            import_path: "github.com/go-git/go-git/v5".to_string(),
            notes: "Use go-git v5. Prefer plumbing.Hash over OID. Avoid inventing RevWalk() methods.".to_string(),
        });

        // Rust Git2 -> Python subprocess
        self.add(LibraryMapping {
            source_lib: "git2".to_string(),
            target_lang: Language::Python,
            target_lib: "subprocess".to_string(),
            import_path: "import subprocess".to_string(),
            notes: "Use the Git CLI via subprocess. Prefer porcelain output formatting.".to_string(),
        });

        // Rust Serde -> Python Dataclasses
        self.add(LibraryMapping {
            source_lib: "serde".to_string(),
            target_lang: Language::Python,
            target_lib: "dataclasses".to_string(),
            import_path: "from dataclasses import dataclass".to_string(),
            notes: "Use @dataclass for structs.".to_string(),
        });

        // === Common Rust Crates -> TypeScript ===
        // These prevent the web search from finding garbage npm packages
        self.add(LibraryMapping {
            source_lib: "tokio".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "native async/await".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "TypeScript has native async/await. Do NOT import any library for this. Just use async functions and Promise<T>.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "async_trait".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "native interfaces".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "TypeScript interfaces natively support async method signatures with Promise<T> return types. Do NOT import any library.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "anyhow".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "Error".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "Use the built-in Error class. throw new Error('message'). Do NOT import any library.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "thiserror".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "Error subclasses".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "Create custom error classes: class MyError extends Error { constructor(msg: string) { super(msg); } }. Do NOT import any library.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "serde".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "JSON".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "Use JSON.stringify() and JSON.parse(). TypeScript interfaces handle type shape. Do NOT import any library.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "serde_json".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "JSON".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "Use JSON.stringify() and JSON.parse(). Do NOT import any library.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "reqwest".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "fetch".to_string(),
            import_path: "(no import needed)".to_string(),
            notes: "Use the built-in fetch() API. Do NOT import any library.".to_string(),
        });
        self.add(LibraryMapping {
            source_lib: "clap".to_string(),
            target_lang: Language::TypeScript,
            target_lib: "commander".to_string(),
            import_path: "import { Command } from 'commander'".to_string(),
            notes: "Use the 'commander' npm package for CLI argument parsing.".to_string(),
        });
        
        // Add more common mappings here
    }

    fn add(&mut self, mapping: LibraryMapping) {
        self.mappings.insert((mapping.source_lib.clone(), mapping.target_lang), mapping);
    }

    pub fn get(&self, source_lib: &str, target_lang: Language) -> Option<&LibraryMapping> {
        self.mappings.get(&(source_lib.to_string(), target_lang))
    }
}
pub struct ConceptMapping {
    pub concept: String,
    pub from: Language,
    pub to: Language,
    pub pattern: String,
}

pub struct ConceptRegistry {
    concepts: Vec<ConceptMapping>,
}

impl ConceptRegistry {
    pub fn new() -> Self {
        let mut registry = Self { concepts: Vec::new() };
        registry.init_defaults();
        registry
    }

    fn init_defaults(&mut self) {
        // Rust -> TypeScript Idioms with CONCRETE examples
        self.add("Enum with Data", Language::Rust, Language::TypeScript, 
            "NEVER use TypeScript enum for Rust enums with data. Use Discriminated Unions:\n\
             Rust: enum Error { NotFound(u64), Query { sql: String, reason: String } }\n\
             TypeScript: type Error = { kind: 'NotFound'; value: number } | { kind: 'Query'; sql: string; reason: string }\n\
             CRITICAL: Do NOT declare both a `type Error` and a `class Error`. Just output the `type Error`.");
        self.add("Option<T>", Language::Rust, Language::TypeScript, 
            "Use 'T | null'. Example: fn get(id: u64) -> Option<User> becomes get(id: number): User | null");
        self.add("Result<T, E>", Language::Rust, Language::TypeScript, 
            "Use try/catch. CRITICAL: TypeScript DOES NOT have a `throws` keyword in method signatures! \n\
             Example: fn save() -> Result<(), Error> becomes `save(): Promise<void>`. \
             Do NOT append `throws Error` to the signature under any circumstances.");
        self.add("async_trait", Language::Rust, Language::TypeScript, 
            "Convert #[async_trait] trait to TypeScript interface. Methods return Promise<T>, do NOT use async keyword in interface.\n\
             Rust: #[async_trait] trait Store { async fn get(&self, id: u64) -> Option<User>; }\n\
             TypeScript: interface Store { get(id: number): Promise<User | null>; }");
        self.add("impl Trait for Struct", Language::Rust, Language::TypeScript, 
            "Use 'class X implements Y'. If the trait has associated types, pass them as generic arguments to the interface. DO NOT add generic parameters to the class itself unless the Rust struct is generic.\n\
             Rust: impl StorageBackend for InMemoryStorage { type Entity = User; }\n\
             TypeScript: class InMemoryStorage implements StorageBackend<User> { ... }");
        self.add("Arc<RwLock<T>>", Language::Rust, Language::TypeScript, 
            "TypeScript is single-threaded. Drop Arc/RwLock entirely. Use plain Map or object.\n\
             Rust: data: Arc<RwLock<HashMap<u64, User>>>\n\
             TypeScript: private data: Map<number, User>");
        self.add("Struct Fields", Language::Rust, Language::TypeScript, 
            "Rust struct fields become class fields with explicit declarations. NEVER import `Map`.\n\
             Rust: pub struct UserProfile { pub id: u64, pub username: String, pub metadata: HashMap<String, String> }\n\
             TypeScript: class UserProfile { id: number; username: string; metadata: Map<string, string>; constructor(id: number, username: string, metadata: Map<string, string>) { this.id = id; this.username = username; this.metadata = metadata; } }");
        self.add("Generic Static Methods", Language::Rust, Language::TypeScript, 
            "Static methods in TS must redeclare generic params WITH THEIR INNER BOUNDS.\n\
             Rust: impl<S: StorageBackend<Entity = User>> Processor<S> { pub fn new(s: S) -> Self { ... } }\n\
             TypeScript: class Processor<S extends StorageBackend<User>> { static create<S extends StorageBackend<User>>(s: S): Processor<S> { return new Processor(s); } }");
        self.add("Trait Associated Types", Language::Rust, Language::TypeScript, 
            "Convert Rust associated types to TypeScript generics on the interface.\n\
             Rust: trait Store { type Entity; fn get(&self) -> Self::Entity; }\n\
             TypeScript: interface Store<Entity> { get(): Entity; }");
    }

    fn add(&mut self, concept: &str, from: Language, to: Language, pattern: &str) {
        self.concepts.push(ConceptMapping {
            concept: concept.to_string(),
            from,
            to,
            pattern: pattern.to_string(),
        });
    }

    pub fn get_mappings(&self, from: Language, to: Language) -> Vec<&ConceptMapping> {
        self.concepts.iter()
            .filter(|c| c.from == from && c.to == to)
            .collect()
    }
}
