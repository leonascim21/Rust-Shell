use libc::{fork, waitpid};
use std::env;
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
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

        if tokens.iter().any(|s| s == ">" || s == "<") {
            io_redirection(tokens);
        } else {
            external_command(tokens, None, None);
        }
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

fn external_command(input: Vec<String>, input_fd: Option<RawFd>, output_fd: Option<RawFd>) {
    if let Some(path) = find_path(&input[0]) {
        let args_cstr: Vec<CString> = input
            .iter()
            .map(|arg| CString::new(arg.as_str()).expect("Failed to convert to CString"))
            .collect();

        let mut arg_ptrs: Vec<*const libc::c_char> = args_cstr
            .iter()
            .map(|arg| arg.as_ptr())
            .collect();
        arg_ptrs.push(std::ptr::null());


        let child = unsafe { fork() };
        if child == 0 {
            if let Some(fd) = input_fd {
                unsafe {
                    libc::dup2(fd, libc::STDIN_FILENO);
                    libc::close(fd);
                }
            }

            if let Some(fd) = output_fd {
                unsafe {
                    libc::dup2(fd, libc::STDOUT_FILENO);
                    libc::close(fd);
                }
            }

            unsafe { libc::execv(path.as_ptr(), arg_ptrs.as_ptr()) };
            println!("Command execution failed");
            std::process::exit(1);
        } else if child > 0 {
            // Parent process
            let mut status = 0;
            unsafe {
                waitpid(child, &mut status, 0);
            }
        } else {
            println!("External Command Failed");
        }
    } else {
        println!("Command Path Not Found");
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

fn io_redirection(input: Vec<String>) {
    let mut command = Vec::new();
    let mut input_file = None;
    let mut output_file = None;
    let mut i = 0;

    while i < input.len() {
        match input[i].as_str() {
            "<" => {
                i += 1;
                if i < input.len() {
                    input_file = Some(input[i].clone());
                } else {
                    eprintln!("No input file specified");
                    return;
                }
            }
            ">" => {
                i += 1;
                if i < input.len() {
                    output_file = Some(input[i].clone());
                } else {
                    eprintln!("No output file specified");
                    return;
                }
            }
            _ => {
                command.push(input[i].to_string());
            }
        }
        i += 1;
    }

    if command.is_empty() {
        println!("No Command Provided");
        return;
    }

    let (input_fd, input_file_handle) = if let Some(ref input_filename) = input_file {
        match File::open(input_filename) {
            Ok(file) => (Some(file.as_raw_fd()), Some(file)),
            Err(e) => {
                println!("Error opening input file: {}", e);
                return;
            }
        }
    } else {
        (None, None)
    };

    let (output_fd, output_file_handle) = if let Some(ref output_filename) = output_file {
        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(output_filename)
        {
            Ok(file) => (Some(file.as_raw_fd()), Some(file)),
            Err(e) => {
                println!("Error creating output file: {}", e);
                return;
            }
        }
    } else {
        (None, None)
    };

    external_command(command, input_fd, output_fd);

    drop(input_file_handle);
    drop(output_file_handle);
}
