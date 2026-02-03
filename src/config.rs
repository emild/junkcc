
use std::{collections::HashMap};

#[derive(Debug)]
pub struct Config {
    pub input_file_path: String,
    pub stop_after_lexer: bool,
    pub stop_after_parser: bool,
    pub stop_after_assembly_generation: bool
}

impl Config {
    pub fn build(mut args: impl Iterator<Item=String>) -> Result<Config, String>
    {
        args.next();

        let mut stop_after_lexer = false;
        let mut stop_after_parser = false;
        let mut stop_after_assembly_generation = false;

        let mut opt_map = HashMap::from([
            (String::from("--lex"), &mut stop_after_lexer),
            (String::from("--parse"), &mut stop_after_parser),
            (String::from("--codegen"), &mut stop_after_assembly_generation)
        ]);

        let mut input_file_path_opt: Option<String> = None;
        
        loop {
            let arg = args.next();
            if arg.is_none() {
                break;
            }

            let arg = arg.unwrap();

            match opt_map.get_mut(&arg) {
                Some(opt_val) => {
                    if **opt_val {
                        return Err(format!("Duplicate option: '{arg}'"));
                    }
                    **opt_val = true;
                },
                None => {
                    if arg.starts_with("--") {
                        return Err(format!("Invalid option: '{arg}'"));
                    }

                    if !arg.to_lowercase().ends_with(".c") {
                        return Err(format!("Input file path '{arg}' does not end in .c)"))
                    }

                    if arg.len() < 3 {
                        return Err(format!("Input file path must be at least 3 characters '{arg}' has {}", arg.len()))
                    }

                    if input_file_path_opt.is_some() {
                        return Err(format!("Only one input file path argument is expected"));
                    }

                    input_file_path_opt.replace(arg);
                }
            }
        }

        match input_file_path_opt {
            Some(input_file_path) => Ok(Config {
                input_file_path,
                stop_after_lexer,
                stop_after_parser,
                stop_after_assembly_generation
            }),
            None => {
                Err(format!("Missing input filename"))
            }
        }
    }
}
