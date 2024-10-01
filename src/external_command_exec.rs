use std::ffi::CString;
use std::os::fd::RawFd;
use std::{env, ptr};
use std::path::Path;
use libc::{close, dup2, execv, exit, fork, waitpid, STDIN_FILENO, STDOUT_FILENO};

pub(crate) fn external_command(
    input: Vec<String>,
    input_fd: Option<RawFd>,
    output_fd: Option<RawFd>,
    is_background: bool,
    background_processes: &mut Vec<(i32, String, i32)>,
    job_number: i32,
    piping: bool
) {
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
            if is_background && !piping {
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
