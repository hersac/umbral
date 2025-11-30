use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn intentar_parsear_instancia_inline(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let inicio_pos = parser.posicion;

    let tipo = match parser.peekear().cloned() {
        Some(LexToken::Identificador(t)) => {
            if t.chars().next().map_or(false, |c| c.is_uppercase())
                && parser.posicion + 1 < parser.tokens.len()
                && matches!(parser.tokens[parser.posicion + 1], LexToken::ParentesisIzq)
            {
                parser.avanzar();
                Some(t)
            } else {
                None
            }
        }
        _ => None,
    };

    let tipo = match tipo {
        Some(t) => t,
        None => {
            parser.posicion = inicio_pos;
            return Err(parser.crear_error("No es una instancia inline"));
        }
    };

    parser.coincidir(|t| matches!(t, LexToken::ParentesisIzq));

    let mut argumentos = Vec::new();
    while !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        argumentos.push(crate::parser::expresiones::parsear_expresion_principal(
            parser,
        )?);
        parser.coincidir(|t| matches!(t, LexToken::Coma));
    }

    Ok(Expresion::Instanciacion { tipo, argumentos })
}
