use libc::{close, dup2, execv, exit, fork, pipe, waitpid, STDIN_FILENO, STDOUT_FILENO, WNOHANG};
use std::env::{current_dir, set_current_dir, set_var};
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::os::fd::{AsRawFd, RawFd};
use std::os::raw::c_int;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::{env, ptr};

fn main() {
    let mut input = String::new();
    let mut cmd_history: Vec<String> = Vec::new();

    let mut job_number = 1;
    // Vector stores PID, Command, Job Number
    let mut background_processes: Vec<(i32, String, i32)> = Vec::new();

    loop {
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
        let mut tokens: Vec<String> = tokenize(&input);

        let mut is_background = false;
        if tokens.len() > 0 && tokens[tokens.len() - 1] == "&" {
            tokens.pop();
            is_background = true;
        }

        //Internal Commands
        if tokens[0] == "jobs" {
            jobs(&background_processes);
        } else if tokens[0] == "cd" {
            cd(&tokens);
        } else if tokens[0] == "exit" {
            exit_shell(cmd_history, background_processes);
            return;
        }

        //External Commands
        else if tokens.iter().any(|s| s == ">" || s == "<") {
            io_redirection(tokens, is_background, &mut background_processes, job_number);
        } else if tokens.iter().any(|s| s == "|") {
            execute_piping(tokens, is_background, &mut background_processes, job_number);
        } else {
            external_command(
                tokens,
                None,
                None,
                is_background,
                &mut background_processes,
                job_number,
            );
        }

        cmd_history.push(input.trim().to_string());
        if is_background {
            job_number += 1;
        }

        check_background_processes(&mut background_processes);
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

fn external_command(
    input: Vec<String>,
    input_fd: Option<RawFd>,
    output_fd: Option<RawFd>,
    is_background: bool,
    background_processes: &mut Vec<(i32, String, i32)>,
    job_number: i32,
) {
    //TODO: FIX OUT OF BOUNDS (&INPUT[0])
    if let Some(path) = find_path(&input[0]) {
        let args_cstr: Vec<CString> = input
            .iter()
            .map(|arg| CString::new(arg.as_str()).expect("Failed to convert to CString"))
            .collect();

        let mut arg_ptrs: Vec<*const libc::c_char> =
            args_cstr.iter().map(|arg| arg.as_ptr()).collect();
        arg_ptrs.push(ptr::null());

        let child = unsafe { fork() };
        if child == 0 {
            if let Some(fd) = input_fd {
                unsafe {
                    dup2(fd, STDIN_FILENO);
                    close(fd);
                }
            }

            if let Some(fd) = output_fd {
                unsafe {
                    dup2(fd, STDOUT_FILENO);
                    close(fd);
                }
            }

            unsafe { execv(path.as_ptr(), arg_ptrs.as_ptr()) };
            println!("Command execution failed");
            unsafe {
                exit(1);
            }
        } else if child > 0 {
            if is_background {
                println!("[{}] [{}]", job_number, child);
                background_processes.push((child, input.join(" "), job_number));
            } else {
                let mut status = 0;
                unsafe {
                    waitpid(child, &mut status, 0);
                }
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

fn io_redirection(
    input: Vec<String>,
    is_background: bool,
    background_processes: &mut Vec<(i32, String, i32)>,
    job_number: i32,
) {
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
                    println!("No input file specified");
                    return;
                }
            }
            ">" => {
                i += 1;
                if i < input.len() {
                    output_file = Some(input[i].clone());
                } else {
                    println!("No output file specified");
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

    external_command(
        command,
        input_fd,
        output_fd,
        is_background,
        background_processes,
        job_number,
    );

    drop(input_file_handle);
    drop(output_file_handle);
}

//TODO: when background processing job number printed i times
fn execute_piping(
    input: Vec<String>,
    is_background: bool,
    background_processes: &mut Vec<(i32, String, i32)>,
    job_number: i32,
) {
    let commands: Vec<Vec<String>> = input
        .split(|token| token == "|")
        .map(|token| token.to_vec())
        .collect();
    let num_commands = commands.len();
    let mut pipe_fd: Vec<[c_int; 2]> = vec![[0, 0]; num_commands - 1];

    for i in 0..num_commands - 1 {
        if unsafe { pipe(pipe_fd[i].as_mut_ptr()) } == -1 {
            println!("Failed to create pipe");
            return;
        }
    }

    for i in 0..num_commands {
        let child = unsafe { fork() };
        if child == 0 {
            if i > 0 {
                unsafe {
                    dup2(pipe_fd[i - 1][0], STDIN_FILENO);
                    close(pipe_fd[i - 1][0]);
                }
            }

            if i < num_commands - 1 {
                unsafe {
                    dup2(pipe_fd[i][1], STDOUT_FILENO);
                    close(pipe_fd[i][1]);
                }
            }

            for j in 0..num_commands - 1 {
                unsafe {
                    close(pipe_fd[j][0]);
                    close(pipe_fd[j][1]);
                }
            }

            external_command(
                commands[i].clone(),
                None,
                None,
                is_background,
                background_processes,
                job_number,
            );
            unsafe {
                exit(0);
            };
        } else if child > 0 {
            if is_background && i == num_commands - 1 {
                println!("[{}] [{}]", job_number, child);
                background_processes.push((child, input.join(" "), job_number));
            } else if !is_background {
                let mut status = 0;
                unsafe {
                    waitpid(child, &mut status, 0);
                }
            }
        }
    }

    if !is_background {
        for _ in 0..num_commands {
            let mut status = 0;
            unsafe {
                //Use -1 for PID to wait for ANY child process (not specific)
                waitpid(-1, &mut status, 0);
            }
        }
    }

    for i in 0..num_commands - 1 {
        unsafe {
            close(pipe_fd[i][0]);
            close(pipe_fd[i][1]);
        }
    }
}

fn check_background_processes(background_processes: &mut Vec<(i32, String, i32)>) {
    let mut status = 0;
    //Looping in reverse so no out of bounds crash if an item is removed
    for i in (0..background_processes.len()).rev() {
        let (pid, command, job_num) = background_processes[i].clone();
        let result = unsafe { waitpid(pid, &mut status, WNOHANG) };
        if result == pid {
            println!("[{}] + done [{}]", job_num, command);
            background_processes.remove(i);
        }
    }
}

fn jobs(background_processes: &Vec<(i32, String, i32)>) {
    if background_processes.is_empty() {
        println!("No background processes running")
    } else {
        for i in 0..background_processes.len() {
            println!(
                "[{}] + [{}][{}]",
                background_processes[i].2, background_processes[i].0, background_processes[i].1
            );
        }
    }
}

fn cd(input: &Vec<String>) {
    if input.len() > 2 {
        println!("Too many arguments provided for cd");
        return;
    }

    let target_dir = if input.len() == 1 {
        get_env_variable("HOME".to_string())
    } else {
        input[1].clone()
    };

    let path = Path::new(&target_dir);
    if !path.exists() {
        println!("Target does not exist");
    } else if !path.is_dir() {
        println!("Target is not a directory")
    } else {
        set_current_dir(target_dir).expect("Failed to change directory");
        if let Ok(current_dir) = current_dir() {
            if let Some(current_dir_str) = current_dir.to_str() {
                set_var("PWD", current_dir_str);
            }
        }
    }
}

fn exit_shell(cmd_history: Vec<String>, background_processes: Vec<(i32, String, i32)>) {
    if background_processes.len() > 0 {
        for i in 0..background_processes.len() {
            let mut status = 0;
            unsafe {
                waitpid(background_processes[i].0, &mut status, 0);
            }
        }
    }

    if cmd_history.is_empty() {
        println!("No valid command history")
    } else if cmd_history.len() < 3 {
        print!("{}", cmd_history[cmd_history.len() - 1]);
    } else {
        for i in cmd_history.len() - 3..cmd_history.len() {
            println!("{}", cmd_history[i]);
        }
    }
}

//Perguntar se eu preciso pipe e io redirect o jobs
//Aceito jobs se tiver arguments?
//hostname unknown
