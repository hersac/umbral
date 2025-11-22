use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_objeto_principal(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut pares = Vec::new();
    if parser.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
        return Ok(Expresion::Objeto(pares));
    }
    loop {
        let clave = match parser.peekear() {
            Some(LexToken::Identificador(n)) => Some(n.clone()),
            Some(LexToken::Cadena(s) | LexToken::CadenaLiteral(s)) => Some(s.clone()),
            _ => None,
        };
        if let Some(k) = clave {
            parser.avanzar();
            if !parser.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
                return Err(ParseError::nuevo(
                    "Se esperaba '=>' en objeto",
                    parser.posicion,
                ));
            }
            let valor = crate::parser::expresiones::parsear_expresion_principal(parser)?;
            pares.push((k, valor));
            if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
                continue;
            }
            if parser.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
                break;
            }
            return Err(ParseError::nuevo(
                "Se esperaba ',' o ']' en objeto",
                parser.posicion,
            ));
        } else {
            return Err(ParseError::nuevo(
                "Clave de objeto esperada",
                parser.posicion,
            ));
        }
    }
    Ok(Expresion::Objeto(pares))
}
