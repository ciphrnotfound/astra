package main

import (
	"fmt"
	"strings"
)

func greet(name string) string {
    return fmt.Sprintf("Hello, %s", name)
}

func add(a int, b int) int {
    return a + b
}

func formatUser(id int, username string) string {
    return fmt.Sprintf("%d:%s", id, strings.ToLower(username))
}

func main() {}