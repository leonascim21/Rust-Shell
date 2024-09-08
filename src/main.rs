use std::io;
use std::io::Write;
use std::env;

fn main() {
    let mut input = String::new();
    while input.trim() != "exit"
    {
        print_working_dir();
        io::stdout().flush().expect("failed to flush output");

        input.clear();
        io::stdin().read_line(&mut input).expect("failed to read input");

        let tokens: Vec<&str> = input.trim().split_whitespace().collect();

    }

}

fn print_working_dir() {
    print!("{}> ", env::current_dir().unwrap().display());
}