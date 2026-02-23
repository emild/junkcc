pub mod config;
pub mod driver;
pub mod compiler;

use config::Config;

pub fn run(config: Config) -> Result<(), String>
{
    driver::preprocess(&config)?;
    driver::compile(&config)?;
    if config.stop_after_lexer ||
        config.stop_after_parser ||
        config.stop_after_semantic_analysis ||
        config.stop_after_tacky_generation ||
        config.stop_after_assembly_generation {
        return Ok(());
    }
    driver::assemble(&config)?;
    driver::link(&config)?;
    Ok(())
}