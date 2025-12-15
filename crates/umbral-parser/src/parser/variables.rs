use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn parsear_tipo_opcional(parseador: &mut Parser) -> Result<Option<Tipo>, ParseError> {
    if parseador.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        return Ok(parseador.parsear_tipo()?);
    }
    Ok(None)
}

pub fn parsear_declaracion_variable(parseador: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = parseador.parsear_identificador_consumir()?;
    let tipo = parsear_tipo_opcional(parseador)?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::Asignacion)) {
        return Err(parseador.crear_error("Se esperaba '=' en declaracion variable"));
    }
    
    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    
    Ok(Sentencia::DeclaracionVariable(DeclaracionVariable {
        nombre,
        tipo,
        valor,
        exportado,
    }))
}
