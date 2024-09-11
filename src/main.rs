use std::io;
use std::io::Write;
use std::env;

fn main() {
    let mut input = String::new();
    while input.trim() != "exit"
    {
        print!("USER@MACHINE:{}> ", env::current_dir().unwrap().display());
        io::stdout().flush().expect("failed to flush output");

        input.clear();
        io::stdin().read_line(&mut input).expect("failed to read input");
        let tokens: Vec<String> = tokenize(&input);


        if tokens[0] == "echo"
        {
            echo(&tokens);
        }
    }
}

fn tokenize(input: &String) -> Vec<String>
{
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();
    let mut result: Vec<String> = Vec::new();
    for token in tokens
    {
        if token == "$USER"
        {
            result.push("LEOZINHO".to_string());
        }
        else {
            result.push(token.to_string());
        }
    }
    result
}

fn echo(input: &Vec<String>) {
    if input.len() == 1
    {
        println!();
        return;
    }
    let mut index = 1;
    let length = input.len();
    while index < length-1
    {
        print!("{} ", input[index] );
        index += 1;
    }
    println!("{}", input[index]);
}