use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_enum(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{' en enum"));
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
            Some(_) => return Err(p.crear_error("Entrada no valida en enum")),
            None => return Err(p.crear_error("Fin inesperado en enum")),
        }
    }
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Enum(DeclaracionEnum { nombre, variantes, exportado }))
}
