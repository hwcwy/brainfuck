use brainfuck::errors::MyError;
use brainfuck::parse_args::Config;
use brainfuck::run;
use std::env::args;

fn main() {
    if let Err(e) = exec() {
        eprintln!("Error: {}", e);
    }
}

fn exec() -> Result<(), MyError> {
    let config = Config::from(args())?;
    run(config)?;
    Ok(())
}
