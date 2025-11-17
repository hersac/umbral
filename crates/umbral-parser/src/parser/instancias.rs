use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn intentar_parsear_instancia_inline(parser: &mut Parser) -> Result<Expresion, ParseError> {
    if let Some(LexToken::Identificador(n)) = parser.peekear() {
        if parser.posicion + 1 < parser.tokens.len()
            && matches!(parser.tokens[parser.posicion + 1], LexToken::ParentesisIzq)
        {
            let tipo = n.clone();
            parser.avanzar();
            parser.coincidir(|t| matches!(t, LexToken::ParentesisIzq));
            let mut args = Vec::new();
            if !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                loop {
                    args.push(crate::parser::expresiones::parsear_expresion_principal(
                        parser,
                    )?);
                    if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
                        continue;
                    }
                    if parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                        break;
                    }
                    return Err(ParseError::nuevo(
                        "Se esperaba ',' o ')' en instancia",
                        parser.posicion,
                    ));
                }
            }
            return Ok(Expresion::Instanciacion {
                tipo,
                argumentos: args,
            });
        }
    }
    Err(ParseError::nuevo("No es instancia inline", parser.posicion))
}
