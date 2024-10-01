use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use crate::external_command_exec::external_command;

pub(crate) fn io_redirection(
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
        false
    );

    drop(input_file_handle);
    drop(output_file_handle);
}
