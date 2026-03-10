pub struct Migration {
    pub id: &'static str,
    pub from_stack: &'static str,
    pub to_stack: &'static str,
    pub description: &'static str,
    pub steps: &'static [&'static str],
}

static MIGRATIONS: &[Migration] = &[
    Migration {
        id: "express-to-rust-axum",
        from_stack: "node-express",
        to_stack: "rust-axum",
        description: "Migrate an Express.js HTTP API to a Rust Axum service.",
        steps: &[
            "Identify core routes and middleware in the Express app.",
            "Design equivalent Axum route tree and extract shared state.",
            "Port request handlers route by route, preserving semantics.",
            "Replace Express middleware with Axum layers and tower middleware.",
            "Align error handling and logging across the new service.",
        ],
    },
    Migration {
        id: "django-to-fastapi",
        from_stack: "django",
        to_stack: "fastapi",
        description: "Migrate a Django REST API to FastAPI.",
        steps: &[
            "Catalog existing Django views and DRF serializers.",
            "Define FastAPI routers and Pydantic models for existing endpoints.",
            "Port business logic into dependency-injected functions.",
            "Recreate authentication and permission checks in FastAPI.",
            "Update clients and tests to target the new API URLs.",
        ],
    },
];

pub fn list_migrations() -> &'static [Migration] {
    MIGRATIONS
}

pub fn find_migration(id: &str) -> Option<&'static Migration> {
    MIGRATIONS.iter().find(|m| m.id == id)
}

