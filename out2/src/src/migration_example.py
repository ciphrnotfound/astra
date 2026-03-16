from typing import Optional

class User:
    def __init__(self, id: int, username: str):
        self.id = id
        self.username = username

def greet(name: str) -> str:
    return f"Hello, {name}"

def add(a: int, b: int) -> int:
    return a + b

def format_user(id: int, username: str) -> str:
    return f"{id}:{username.lower()}"