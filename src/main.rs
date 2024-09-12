use std::io;
use std::io::Write;
use std::env;

fn main() {
    let mut input = String::new();
    while input.trim() != "exit"
    {
        print!("{}@{}:{}> ", get_env_variable("USER".to_string()), get_env_variable("HOSTNAME".to_string()), get_env_variable("PWD".to_string()));
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
        if token.starts_with('$')
        {
            result.push(get_env_variable(token[1..].to_string()));
        }
        else if token.starts_with("~/") || (token == "~" && token.len() == 1)
        {
            result.push(get_env_variable("HOME".to_string()))
        }
        else
        {
            result.push(token.to_string());
        }
    }
    result
}

fn echo(input: &Vec<String>)
{
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

fn get_env_variable(input: String) -> String
{
    if let Ok(output) = env::var(input)
    {
        return output;
    }
    "unknown".to_string()
}