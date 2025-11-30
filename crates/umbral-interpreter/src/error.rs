use std::fmt;

#[derive(Debug, Clone)]
pub enum InterpreterError {
    LexerError(String),
    ParserError(String),
    RuntimeError(String),
    IoError(String),
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterError::LexerError(msg) => write!(f, "Error de Lexer: {}", msg),
            InterpreterError::ParserError(msg) => write!(f, "{}", msg),
            InterpreterError::RuntimeError(msg) => write!(f, "Error de Runtime: {}", msg),
            InterpreterError::IoError(msg) => write!(f, "Error de I/O: {}", msg),
        }
    }
}

impl std::error::Error for InterpreterError {}

impl From<std::io::Error> for InterpreterError {
    fn from(error: std::io::Error) -> Self {
        InterpreterError::IoError(error.to_string())
    }
}

pub type InterpreterResult<T> = Result<T, InterpreterError>;
