package main

import "fmt"

func greet(name string) string {
    return fmt.Sprintf("Hello, %s", name)
}

func add(a int32, b int32) (int32, error) {
    return a + b, nil
}

func formatUser(id int32, username string) (string, error) {
    return fmt.Sprintf("%d:%s", id, username), nil
}

func main() {
    // Example usage:
    fmt.Println(greet("Alice"))  
    res, _ := add(1, 2)
    fmt.Println(res)
    res, _ = formatUser(123, "Bob")
    fmt.Println(res)
}