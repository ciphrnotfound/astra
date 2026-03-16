package main

import (
	"detect"
	"orchestrate"
)

type MigrationConfig orchestrate.MigrationConfig
type MigrationResult orchestrate.MigrationResult

// Language is an alias for detect.Language
type Language detect.Language

func run_migration(config MigrationConfig) MigrationResult {
	return orchestrate.RunMigration(config)
}

// clean
package clean

type Cleaner interface {
	Clean() error
}

type CleanupTask struct {
	// fields
}

func NewCleanupTask() CleanupTask {
	return CleanupTask{}
}

func (t *CleanupTask) Clean() error {
	// implementation
	return nil
}

// mapping
package mapping

type Mapping interface {
	Map() error
}

type MappingTask struct {
	// fields
}

func NewMappingTask() MappingTask {
	return MappingTask{}
}

func (t *MappingTask) Map() error {
	// implementation
	return nil
}

// Scaffold and translate are not used in this snippet, but their modules can be declared similarly
```

```go
// orchestrate
package orchestrate

type MigrationConfig struct {
	// fields
}

type MigrationResult struct {
	// fields
}

func RunMigration(config MigrationConfig) MigrationResult {
	// implementation
	return MigrationResult{}
}

// detect
package detect

type Language string

// scaffold
package scaffold

// translate
package translate

// mapping
package common

// This package contains the shared types and interfaces
package shared

type Cleaner interface {
	Clean() error
}

type Mapping interface {
	Map() error
}