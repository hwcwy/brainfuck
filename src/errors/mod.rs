use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum MyError {
    Io(io::Error),
    Parse(std::num::ParseIntError),
    Compile(CompileError),
    Custom(String),
}

#[derive(Debug)]
pub struct CompileError {
    pub line: u32,
    pub col: u32,
    pub kind: CompileErrorKind,
}

#[derive(Debug)]
pub enum CompileErrorKind {
    UnclosedLeftBracket,
    UnexpectedRightBracket,
}

impl Error for CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at line {}:{}", self.kind, self.line, self.col)
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Io(err) => write!(f, "IO error: {}", err),
            MyError::Parse(err) => write!(f, "Parse error: {}", err),
            MyError::Compile(err) => write!(f, "Compile error: {}", err),
            MyError::Custom(err) => write!(f, "{}", err),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::Io(err) => Some(err),
            MyError::Parse(err) => Some(err),
            MyError::Compile(err) => Some(err),
            MyError::Custom(_) => None,
        }
    }
}

impl From<io::Error> for MyError {
    fn from(err: io::Error) -> MyError {
        MyError::Io(err)
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(err: std::num::ParseIntError) -> MyError {
        MyError::Parse(err)
    }
}

impl From<CompileError> for MyError {
    fn from(err: CompileError) -> MyError {
        MyError::Compile(err)
    }
}
