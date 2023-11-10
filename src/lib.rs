pub mod errors;
mod interpreter;
pub mod parse_args;

use errors::MyError;
use parse_args::Config;
use std::fmt;
use std::io::{self, Write};

pub fn run(mut config: Config) -> Result<(), MyError> {
    config.token_vec = raw_code_to_token_vec(&config.raw_code)?;
    match config.show_ir {
        true => show_ir(config.token_vec),
        false => interpreter::run(config)?,
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum Token {
    PtrIncrease(u32),
    PtrDecreate(u32),
    DataIncrease(u32),
    DataDecrease(u32),
    JumpForward(u32),
    JumpBack(u32),
    Output,
    Input,
}

#[derive(Debug, PartialEq)]
pub enum OutputMode {
    Individually,
    Bulk,
}

struct IO {
    output_mode: OutputMode,
    output_buffer: Vec<u32>,
}

impl IO {
    fn new(output_mode: OutputMode) -> Self {
        IO {
            output_mode,
            output_buffer: Vec::new(),
        }
    }

    fn buffer_to_string(&self) -> String {
        let mut result = String::new();
        for &n in self.output_buffer.iter() {
            result.push(char::from_u32(n).unwrap());
        }
        result
    }

    fn output(&mut self, runtime_memory: &Memory) -> Result<(), MyError> {
        let n = runtime_memory.output();
        let char = match char::from_u32(n) {
            Some(c) => c,
            None => {
                return Err(MyError::Custom(format!(
                    "Invalid Unicode scalar value: {}",
                    n
                )))
            }
        };
        match self.output_mode {
            OutputMode::Individually => {
                self.output_buffer.push(n);
                print!("{}", char);
                match io::stdout().flush() {
                    Ok(_) => {}
                    Err(e) => return Err(MyError::Io(e)),
                }
            }
            OutputMode::Bulk => {
                self.output_buffer.push(n);
            }
        };
        Ok(())
    }

    fn input(&self, runtime_memory: &mut Memory) -> Result<(), MyError> {
        print!("\nInput:");
        match io::stdout().flush() {
            Ok(_) => {}
            Err(e) => return Err(MyError::Io(e)),
        }
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_line(&mut buffer) {
            return Err(MyError::Io(e));
        }
        let n = input_to_u32(buffer)?;
        if n > runtime_memory.cell_max {
            return Err(MyError::Custom(format!(
                "Input value {} exceeds the maximum cell value {}",
                n, runtime_memory.cell_max
            )));
        }
        runtime_memory.input(n);
        if self.output_mode == OutputMode::Individually {
            print!("{}", self.buffer_to_string());
        };
        Ok(())
    }
}

struct Memory {
    view: Vec<u32>,
    ptr: u32,
    cell_max: u32,
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (i, &cell) in self.view.iter().enumerate() {
            if i == self.ptr as usize {
                s.push('>');
            }
            s.push_str(&cell.to_string());
            if i != self.view.len() - 1 {
                s.push_str(", ");
            }
        }
        write!(f, "[{}]", s)
    }
}

impl Memory {
    fn new(cell_max: u32) -> Self {
        Memory {
            view: vec![0],
            ptr: 0,
            cell_max,
        }
    }

    fn ptr_increase(&mut self, n: u32) {
        self.ptr += n;
        if self.view.len() <= self.ptr as usize {
            self.view.resize(self.ptr as usize + 1, 0);
        }
    }

    fn ptr_decrease(&mut self, n: u32) -> Result<(), MyError> {
        if n > self.ptr {
            return Err(MyError::Custom(format!(
                "The current pointer is at position {} and cannot move left by {} positions",
                self.ptr, n
            )));
        }
        self.ptr -= n;
        Ok(())
    }

    fn data_increase(&mut self, n: u32) {
        let v = &mut self.view[self.ptr as usize];
        *v = if self.cell_max - *v >= n {
            *v + n
        } else {
            n - (self.cell_max - *v + 1)
        };
    }

    fn data_decrease(&mut self, n: u32) {
        let v = &mut self.view[self.ptr as usize];
        *v = if *v >= n {
            *v - n
        } else {
            self.cell_max - (n - *v - 1)
        };
    }

    fn output(&self) -> u32 {
        self.view[self.ptr as usize]
    }

    fn input(&mut self, n: u32) {
        self.view[self.ptr as usize] = n;
    }
}

fn raw_code_to_token_vec(raw_code: &str) -> Result<Vec<Token>, MyError> {
    let mut vec = Vec::new();
    let mut stack = Vec::new();
    let chars = raw_code.chars().into_iter().enumerate();
    for (i, char) in chars {
        match char {
            '>' => {
                if let Some(Token::PtrIncrease(n)) = vec.last_mut() {
                    *n += 1;
                } else {
                    vec.push(Token::PtrIncrease(1));
                }
            }
            '<' => {
                if let Some(Token::PtrDecreate(n)) = vec.last_mut() {
                    *n += 1;
                } else {
                    vec.push(Token::PtrDecreate(1));
                }
            }
            '+' => {
                if let Some(Token::DataIncrease(n)) = vec.last_mut() {
                    *n += 1;
                } else {
                    vec.push(Token::DataIncrease(1));
                }
            }
            '-' => {
                if let Some(Token::DataDecrease(n)) = vec.last_mut() {
                    *n += 1;
                } else {
                    vec.push(Token::DataDecrease(1));
                }
            }
            '.' => vec.push(Token::Output),
            ',' => vec.push(Token::Input),
            '[' => {
                vec.push(Token::JumpForward(0));
                stack.push(vec.len() as u32 - 1);
            }
            ']' => {
                if let Some(start) = stack.pop() {
                    vec.push(Token::JumpBack(start));
                    *vec.get_mut(start as usize).unwrap() =
                        Token::JumpForward(vec.len() as u32 - 1);
                } else {
                    return Err(MyError::Custom(format!(
                        "Unmatched JumpBack found at [{}]",
                        i
                    )));
                }
            }
            _ => (),
        }
    }

    if !stack.is_empty() {
        return Err(MyError::Custom(format!(
            "Unmatched JumpForward found at {:?}",
            stack
        )));
    }

    Ok(vec)
}

fn input_to_u32(mut s: String) -> Result<u32, MyError> {
    let trimed = s.trim();
    if trimed.ends_with("u32") {
        s.truncate(trimed.len() - 3);
        return match s.parse::<u32>() {
            Ok(value) => Ok(value),
            Err(e) => Err(MyError::Parse(e)),
        };
    }

    match s.as_bytes() {
        x if x.ends_with(&[13, 10]) => {
            // Windows
            match s.chars().count() {
                3 => {
                    let c = s.chars().next().unwrap();
                    Ok(c as u32)
                }
                2 => Ok('\n' as u32),
                _ => Err(MyError::Custom(
                    "The length of the input string is greater than 1, unable to parse into char"
                        .to_string(),
                )),
            }
        }

        x if x.ends_with(&[10]) => {
            // Unix & Unix like
            match s.chars().count() {
                2 => {
                    let c = s.chars().next().unwrap();
                    Ok(c as u32)
                }
                1 => Ok('\n' as u32),
                _ => Err(MyError::Custom(
                    "The length of the input string is greater than 1, unable to parse into char"
                        .to_string(),
                )),
            }
        }

        _ => {
            // EOF
            Ok(0)
        }
    }
}

fn show_ir(token_vec: Vec<Token>) {
    for token in token_vec {
        match token {
            Token::PtrIncrease(n) => println!("PtrIncrease  {}", n),
            Token::PtrDecreate(n) => println!("PtrDecreate  {}", n),
            Token::DataIncrease(n) => println!("DataIncrease {}", n),
            Token::DataDecrease(n) => println!("DataDecrease {}", n),
            Token::JumpForward(n) => println!("JumpForward  {}", n),
            Token::JumpBack(n) => println!("JumpBack     {}", n),
            Token::Output => println!("Output"),
            Token::Input => println!("Input"),
        }
    }
}
