package main

import (
	"fmt"
)

type Migration struct {
	ID          string
	FromStack   string
	ToStack     string
	Description string
	Steps       []string
}

var MIGRATIONS = []Migration{
	{
		ID:          "express-to-rust-axum",
		FromStack:   "node-express",
		ToStack:     "rust-axum",
		Description: "Migrate an Express.js HTTP API to a Rust Axum service.",
		Steps: []string{
			"Identify core routes and middleware in the Express app.",
			"Design equivalent Axum route tree and extract shared state.",
			"Port request handlers route by route, preserving semantics.",
			"Replace Express middleware with Axum layers and tower middleware.",
			"Align error handling and logging across the new service.",
		},
	},
	{
		ID:          "django-to-fastapi",
		FromStack:   "django",
		ToStack:     "fastapi",
		Description: "Migrate a Django REST API to FastAPI.",
		Steps: []string{
			"Catalog existing Django views and DRF serializers.",
			"Define FastAPI routers and Pydantic models for existing endpoints.",
			"Port business logic into dependency-injected functions.",
			"Recreate authentication and permission checks in FastAPI.",
			"Update clients and tests to target the new API URLs.",
		},
	},
}

func listMigrations() []Migration {
	return MIGRATIONS
}

func findMigration(id string) *Migration {
	for _, migration := range MIGRATIONS {
		if migration.ID == id {
			return &migration
		}
	}
	return nil
}