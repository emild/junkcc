
use std::{collections::HashMap, fs::File};

#[derive(Debug)]
pub struct Config {
    pub input_file_paths: Vec<String>,
    pub stop_after_lexer: bool,
    pub stop_after_parser: bool,
    pub stop_after_semantic_analysis: bool,
    pub stop_after_tacky_generation: bool,
    pub stop_after_assembly_generation: bool,
    pub do_not_link: bool
}

pub enum FileType {
    Assembly,
    CSource,
    Object
}


const VALID_EXTENSIONS: [(&str, FileType);4]   = [
    (".c", FileType::CSource),
    (".s", FileType::Assembly),
    (".S", FileType::Assembly),
    (".o", FileType::Object)
];


fn filename_has_valid_extension(file_name: &String) -> bool {
    for (ext, _) in VALID_EXTENSIONS {
        if file_name.ends_with(ext) {
            return true;
        }
    }

    return false;
}


pub fn get_file_type(file_name: &String) -> Option<FileType> {
    for (ext, file_type) in VALID_EXTENSIONS {
        if file_name.ends_with(ext) {
            return Some(file_type);
        }
    }

    return None;
}



fn supported_file_types() -> Vec<String> {
    let mut supported_exts = vec![];

    for (ext, _) in VALID_EXTENSIONS {
        supported_exts.push(format!("'{ext}'"));
    }

    return supported_exts;
}



impl Config {
    pub fn build(mut args: impl Iterator<Item=String>) -> Result<Config, String>
    {
        args.next();

        let mut stop_after_lexer = false;
        let mut stop_after_parser = false;
        let mut stop_after_semantic_analysis = false;
        let mut stop_after_tacky_generation = false;
        let mut stop_after_assembly_generation = false;
        let mut do_not_link = false;

        let mut opt_map = HashMap::from([
            (String::from("--lex"), &mut stop_after_lexer),
            (String::from("--parse"), &mut stop_after_parser),
            (String::from("--validate"), &mut stop_after_semantic_analysis),
            (String::from("--tacky"), &mut stop_after_tacky_generation),
            (String::from("--codegen"), &mut stop_after_assembly_generation),
            (String::from("-c"), &mut do_not_link)
        ]);



        let mut input_file_paths = vec![];

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
                    if arg.starts_with("-") {
                        return Err(format!("Invalid option: '{arg}'"));
                    }

                    if arg.len() < 3 {
                        return Err(format!("Input file path must be at least 3 characters '{arg}' has {}", arg.len()))
                    }

                    if !filename_has_valid_extension(&arg) {
                        return Err(format!("Unsupported input file '{}' (supported file type: {})", arg, supported_file_types().join(", ")));
                    }

                    input_file_paths.push(arg);
                }
            }
        }

        if input_file_paths.is_empty() {
            return Err(format!("Missing input filename(s)"));
        }



        Ok(Config {
            input_file_paths: input_file_paths,
            stop_after_lexer,
            stop_after_parser,
            stop_after_semantic_analysis,
            stop_after_tacky_generation,
            stop_after_assembly_generation,
            do_not_link
        })
    }
}
