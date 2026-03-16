fn main() {
    let mut numbers = vec![1, 2, 3, 4, 5];
    numbers.push(6);
    
    for n in numbers {
        if n % 2 == 0 {
            println!("Even: {}", n);
        } else {
            println!("Odd: {}", n);
        }
    }
}

pub fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
