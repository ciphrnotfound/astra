from typing import Union

def greet(name: str) -> str:
    return f"Hello, {name}"

def add(a: int, b: int) -> int:
    return a + b

def format_user(id: int, username: str) -> str:
    return f"{id}:{username.lower()}"