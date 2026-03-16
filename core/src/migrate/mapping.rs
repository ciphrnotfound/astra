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
        
        // Add more common mappings here
    }

    fn add(&mut self, mapping: LibraryMapping) {
        self.mappings.insert((mapping.source_lib.clone(), mapping.target_lang), mapping);
    }

    pub fn get(&self, source_lib: &str, target_lang: Language) -> Option<&LibraryMapping> {
        self.mappings.get(&(source_lib.to_string(), target_lang))
    }
}
