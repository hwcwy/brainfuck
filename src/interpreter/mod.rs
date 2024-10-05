use crate::errors::MyError;
use crate::parse_args::Config;
use crate::{raw_code_to_token_vec, Memory, OutputMode, Token, IO};
use std::io::{self, Write};

static REPL_HELP: &str = "Commands:
exit         Exit REPL mode
clear        Clear memory
v            Enable verbose mode
uv           Disable verbose mode
? | help     Print this";

#[derive(Debug)]
struct ExecQueue {
    view: Vec<Token>,
    ptr: usize,
}

impl ExecQueue {
    fn new(token_vec: Vec<Token>) -> Self {
        ExecQueue {
            view: token_vec,
            ptr: 0,
        }
    }

    fn jump_forward(&mut self, n: u32) {
        self.ptr = n as usize
    }

    fn jump_back(&mut self, n: u32) {
        self.ptr = n as usize
    }

    fn next_token(&mut self) -> Option<Token> {
        let token = match self.view.get(self.ptr) {
            Some(&t) => t,
            None => return None,
        };
        self.ptr += 1;
        Some(token)
    }
}

pub fn run(mut config: Config) -> Result<(), MyError> {
    let mut runtime_memory = Memory::new(config.cell_max);
    let with_buffer = config.output_mode == OutputMode::Bulk || config.verbose;
    let mut io = IO::new(config.output_mode, with_buffer);

    if config.repl_mode {
        println!("{}", REPL_HELP);
        println!();
        println!("{}", runtime_memory);
        println!();
        loop {
            match repl_mode(&mut runtime_memory, &mut io, &mut config.verbose) {
                Ok(_) => break,
                Err(e) => {
                    println!("Recovering from error: {e}");
                    println!("{}", runtime_memory);
                    println!("{}", io.buffer_to_string());
                }
            };
        }
    } else {
        normal_mode(
            runtime_memory,
            io,
            config.verbose,
            ExecQueue::new(config.token_vec),
        )?;
    }
    Ok(())
}

fn normal_mode(
    mut runtime_memory: Memory,
    mut io: IO,
    verbose: bool,
    mut exec_queue: ExecQueue,
) -> Result<(), MyError> {
    let should_print_individually = !verbose && io.output_mode == OutputMode::Individually;
    while let Some(token) = exec_queue.next_token() {
        if verbose {
            print!("{} ", runtime_memory);
        }
        match token {
            Token::PtrIncrease(n) => runtime_memory.ptr_increase(n),
            Token::PtrDecrease(n) => runtime_memory.ptr_decrease(n)?,
            Token::DataIncrease(n) => runtime_memory.data_increase(n),
            Token::DataDecrease(n) => runtime_memory.data_decrease(n),
            Token::JumpForward(n) => {
                if runtime_memory.output() == 0 {
                    exec_queue.jump_forward(n);
                }
            }
            Token::JumpBack(n) => {
                if runtime_memory.output() != 0 {
                    exec_queue.jump_back(n);
                }
            }
            Token::Output => {
                let char = io.output(&runtime_memory)?;
                if should_print_individually {
                    print!("{}", char);
                    if let Err(e) = io::stdout().flush() {
                        return Err(MyError::Io(e));
                    }
                }
            }
            Token::Input => io.input(&mut runtime_memory)?,
        };
        if verbose {
            if token != Token::Input {
                println!("{:?}", token);
            }
            println!("{}", io.buffer_to_string());
        }
    }
    if !verbose && io.output_mode == OutputMode::Bulk {
        print!("{}", io.buffer_to_string());
    };
    Ok(())
}

fn repl_mode(runtime_memory: &mut Memory, io: &mut IO, verbose: &mut bool) -> Result<(), MyError> {
    io.output_buffer = Some(Vec::new());
    loop {
        print!("> ");
        if let Err(e) = io::stdout().flush() {
            return Err(MyError::Io(e));
        }

        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_line(&mut buffer) {
            return Err(MyError::Io(e));
        }
        let buffer = buffer.trim_end();

        //EOF
        if buffer.as_bytes().is_empty() {
            println!()
        }

        match buffer.trim() {
            "exit" => break,
            "clear" => {
                runtime_memory.view = vec![0];
                io.output_buffer = Some(Vec::new());
            }
            "v" => *verbose = true,
            "uv" => *verbose = false,
            "?" | "help" => println!("{REPL_HELP}"),
            _ => {}
        }

        let mut exec_queue = ExecQueue::new(raw_code_to_token_vec(buffer)?);
        let should_print_individually = !*verbose && io.output_mode == OutputMode::Individually;

        while let Some(token) = exec_queue.next_token() {
            if *verbose {
                print!("{} ", runtime_memory);
            }
            match token {
                Token::PtrIncrease(n) => runtime_memory.ptr_increase(n),
                Token::PtrDecrease(n) => runtime_memory.ptr_decrease(n)?,
                Token::DataIncrease(n) => runtime_memory.data_increase(n),
                Token::DataDecrease(n) => runtime_memory.data_decrease(n),
                Token::JumpForward(n) => {
                    if runtime_memory.output() == 0 {
                        exec_queue.jump_forward(n);
                    }
                }
                Token::JumpBack(n) => {
                    if runtime_memory.output() != 0 {
                        exec_queue.jump_back(n);
                    }
                }
                Token::Output => {
                    let char = io.output(runtime_memory)?;
                    if should_print_individually {
                        print!("{}", char);
                        if let Err(e) = io::stdout().flush() {
                            return Err(MyError::Io(e));
                        }
                    }
                }
                Token::Input => io.input(runtime_memory)?,
            };
            if *verbose {
                if token != Token::Input {
                    println!("{:?}", token);
                }
                println!("{}", io.buffer_to_string());
            }
        }
        println!("\r{}", runtime_memory);
        println!("{}", io.buffer_to_string());
    }
    Ok(())
}
