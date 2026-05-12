
use std::{collections::HashMap, fs::File};

#[derive(Debug)]
pub struct Config {
    pub input_file_paths: Vec<String>,
    pub libraries: Vec<String>, //Should be of format: "-l<foo>"
    pub library_search_paths: Vec<String>, //Should be of format -L<foo>
    pub include_search_paths: Vec<String>, //Should be of format -I<foo>
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

    //accepts both -<opt> <value> and -<opt><value> forms
    fn parse_value_option(
        arg: &String,
        args: &mut impl Iterator<Item=String>,
        opt_map: &mut HashMap<&str, &mut Vec<String>>
    ) -> Result<(), String>
    {
        if !arg.starts_with('-') {
            return Err(format!("Option '{arg}' does not start with '-'"));
        }

        if arg.len()< 2 {
            return Err(format!("Plain '-' is not a valid option"));
        }

        match opt_map.get_mut(&arg[1..2]) {
            None => { return Err(format!("Invalid option: '{arg}'")); },
            Some(str_vec) => {
                let opt_val = if arg.len() > 2 {
                    String::from(&arg[2..])
                }
                else {
                    let next = args.next();
                    if next.is_none() {
                        return Err(format!("Option '{arg}' expects a value, but none was given"));
                    }
                    let arg_val = next.unwrap();
                    if arg_val.starts_with("-") {
                        return Err(format!("Option '{arg}' expects a value, but none was given"));
                    }
                    arg_val
                };

                (*str_vec).push(opt_val);
            }
        }

        Ok(())
    }

    pub fn build(mut args: impl Iterator<Item=String>) -> Result<Config, String>
    {
        args.next();

        let mut stop_after_lexer = false;
        let mut stop_after_parser = false;
        let mut stop_after_semantic_analysis = false;
        let mut stop_after_tacky_generation = false;
        let mut stop_after_assembly_generation = false;
        let mut do_not_link = false;

        let mut flag_opt_map = HashMap::from([
            (String::from("--lex"), &mut stop_after_lexer),
            (String::from("--parse"), &mut stop_after_parser),
            (String::from("--validate"), &mut stop_after_semantic_analysis),
            (String::from("--tacky"), &mut stop_after_tacky_generation),
            (String::from("--codegen"), &mut stop_after_assembly_generation),
            (String::from("-c"), &mut do_not_link)
        ]);



        let mut input_file_paths = vec![];
        let mut include_search_paths: Vec<String> = vec![];
        let mut library_search_paths: Vec<String> = vec![];
        let mut libraries: Vec<String> = vec![];

        let mut value_opt_map = HashMap::from([
            ("I", &mut include_search_paths),
            ("L", &mut library_search_paths),
            ("l", &mut libraries)
        ]);

        loop {
            let arg = args.next();
            if arg.is_none() {
                break;
            }

            let arg = arg.unwrap();

            match flag_opt_map.get_mut(&arg) {
                Some(opt_val) => {
                    if **opt_val {
                        return Err(format!("Duplicate option: '{arg}'"));
                    }
                    **opt_val = true;
                },
                None => {
                    if arg.starts_with("-") {
                        Self::parse_value_option(&arg, &mut args, &mut value_opt_map)?;
                        continue;
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
            include_search_paths,
            library_search_paths,
            libraries,
            stop_after_lexer,
            stop_after_parser,
            stop_after_semantic_analysis,
            stop_after_tacky_generation,
            stop_after_assembly_generation,
            do_not_link
        })
    }
}
