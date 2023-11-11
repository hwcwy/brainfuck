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

    fn get_token(&mut self) -> Option<Token> {
        let token = match self.view.get(self.ptr) {
            Some(&t) => t,
            None => return None,
        };
        Some(token)
    }

    fn ptr_increase(&mut self) {
        self.ptr += 1;
    }
}

pub fn run(mut config: Config) -> Result<(), MyError> {
    let mut runtime_memory = Memory::new(config.cell_max);
    let mut io = IO::new(config.output_mode);

    if config.repl_mode {
        println!("{}", REPL_HELP);
        println!("\n{}", runtime_memory);
        loop {
            match repl_mode(&mut runtime_memory, &mut io, &mut config.verbose) {
                Ok(_) => break,
                Err(e) => {
                    println!("Recover from error: {e}");
                    print!("{}", runtime_memory);
                    print!("\n{}", io.buffer_to_string());
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
    while let Some(token) = exec_queue.get_token() {
        if verbose {
            match io.output_mode {
                OutputMode::Individually => {
                    println!("{} {:?}", runtime_memory, token);
                    println!("{}", io.buffer_to_string());
                }
                OutputMode::Bulk => {
                    println!("{} {:?}", runtime_memory, token);
                }
            }
        }
        match token {
            Token::PtrIncrease(n) => runtime_memory.ptr_increase(n),
            Token::PtrDecreate(n) => runtime_memory.ptr_decrease(n)?,
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
            Token::Output => io.output(&runtime_memory)?,
            Token::Input => io.input(&mut runtime_memory)?,
        };
        exec_queue.ptr_increase();
    }
    if io.output_mode == OutputMode::Bulk {
        print!("{}", io.buffer_to_string());
    };
    Ok(())
}

fn repl_mode(runtime_memory: &mut Memory, io: &mut IO, verbose: &mut bool) -> Result<(), MyError> {
    io.output_mode = OutputMode::Bulk;
    loop {
        print!("\n> ");
        match io::stdout().flush() {
            Ok(_) => {}
            Err(e) => return Err(MyError::Io(e)),
        }

        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_line(&mut buffer) {
            return Err(MyError::Io(e));
        }

        //EOF
        if buffer.as_bytes().is_empty() {
            println!()
        }

        match buffer.trim() {
            "exit" => break,
            "clear" => {
                runtime_memory.view = vec![0];
                io.output_buffer.clear();
            }
            "v" => *verbose = true,
            "uv" => *verbose = false,
            "?" | "help" => println!("{REPL_HELP}"),
            _ => {}
        }

        let mut exec_queue = ExecQueue::new(raw_code_to_token_vec(&buffer)?);

        while let Some(token) = exec_queue.get_token() {
            if *verbose {
                println!("{} {:?}", runtime_memory, token);
            }
            match token {
                Token::PtrIncrease(n) => runtime_memory.ptr_increase(n),
                Token::PtrDecreate(n) => runtime_memory.ptr_decrease(n)?,
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
                Token::Output => io.output(runtime_memory)?,
                Token::Input => io.input(runtime_memory)?,
            };
            exec_queue.ptr_increase();
        }

        print!("{}", runtime_memory);
        print!("\n{}", io.buffer_to_string());
    }
    Ok(())
}
