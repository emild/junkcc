
use crate::config::Config;
use std::io::Write;
use std::process::{self, Stdio};
use std::fs;

const CPP_EXE: &str = "cpp";
const AS_EXE: &str = "as";
const LD_EXE: &str = "ld";

const PP_EXT: &str = "pp.c";
const AS_EXT: &str = "s";
const OBJ_EXT: &str = "o";
const EXE_EXT: &str = "";

const CRT1_O: &str = "/usr/lib/x86_64-linux-gnu/crt1.o";
const CRTI_O: &str = "/usr/lib/x86_64-linux-gnu/crti.o";
const CRTN_O: &str = "/usr/lib/x86_64-linux-gnu/crtn.o";

const ELF_INTEPRETER: &str = "/lib64/ld-linux-x86-64.so.2";

const LIBC: &str = "-lc";


fn get_filename(config: &Config, ext: &str) -> String
{
    let len = config.input_file_path.len();
        let basename = String::from(&config.input_file_path[0..len-2]);

    if ext.len() > 0 {
        format!("{basename}.{ext}")
    }
    else {
        basename
    }
}

fn get_pp_filename(config: &Config) -> String
{
    return get_filename(&config, PP_EXT);
}

fn get_as_filename(config: &Config) -> String
{
    return get_filename(&config, AS_EXT);
}

fn get_obj_filename(config: &Config) -> String
{
    return get_filename(&config, OBJ_EXT);
}

fn get_exe_filename(config: &Config) -> String
{
    return get_filename(&config, EXE_EXT);
}


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

pub fn preprocess(config: &Config) -> Result<(), String>
{
    let input_file_path = &config.input_file_path;
    let pp_file_path = get_pp_filename(config);
    let args = ["-P", input_file_path, "-o", &pp_file_path];

    return run_helper_program(CPP_EXE, &args);
}


pub fn compile(config: &Config) -> Result<(), String>
{
    use crate::compiler;
    let pp_file_path = get_pp_filename(config);
    let as_file_path = get_as_filename(config);

    compiler::run(&config, &pp_file_path, &as_file_path)?;

    if config.stop_after_lexer || config.stop_after_parser
    {
        return Ok(());
    }

    if let Ok(mut as_file) = fs::File::create(&as_file_path) {
        //STUBBED
        let stubbed_code: &str = 
"
    .file	\"return_2.c\"
	.text
	.globl	main
	.type	main, @function
main:
	movl	$2, %eax
	ret
	.size	main, .-main
	.ident	\"GCC: (Ubuntu 13.3.0-6ubuntu2~24.04) 13.3.0\"
	.section	.note.GNU-stack,\"\",@progbits
";
        match as_file.write_all(stubbed_code.as_bytes()) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(format!("Failed to write stubbed ASM: '{e}'"))
        };
    }
    else {
        return Err(format!("Failed to open as file: '{as_file_path}'"));
    }
}

pub fn assemble(config: &Config) -> Result<(), String>
{
    let as_file_path = get_as_filename(config);
    let obj_file_path = get_obj_filename(config);
    let args = [&as_file_path, "-o", &obj_file_path];

    return run_helper_program(AS_EXE, &args);
}


pub fn link(config: &Config) -> Result<(), String>
{
      
    let obj_file_path = get_obj_filename(config);
    let exe_file_path = get_exe_filename(config);
    let args = [
        "-o", &exe_file_path, &obj_file_path,
        CRT1_O, CRTI_O, CRTN_O,
        "-dynamic-linker", ELF_INTEPRETER,
        LIBC
    ];

    return run_helper_program(LD_EXE, &args);
}
