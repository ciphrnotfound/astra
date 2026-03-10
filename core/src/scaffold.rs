pub struct ScaffoldPlan {
    pub stack: String,
    pub commands: Vec<String>,
    pub notes: Vec<String>,
}

pub fn plan_scaffold(stack: &str) -> ScaffoldPlan {
    match stack {
        "node-express" => ScaffoldPlan {
            stack: stack.to_string(),
            commands: vec![
                "mkdir backend && cd backend".to_string(),
                "npm init -y".to_string(),
                "npm install express".to_string(),
                "npm install --save-dev typescript @types/node @types/express".to_string(),
                "npx tsc --init".to_string(),
            ],
            notes: vec![
                "Create src/index.ts with a basic Express server.".to_string(),
                "Add npm scripts for dev (nodemon/ts-node) and build.".to_string(),
            ],
        },
        "rust-axum" => ScaffoldPlan {
            stack: stack.to_string(),
            commands: vec![
                "cargo new backend".to_string(),
                "cd backend".to_string(),
                r#"cargo add axum tokio --features tokio/full"#.to_string(),
            ],
            notes: vec![
                "Update src/main.rs to start an Axum HTTP server.".to_string(),
                "Add routes and extractors for your core API endpoints.".to_string(),
            ],
        },
        other => ScaffoldPlan {
            stack: other.to_string(),
            commands: vec![],
            notes: vec![
                "No predefined scaffold for this stack.".to_string(),
                "Describe the stack in detail and let the LLM propose a plan.".to_string(),
            ],
        },
    }
}
