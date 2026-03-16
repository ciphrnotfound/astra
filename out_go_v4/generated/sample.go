package main

import (
	"fmt"
)

func greet(name string) string {
	return fmt.Sprintf("Hello, %s", name)
}

func add(a int, b int) int {
	return a + b
}

func formatUser(id int, username string) string {
	return fmt.Sprintf("%d:%s", id, username)
}

func main() {}