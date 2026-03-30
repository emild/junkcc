pub mod config;
pub mod driver;
pub mod compiler;

use config::Config;
use std::path;


const PP_EXT: &str = "pp.c";
const AS_EXT: &str = "s";
const OBJ_EXT: &str = "o";
const EXE_EXT: &str = "";

fn get_filename(input_file_path: &str, ext: &str) -> String
{
    let p = path::Path::new(input_file_path);
    let parent = p.parent();
    let stem = p.file_stem();

    let mut pb = path::PathBuf::from(parent.unwrap());
    pb.push(stem.unwrap_or_default());
    if ext.len() > 0 {
        pb.add_extension(ext);
    }

    String::from(pb.as_os_str().to_str().unwrap_or_default())
}



fn get_pp_filename(input_file_path: &str) -> String
{
    return get_filename(input_file_path, PP_EXT);
}

fn get_as_filename(input_file_path: &str) -> String
{
    return get_filename(input_file_path, AS_EXT);
}

fn get_obj_filename(input_file_path: &str) -> String
{
    return get_filename(input_file_path, OBJ_EXT);
}

fn get_exe_filename(input_file_path: &str) -> String
{
    return get_filename(input_file_path, EXE_EXT);
}


pub fn run(config: Config) -> Result<(), String>
{
    let mut obj_file_paths = vec![];

    let stop_early = config.stop_after_lexer ||
                    config.stop_after_parser ||
                    config.stop_after_semantic_analysis ||
                    config.stop_after_tacky_generation ||
                    config.stop_after_assembly_generation;

    let exe_file_path = get_exe_filename(&config.input_file_paths[0]);

    for input_file_path in &config.input_file_paths {
        match config::get_file_type(&input_file_path) {
            Some(config::FileType::CSource) => {
                let pp_file_path = get_pp_filename(&input_file_path);
                driver::preprocess(&input_file_path, &pp_file_path)?;
                let as_file_path = get_as_filename(&input_file_path);
                driver::compile(&config, &pp_file_path, &as_file_path)?;
                if  stop_early {

                    continue;
                }

                let obj_file_path = get_obj_filename(&input_file_path);
                driver::assemble(&as_file_path, &obj_file_path)?;

                obj_file_paths.push(obj_file_path);
            },
            Some(config::FileType::Assembly) => {
                let obj_file_path = get_obj_filename(&input_file_path);
                let as_file_path = input_file_path;
                driver::assemble(&as_file_path, &obj_file_path)?;

                obj_file_paths.push(obj_file_path);
            },
            Some(config::FileType::Object) => {
                obj_file_paths.push(input_file_path.clone());
            },
            None => {
                panic!("Unsupported File type: '{}'", input_file_path);
            }

        }
    }

    if !stop_early && !config.do_not_link {

        driver::link(&exe_file_path, &obj_file_paths)?;
    }

    Ok(())
}