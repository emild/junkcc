use log::{info, trace, warn, error};

use crate::config::Config;

mod lexer;
use lexer::Lexer;
use lexer::Token;
mod parser;

use parser::{parse_program, pretty_print_ast};


pub fn run_lexer_test(config: &Config, lexer: &mut Lexer) -> Result<(), String>
{
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
        return Err(format!("No tokens found in input file: '{}'", config.input_file_path));
    }

    Ok(())
}

pub fn run(config: &Config, input_file_path: &str, _output_file_path: &str) -> Result<(), String>
{
    let mut lexer = Lexer::new(input_file_path)?;

    if config.stop_after_lexer {
        run_lexer_test(config, &mut lexer)?;
        info!("Stopped after lexer");
        return Ok(());
    } 


    match parse_program(&mut lexer) {
        Ok(prog) => { pretty_print_ast(&prog); },
        Err(msg) => { return Err(format!("ERROR: {msg}")); }
    }

    Ok(())
}