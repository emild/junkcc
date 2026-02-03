use log::{info, trace, warn, error};

use crate::config::Config;

mod lexer;
use lexer::Lexer;
use lexer::Token;

pub fn run(config: &Config, input_file_path: &str, _output_file_path: &str) -> Result<(), String>
{
    let mut lexer = Lexer::new(input_file_path)?;
    let mut no_tokens = true;

    loop {
        let token = lexer.get_token()?;
        info!("Token: '{:?}'", token);
        match token {
            Token::EOS => { break; },
            _ => no_tokens = false
        };
    }

    if no_tokens {
        return Err(format!("No tokens found in input file: '{input_file_path}'"));
    }

    if config.stop_after_lexer {
        info!("Stopped after lexer");
    } 

    Ok(())
}