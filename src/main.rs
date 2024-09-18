use libc::{fork, waitpid};
use nix::unistd::execv;
use std::env;
use std::ffi::CString;
use std::io;
use std::io::Write;
use std::path::Path;

fn main() {
    let mut input = String::new();
    while input.trim() != "exit" {
        print!(
            "{}@{}:{}> ",
            get_env_variable("USER".to_string()),
            get_env_variable("HOSTNAME".to_string()),
            get_env_variable("PWD".to_string())
        );
        io::stdout().flush().expect("failed to flush output");

        input.clear();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");
        let tokens: Vec<String> = tokenize(&input);

            external_command(tokens);
    }
}

fn tokenize(input: &String) -> Vec<String> {
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();
    let mut result: Vec<String> = Vec::new();
    for token in tokens {
        if token.starts_with('$') {
            result.push(get_env_variable(token[1..].to_string()));
        } else if token.starts_with("~/") || (token == "~" && token.len() == 1) {
            result.push(get_env_variable("HOME".to_string()) + &token[1..])
        } else {
            result.push(token.to_string());
        }
    }
    result
}

fn get_env_variable(input: String) -> String {
    if let Ok(output) = env::var(input) {
        return output;
    }
    "unknown".to_string()
}

fn external_command(input: Vec<String>) {
    if let Some(path) = find_path(&input[0]) {
        let mut args: Vec<CString> = Vec::new();
        for argument in input.clone() {
            args.push(CString::new(argument).expect("Invalid CString"));
        }

        let child = unsafe { fork() };
        if child == 0 {
            execv(&*path, &*args).expect("Command Execution Failed");
        } else if child > 0 {
            let mut status = 0;
            unsafe {
                waitpid(child, &mut status, 0);
            }
        } else {
            eprintln!("External Command Failed")
        }
    } else {
        println!("Command Path Not Found")
    }
}

fn find_path(input: &String) -> Option<CString> {
    if let Ok(paths) = env::var("PATH") {
        for path in paths.split(':') {
            let full_path = Path::new(path).join(input);
            if full_path.exists() {
                if let Ok(c_string) = CString::new(full_path.to_string_lossy().into_owned()) {
                    return Some(c_string);
                }
            }
        }
    }
    None
}