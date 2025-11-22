use crate::error::ParseError;
use umbral_lexer::Token as LexToken;

pub fn error(mensaje: &str, posicion: usize) -> ParseError {
    ParseError::nuevo(mensaje, posicion)
}

pub fn token_es_identificador_o_cadena(t: &LexToken) -> bool {
    matches!(t, LexToken::Identificador(_) | LexToken::Cadena(_) | LexToken::CadenaLiteral(_))
}
