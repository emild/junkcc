
use crate::config::Config;
use std::process::{self, Stdio};

const CPP_EXE: &str = "cpp";
const AS_EXE: &str = "as";
const LD_EXE: &str = "ld";



const CRT1_O: &str = "/usr/lib/x86_64-linux-gnu/crt1.o";
const CRTI_O: &str = "/usr/lib/x86_64-linux-gnu/crti.o";
const CRTN_O: &str = "/usr/lib/x86_64-linux-gnu/crtn.o";

const ELF_INTEPRETER: &str = "/lib64/ld-linux-x86-64.so.2";

const LIBC: &str = "-lc";





fn run_helper_program(exe_name: &str,
                args: &[&str]) -> Result<(), String>
{
    match process::Command::new(exe_name)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        {
            Ok(child) => {
                match child.wait_with_output() {
                    Ok(output) => {
                        if !output.status.success() {
                            return Err(format!("Process '{exe_name}' exited with failed status: {}.\nstdout=\n'{}'\n\nstderr='\n{}",
                                output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)));
                        }

                        return Ok(());
                    },
                    Err(err) => {
                        return Err(format!("Failed to wait for process '{CPP_EXE}' to complete. Error: {err}"));
                    }
                };
            },
            Err(err) => {
                return Err(format!("Failed to spawn pchild rocess '{CPP_EXE}'. Error: {err}"));
            }
        }
}


pub fn preprocess(input_file_path: &String, pp_file_path: &String) -> Result<(), String>
{
    let args = ["-P", &input_file_path, "-o", &pp_file_path];

    return run_helper_program(CPP_EXE, &args);
}


pub fn compile(config: &Config, pp_file_path: &String, as_file_path: &String) -> Result<(), String>
{
    use crate::compiler;

    compiler::run(&config, &pp_file_path, &as_file_path)?;
    Ok(())
}

pub fn assemble(as_file_path: &String, obj_file_path: &String) -> Result<(), String>
{
    let args = [&as_file_path, "-o", &obj_file_path];

    return run_helper_program(AS_EXE, &args);
}


pub fn link(exe_file_path: &String, obj_file_paths: &[String]) -> Result<(), String>
{
    let mut args = vec![
        "-o", &exe_file_path
    ];

    for obj_file_path in obj_file_paths {
        args.push(obj_file_path);
    }

    for arg in [
        CRT1_O, CRTI_O, CRTN_O,
        "-dynamic-linker", ELF_INTEPRETER,
        LIBC
    ] {
        args.push(arg);
    }

    return run_helper_program(LD_EXE, &args);
}
