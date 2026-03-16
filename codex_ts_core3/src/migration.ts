export interface Migration {
  id: any;
  from_stack: any;
  to_stack: any;
  description: any;
  steps: any;
}


// static MIGRATIONS: &[Migration] = &[
// Migration {
// id: "express-to-rust-axum",
// from_stack: "node-express",
// to_stack: "rust-axum",
// description: "Migrate an Express.js HTTP API to a Rust Axum service.",
// steps: &[
// "Identify core routes and middleware in the Express app.",
// "Design equivalent Axum route tree and extract shared state.",
// "Port request handlers route by route, preserving semantics.",
// "Replace Express middleware with Axum layers and tower middleware.",
// "Align error handling and logging across the new service.",
// ],
// },
// Migration {
// id: "django-to-fastapi",
// from_stack: "django",
// to_stack: "fastapi",
// description: "Migrate a Django REST API to FastAPI.",
// steps: &[
// "Catalog existing Django views and DRF serializers.",
// "Define FastAPI routers and Pydantic models for existing endpoints.",
// "Port business logic into dependency-injected functions.",
// "Recreate authentication and permission checks in FastAPI.",
// "Update clients and tests to target the new API URLs.",
// ],
// },
// ];

export function list_migrations(): any {
  // MIGRATIONS
}


export function find_migration(id: string): any {
  // MIGRATIONS.iter().find(|m| m.id == id)
}


