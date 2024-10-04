mod external_command_exec;
mod internal_commands;
mod io_redirection;
mod piping;
mod tokenization;

use crate::external_command_exec::external_command;
use crate::internal_commands::{cd, exit_shell, jobs};
use crate::io_redirection::io_redirection;
use crate::piping::execute_piping;
use crate::tokenization::{get_env_variable, tokenize};
use libc::{waitpid, WNOHANG};
use std::io;
use std::io::Write;

fn main() {
    let mut input = String::new();
    let mut cmd_history: Vec<String> = Vec::new();

    let mut job_number = 1;
    // Vector stores PID, Command, Job Number
    let mut background_processes: Vec<(i32, String, i32)> = Vec::new();
    let mut is_success: bool = false;

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

        if tokens.is_empty() {
            print!("");
            continue;
        }
        let mut is_background = false;
        if tokens.len() > 0 && tokens[tokens.len() - 1] == "&" {
            tokens.pop();
            is_background = true;
        }

        //Internal Commands
        if tokens[0] == "jobs" {
            jobs(&background_processes);
            is_success = true;
        } else if tokens[0] == "cd" {
            is_success = cd(&tokens);
        } else if tokens[0] == "exit" {
            exit_shell(cmd_history, background_processes);
            return;
        }
        //External Commands
        else if tokens.iter().any(|s| s == ">" || s == "<") {
            is_success =
                io_redirection(tokens, is_background, &mut background_processes, job_number);
        } else if tokens.iter().any(|s| s == "|") {
            execute_piping(tokens, is_background, &mut background_processes, job_number);
        } else {
            is_success = external_command(
                tokens,
                None,
                None,
                is_background,
                &mut background_processes,
                job_number,
                false,
            );
        }

        if is_success {
            cmd_history.push(input.trim().to_string());
        }

        if is_background {
            job_number += 1;
        }

        check_background_processes(&mut background_processes);
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
