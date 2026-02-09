use log::{info, trace, warn, error};

use crate::config::Config;

mod lexer;
use lexer::Lexer;
use lexer::Token;
mod parser;
mod tacky;
mod codegen;
mod code_emitter;


pub fn run_lexer_test(config: &Config, lexer: &mut Lexer) -> Result<(), String>
{
    let mut no_tokens = true;

    loop {
        let peeked_token = lexer.peek_token()?;
        info!("Peeked Token: '{:?}'", peeked_token);
        let peeked_token = lexer.peek_token()?;
        info!("Peeked Token: '{:?}'", peeked_token);
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

pub fn run(config: &Config, input_file_path: &str, output_file_path: &str) -> Result<(), String>
{
    let mut lexer = Lexer::new(input_file_path)?;

    if config.stop_after_lexer {
        run_lexer_test(config, &mut lexer)?;
        info!("Stopped after lexer");
        return Ok(());
    }


    match parser::parse_program(&mut lexer) {
        Ok(prog_ast) => {
            parser::pretty_print_ast(&prog_ast);
            if config.stop_after_parser {
                info!("Stopped after parser");
                return Ok(());
            }

            let prog_tacky_ast = tacky::emit_tacky_program(&prog_ast)?;
            tacky::pretty_print_ast(&prog_tacky_ast);
            if config.stop_after_tacky_generation {
                info!("Stopped after TACKY generation");
                return Ok(());
            }

            let mut prog_code_ast = codegen::generate_code(&prog_tacky_ast)?;
            codegen::pretty_print_ast(&prog_code_ast);

            let stack_allocation_size = codegen::replace_pseudo_operands(&mut prog_code_ast)?;
            codegen::pretty_print_ast(&prog_code_ast);

            codegen::fixup_instructions(&mut prog_code_ast, stack_allocation_size)?;
            codegen::pretty_print_ast(&prog_code_ast);

            if config.stop_after_assembly_generation {
                info!("Stopped after assembly generation");
                return Ok(());
            }

            match code_emitter::emit_code(&prog_code_ast, output_file_path) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Code emission error: {}", e))
            }
        },
        Err(msg) => { return Err(format!("ERROR: {msg}")); }
    }
}