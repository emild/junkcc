use log::{info, trace, warn, error};

use crate::config::Config;

mod lexer;
use lexer::Lexer;
use lexer::Token;
mod parser;
mod tacky;
mod codegen;


pub fn run_lexer_test(lexer: &mut Lexer, input_file_path: &str) -> Result<(), String>
{
    let mut no_tokens = true;

    loop {
        // let peeked_token = lexer.peek_token()?;
        // info!("Peeked Token: '{:?}'", peeked_token);
        // let peeked_token = lexer.peek_token()?;
        // info!("Peeked Token: '{:?}'", peeked_token);
        let token = lexer.get_token()?;
        info!("Token: '{:?}'", token);
        match token {
            Token::EOS => { break; },
            _ => no_tokens = false
        };
    }

    if no_tokens {
        return Err(format!("No tokens found in input file: '{}'", input_file_path));
    }

    Ok(())
}

pub fn run(config: &Config, input_file_path: &str, output_file_path: &str) -> Result<(), String>
{
    let mut lexer = Lexer::new(input_file_path)?;

    if config.stop_after_lexer {
        run_lexer_test(&mut lexer, input_file_path)?;
        info!("Stopped after lexer");
        return Ok(());
    }


    match parser::parse_program(&mut lexer) {
        Ok(prog_ast) => {
            println!("\nAFTER STAGE: PARSE");
            parser::pretty_print_ast(&prog_ast);
            if config.stop_after_parser {
                info!("Stopped after parser");
                return Ok(());
            }

            let (prog_ast, symbol_table) = parser::semantic_analysis(&prog_ast)?;
            println!("\nAFTER STAGE: SEMANTIC ANALYSIS");
            parser::pretty_print_ast(&prog_ast);
            if config.stop_after_semantic_analysis {
                info!("Stopped after semantic analysis");
                return Ok(());
            }

            let prog_tacky_ast = tacky::generate_tacky_ast(&prog_ast, &symbol_table)?;
            println!("\nAFTER STAGE: TACKY GENERATION");
            tacky::pretty_print_tacky_ast(&prog_tacky_ast);
            if config.stop_after_tacky_generation {
                info!("Stopped after TACKY generation");
                return Ok(());
            }

            let mut prog_code_ast = codegen::generate_code_ast(&prog_tacky_ast)?;

            println!("\nAFTER STAGE: CODE GENERATION");
            codegen::pretty_print_code_ast(&prog_code_ast);

            codegen::replace_pseudo_operands(&mut prog_code_ast)?;

            println!("\nAFTER STAGE: PSEUDO REGISTERS REPLACEMENT");
            codegen::pretty_print_code_ast(&prog_code_ast);

            if config.stop_after_assembly_generation {
                info!("Stopped after assembly generation");
                return Ok(());
            }

            match codegen::emit_code(&prog_code_ast, &symbol_table, output_file_path) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Code emission error: {}", e))
            }
        },
        Err(msg) => { return Err(format!("ERROR: {msg}")); }
    }
}
