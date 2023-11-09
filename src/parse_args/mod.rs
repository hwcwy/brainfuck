use crate::errors::MyError;
use crate::OutputMode;
use crate::Token;
use std::{env::Args, fs};

static HELP: &str = "Usage:
[code]              Use the last argument as the code
-f [path]           Read code from a file
--REPL | --repl     Start in REPL mode
--bulk              Bulk output mode
--cell [u8|u16|u32] Set the cell size
-v | --verbose      Display verbose information
--IR                Display intermediate representation of the code
[input]             Input can be a string ending with 'u32' to be parsed as uint32";

pub struct Config {
    pub raw_code: String,
    pub output_mode: OutputMode,
    pub token_vec: Vec<Token>,
    pub cell_max: u32,
    pub verbose: bool,
    pub repl_mode: bool,
    pub show_ir: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            raw_code: String::new(),
            output_mode: OutputMode::Individually,
            token_vec: Vec::new(),
            cell_max: 255,
            verbose: false,
            repl_mode: false,
            show_ir: false,
        }
    }

    pub fn from(args: Args) -> Result<Config, MyError> {
        let mut config = Config::new();
        let args: Vec<String> = args.skip(1).collect();
        let mut args_iter = args.into_iter().peekable();

        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-h" | "-H" | "--help" => {
                    println!("{HELP}");
                    std::process::exit(0);
                }
                "-f" => {
                    if let Some(file_path) = args_iter.peek() {
                        match fs::read_to_string(file_path) {
                            Ok(code) => {
                                if code.is_empty() {
                                    return Err(MyError::Custom("No code found".to_string()));
                                }
                                config.raw_code = code;
                            }
                            Err(e) => return Err(MyError::Io(e)),
                        }
                    } else {
                        return Err(MyError::Custom("File path not specified".to_string()));
                    }
                }
                "-v" | "--verbose" => config.verbose = true,
                "--bulk" => config.output_mode = OutputMode::Bulk,
                "--cell" => {
                    if let Some(cell_size_type) = args_iter.peek() {
                        match cell_size_type.as_str() {
                            "u8" => config.cell_max = 255,
                            "u16" => config.cell_max = 65535,
                            "u32" => config.cell_max = 4294967295,
                            _ => {
                                return Err(MyError::Custom(format!(
                                    "Invalid cell size type {cell_size_type}"
                                )))
                            }
                        }
                    } else {
                        return Err(MyError::Custom("Cell size type not specified".to_string()));
                    }
                }
                "--REPL" | "--repl" => {
                    config.repl_mode = true;
                }
                "--IR" => config.show_ir = true,

                _ => {
                    if arg.starts_with('-') {
                        println!("{HELP}")
                    }
                    if config.raw_code.is_empty() {
                        config.raw_code = arg;
                    }
                }
            }
        }

        if config.raw_code.is_empty() && !config.repl_mode {
            return Err(MyError::Custom("No code found".to_string()));
        }

        Ok(config)
    }
}
