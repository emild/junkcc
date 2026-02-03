use std::env;
use junkcc::config::Config;
use log::{trace};

fn main() -> Result<(), String> {
    env_logger::init();

    let cfg = 
        Config::build(env::args())
            .unwrap_or_else(|err| { 
                eprintln!("{err}");
                std::process::exit(1);
            });


    trace!("{cfg:?}");

    junkcc::run(cfg)
        .unwrap_or_else(|err| { 
                eprintln!("{err}");
                std::process::exit(1);
    });

    Ok(())
}
