use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_enum(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{' en enum", p.posicion));
    }
    let mut variantes = Vec::new();
    loop {
        match p.avanzar() {
            Some(LexToken::Identificador(v)) => {
                variantes.push(v.clone());
                if p.coincidir(|t| matches!(t, LexToken::Coma)) {
                    continue;
                }
            }
            Some(LexToken::LlaveDer) => break,
            Some(_) => return Err(ParseError::nuevo("Entrada no valida en enum", p.posicion)),
            None => return Err(ParseError::nuevo("Fin inesperado en enum", p.posicion)),
        }
    }
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Enum(DeclaracionEnum { nombre, variantes }))
}
