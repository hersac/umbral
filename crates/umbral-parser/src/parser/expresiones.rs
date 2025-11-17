use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::instancias;
use crate::parser::objetos;
use umbral_lexer::Token as LexToken;

pub fn parsear_expresion_principal(parser: &mut Parser) -> Result<Expresion, ParseError> {
    parsear_or(parser)
}

fn parsear_or(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_and(parser)?;
    while parser.coincidir(|t| matches!(t, LexToken::Or)) {
        let derecha = parsear_and(parser)?;
        izquierda = Expresion::Binaria {
            izquierda: Box::new(izquierda),
            operador: "||".to_string(),
            derecha: Box::new(derecha),
        };
    }
    Ok(izquierda)
}

fn parsear_and(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_igualdad(parser)?;
    while parser.coincidir(|t| matches!(t, LexToken::And)) {
        let derecha = parsear_igualdad(parser)?;
        izquierda = Expresion::Binaria {
            izquierda: Box::new(izquierda),
            operador: "&&".to_string(),
            derecha: Box::new(derecha),
        };
    }
    Ok(izquierda)
}

fn parsear_igualdad(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_comparacion(parser)?;
    loop {
        if parser.coincidir(|t| matches!(t, LexToken::IgualIgual)) {
            let derecha = parsear_comparacion(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "==".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Diferente)) {
            let derecha = parsear_comparacion(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "!=".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_comparacion(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_termino(parser)?;
    loop {
        if parser.coincidir(|t| matches!(t, LexToken::Menor)) {
            let derecha = parsear_termino(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "<".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::MenorIgual)) {
            let derecha = parsear_termino(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "<=".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Mayor)) {
            let derecha = parsear_termino(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: ">".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::MayorIgual)) {
            let derecha = parsear_termino(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: ">=".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_termino(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_factor(parser)?;
    loop {
        if parser.coincidir(|t| matches!(t, LexToken::Suma)) {
            let derecha = parsear_factor(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "+".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Resta)) {
            let derecha = parsear_factor(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "-".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_factor(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_unaria(parser)?;
    loop {
        if parser.coincidir(|t| matches!(t, LexToken::Multiplicacion)) {
            let derecha = parsear_unaria(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "*".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Division)) {
            let derecha = parsear_unaria(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "/".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Modulo)) {
            let derecha = parsear_unaria(parser)?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "%".to_string(),
                derecha: Box::new(derecha),
            };
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_unaria(parser: &mut Parser) -> Result<Expresion, ParseError> {
    if parser.coincidir(|t| matches!(t, LexToken::Not)) {
        let expr = parsear_unaria(parser)?;
        return Ok(Expresion::Unaria {
            operador: "!".to_string(),
            expresion: Box::new(expr),
        });
    }
    if parser.coincidir(|t| matches!(t, LexToken::Resta)) {
        let expr = parsear_unaria(parser)?;
        return Ok(Expresion::Unaria {
            operador: "-".to_string(),
            expresion: Box::new(expr),
        });
    }
    parsear_primaria(parser)
}

fn parsear_primaria(parser: &mut Parser) -> Result<Expresion, ParseError> {
    match parser.peekear() {
        Some(LexToken::Numero(n)) => {
            let s = n.clone();
            parser.avanzar();
            if s.contains('.') {
                Ok(Expresion::LiteralFloat(s.parse::<f64>().unwrap_or(0.0)))
            } else {
                Ok(Expresion::LiteralEntero(s.parse::<i64>().unwrap_or(0)))
            }
        }
        Some(LexToken::Cadena(s)) => {
            let s = s.clone();
            parser.avanzar();
            Ok(Expresion::LiteralCadena(s))
        }
        Some(LexToken::CadenaMultilinea(s)) => {
            let s = s.clone();
            parser.avanzar();
            Ok(Expresion::LiteralCadena(s))
        }
        Some(LexToken::Verdadero) => {
            parser.avanzar();
            Ok(Expresion::LiteralBool(true))
        }
        Some(LexToken::Falso) => {
            parser.avanzar();
            Ok(Expresion::LiteralBool(false))
        }
        Some(LexToken::Identificador(n)) => {
            let nombre = n.clone();
            if parser.posicion + 1 < parser.tokens.len()
                && matches!(parser.tokens[parser.posicion + 1], LexToken::ParentesisIzq)
            {
                parser.avanzar();
                parser.coincidir(|t| matches!(t, LexToken::ParentesisIzq));
                let mut args = Vec::new();
                if !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                    loop {
                        args.push(parsear_expresion_principal(parser)?);
                        if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
                            continue;
                        }
                        if parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                            break;
                        }
                        return Err(ParseError::nuevo("Se esperaba ',' o ')'", parser.posicion));
                    }
                }
                Ok(Expresion::Instanciacion {
                    tipo: nombre,
                    argumentos: args,
                })
            } else {
                parser.avanzar();
                Ok(Expresion::Identificador(nombre))
            }
        }
        Some(LexToken::ParentesisIzq) => {
            parser.avanzar();
            let expr = parsear_expresion_principal(parser)?;
            if !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                return Err(ParseError::nuevo("Se esperaba ')'", parser.posicion));
            }
            Ok(Expresion::Agrupada(Box::new(expr)))
        }
        Some(LexToken::CorcheteIzq) => {
            parser.avanzar();
            objetos::parsear_objeto_principal(parser)
        }
        Some(LexToken::LlaveIzq) => {
            parser.avanzar();
            parsear_array_principal(parser)
        }
        _ => Err(ParseError::nuevo("Expresion no valida", parser.posicion)),
    }
}

fn parsear_array_principal(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut elementos = Vec::new();
    if parser.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        return Ok(Expresion::Array(elementos));
    }
    loop {
        if let Ok(exp) = instancias::intentar_parsear_instancia_inline(parser) {
            elementos.push(exp);
        } else {
            elementos.push(parsear_expresion_principal(parser)?);
        }
        if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
            break;
        }
        return Err(ParseError::nuevo("Se esperaba ',' o '}'", parser.posicion));
    }
    Ok(Expresion::Array(elementos))
}
