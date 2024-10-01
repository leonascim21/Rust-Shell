use std::os::raw::c_int;
use libc::{close, dup2, exit, fork, pipe, waitpid, STDIN_FILENO, STDOUT_FILENO};
use crate::external_command_exec::external_command;

pub(crate) fn execute_piping(
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
                true
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