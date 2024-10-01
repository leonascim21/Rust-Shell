use std::env::{current_dir, set_current_dir, set_var};
use std::path::Path;
use libc::waitpid;
use crate::tokenization::get_env_variable;

pub(crate) fn jobs(background_processes: &Vec<(i32, String, i32)>) {
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

pub(crate) fn cd(input: &Vec<String>) {
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

pub(crate) fn exit_shell(cmd_history: Vec<String>, background_processes: Vec<(i32, String, i32)>) {
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