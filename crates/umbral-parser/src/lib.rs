pub mod ast;
pub mod error;
pub mod parser;

pub use ast::*;
pub use error::ParseError;
pub use parser::Parser;

use umbral_lexer::Token;

pub fn parsear_programa(tokens: Vec<Token>) -> Result<Programa, ParseError> {
    let mut p = Parser::nuevo(tokens);
    p.parsear_programa()
}
