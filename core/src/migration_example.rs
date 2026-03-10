pub fn greet(name: &str) -> String {
    format!("Hello, {}", name)
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn format_user(id: i32, username: &str) -> String {
    format!("{}:{}", id, username.to_lowercase())
}

