use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_tprint(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo(
            "Se esperaba '(' despues de tprint",
            p.posicion,
        ));
    }
    let valor = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::LlamadoTPrint(LlamadoTPrint { valor }))
}

pub fn parsear_llamado_funcion(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' en llamada", p.posicion));
    }
    let mut argumentos = Vec::new();
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        loop {
            argumentos.push(crate::parser::expresiones::parsear_expresion_principal(p)?);
            if p.coincidir(|t| matches!(t, LexToken::Coma)) {
                continue;
            }
            if p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                break;
            }
            return Err(ParseError::nuevo("Se esperaba ',' o ')'", p.posicion));
        }
    }
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::LlamadoFuncion(LlamadoFuncion {
        nombre,
        argumentos,
    }))
}

pub fn parsear_asignacion(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    
    let objetivo = if p.coincidir(|t| matches!(t, LexToken::Punto)) {
        let propiedad = p.parsear_identificador_consumir()?;
        ObjetivoAsignacion::Propiedad {
            objeto: Box::new(Expresion::Identificador(nombre)),
            propiedad,
        }
    } else {
        ObjetivoAsignacion::Variable(nombre)
    };
    
    if !p.coincidir(|t| matches!(t, LexToken::Asignacion)) {
        return Err(ParseError::nuevo(
            "Se esperaba '=' en asignacion",
            p.posicion,
        ));
    }
    let valor = crate::parser::expresiones::parsear_expresion_principal(p)?;
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Asignacion(Asignacion { objetivo, valor }))
}

pub fn parsear_return(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo(
            "Se esperaba '(' despues de return",
            p.posicion,
        ));
    }
    let valor = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Return(valor))
}
