export function greet(name: string): string {
    return `Hello, ${name}`;
}

export function add(a: number, b: number): number {
    return a + b;
}

export function formatUser(id: number, username: string): string {
    return `${id}:${username.toLowerCase()}`;
}

