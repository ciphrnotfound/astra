package main

import (
    "fmt"
)

// Note: In Go, it's conventional to put types and functions in the same package.
// This code snippet doesn't need a separate type for IndexStats as it's assumed
// that IndexStats is already a type that's being used in your program.

type QueryResult struct {
    Message string
    Stats    IndexStats
}

type IndexStats map[string]interface{}