package main

import (
	_ "example/engine"
	_ "example/migration"
	_ "example/migration_example"
	_ "example/parser"
	_ "example/query"
	_ "example/model"
	_ "example/memory"
	_ "example/git"
	_ "example/scaffold"
	_ "example/ts_migrate"
	_ "example/migrate"
	_ "example/teams"
	_ "example/health"
	_ "example/time_travel"
	_ "example/security"
	_ "example/persona"
)

func main() {}

type Engine = engine.Engine
type Parser = parser.Parser
type Index = index.Index
type Query = query.Query
type Model = model.Model
type Memory = memory.Memory
type Git = git.Git
type Migration = migration.Migration
type Scaffold = scaffold.Scaffold
type TS_Migrate = ts_migrate.TS_Migrate
type Migrate = migrate.Migration
type Teams = teams.Teams
type Health = health.Health
type Time_travel = time_travel.Time_travel
type Security = security.Security
type Persona = persona.Persona

func NewEngine() Engine { return engine.NewEngine() }
func NewParser() Parser { return parser.NewParser() }
func NewIndex() Index   { return index.NewIndex() }
func NewQuery() Query   { return query.NewQuery() }
func NewModel() Model   { return model.NewModel() }
func NewMemory() Memory { return memory.NewMemory() }
func NewGit() Git      { return git.NewGit() }
func NewMigration() Migration { return migration.NewMigration() }
func NewScaffold() Scaffold  { return scaffold.NewScaffold() }
func NewTS_Migrate() TS_Migrate { return ts_migrate.NewTS_Migrate() }
func NewMigrate() Migrate      { return migrate.NewMigration() }
func NewTeams() Teams         { return teams.NewTeams() }
func NewHealth() Health       { return health.NewHealth() }
func NewTime_travel() Time_travel { return time_travel.NewTime_travel() }
func NewSecurity() Security   { return security.NewSecurity() }
func NewPersona() Persona     { return persona.NewPersona() }