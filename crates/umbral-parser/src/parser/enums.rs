use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::expresiones;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_enum(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{' en enum"));
    }
    let mut variantes = Vec::new();
    loop {
        match p.peekear() {
            Some(LexToken::Identificador(nombre_variante)) => {
                let nombre_var = nombre_variante.clone();
                p.avanzar();
                
                let valor = if p.coincidir(|t| matches!(t, LexToken::Asignacion)) {
                    let expr = expresiones::parsear_expresion_principal(p)?;
                    Some(expr)
                } else {
                    None
                };
                
                variantes.push(VarianteEnum {
                    nombre: nombre_var,
                    valor,
                });
                
                if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
                    if !matches!(p.peekear(), Some(LexToken::LlaveDer)) {
                        return Err(p.crear_error("Se esperaba ',' o '}' en enum"));
                    }
                }
            }
            Some(LexToken::LlaveDer) => {
                p.avanzar();
                break;
            }
            Some(_) => return Err(p.crear_error("Entrada no valida en enum")),
            None => return Err(p.crear_error("Fin inesperado en enum")),
        }
    }
    
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Enum(DeclaracionEnum { nombre, variantes, exportado }))
}
