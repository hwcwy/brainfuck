use brainfuck_hcy::{run, Config, MyError};
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
