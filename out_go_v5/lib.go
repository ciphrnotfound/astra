package main

import (
	"github.com/google/go-github-v4/v5"
	"github.com/jmoiron/sqlx"
	"github.com/rubenv/sql-migrate"
	_ "github.com/lib/pq"

	"github.com/doug-martin/go-pg/migrations"
)

type engine struct{}

func NewEngine() *engine {
	return &engine{}
}

func (e *engine) Ping() error {
	// TODO: Add connection details
	return nil
}

type parser interface{}

type config struct {
	Git github.Client
	Pg   *sqlx.DB
}

func NewConfig() *config {
	return &config{
		Git: *github.NewClient(&github.Config{
			AppID:        "",
			AppSecret:    "",
			Token:        "",
			Verbosity:    0,
			Server:       "https://api.github.com",
			ClientServer: "https://api.github.com",
			Redirect:     "",
		}, nil),
		Pg: pg,
	}
}

func (c *config) GetParser() parser {
	return // TODO: Implementation
}

type idx struct{}

type Query struct {
	// Fields
}

func (q *Query) Query() error {
	return nil
}

// NewQuery returns a new query
func NewQuery() *Query {
	return &Query{}
}

func main() {
	p := NewConfig()
	m, err := migrations.NewDatabaseMigrator(pg, &migrations.Config{})
	if err != nil {
		log.Fatal(err)
	}

	// Register all SQL files in the "db/migrations" directory
	m.SetTable("history")

	migrate.SetTableMigration(m.GetTable(), "schema")
	err = m.Up()
	if err != nil {
		log.Fatal(err)
	}
}