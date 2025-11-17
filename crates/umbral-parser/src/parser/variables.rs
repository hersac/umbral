use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_variable(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let tipo = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        p.parsear_tipo()?
    } else {
        None
    };
    if !p.coincidir(|t| matches!(t, LexToken::Asignacion)) {
        return Err(ParseError::nuevo(
            "Se esperaba '=' en declaracion variable",
            p.posicion,
        ));
    }
    let valor = crate::parser::expresiones::parsear_expresion_principal(p)?;
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::DeclaracionVariable(DeclaracionVariable {
        nombre,
        tipo,
        valor,
    }))
}
