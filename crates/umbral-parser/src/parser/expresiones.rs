use crate::ast::*;
use crate::error::ParseError;
use crate::parser::{Parser, objetos};
use umbral_lexer::Token as LexToken;

pub fn parsear_expresion_principal(parser: &mut Parser) -> Result<Expresion, ParseError> {
    parsear_or(parser)
}

fn parsear_or(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_and(parser)?;
    while parser.coincidir(|t| matches!(t, LexToken::Or)) {
        let derecha = parsear_and(parser)?;
        izquierda = crear_binaria(izquierda, derecha, "||");
    }
    Ok(izquierda)
}

fn parsear_and(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_igualdad(parser)?;
    while parser.coincidir(|t| matches!(t, LexToken::And)) {
        let derecha = parsear_igualdad(parser)?;
        izquierda = crear_binaria(izquierda, derecha, "&&");
    }
    Ok(izquierda)
}

fn parsear_igualdad(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_comparacion(parser)?;
    loop {
        if parser.coincidir(|t| matches!(t, LexToken::IgualIgual)) {
            izquierda = crear_binaria(izquierda, parsear_comparacion(parser)?, "==");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Diferente)) {
            izquierda = crear_binaria(izquierda, parsear_comparacion(parser)?, "!=");
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
            izquierda = crear_binaria(izquierda, parsear_termino(parser)?, "<");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::MenorIgual)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parser)?, "<=");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Mayor)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parser)?, ">");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::MayorIgual)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parser)?, ">=");
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
            izquierda = crear_binaria(izquierda, parsear_factor(parser)?, "+");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Resta)) {
            izquierda = crear_binaria(izquierda, parsear_factor(parser)?, "-");
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
            izquierda = crear_binaria(izquierda, parsear_unaria(parser)?, "*");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Division)) {
            izquierda = crear_binaria(izquierda, parsear_unaria(parser)?, "/");
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Modulo)) {
            izquierda = crear_binaria(izquierda, parsear_unaria(parser)?, "%");
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_unaria(parser: &mut Parser) -> Result<Expresion, ParseError> {
    if parser.coincidir(|t| matches!(t, LexToken::Not)) {
        return Ok(Expresion::Unaria {
            operador: "!".to_string(),
            expresion: Box::new(parsear_unaria(parser)?),
        });
    }
    if parser.coincidir(|t| matches!(t, LexToken::Resta)) {
        return Ok(Expresion::Unaria {
            operador: "-".to_string(),
            expresion: Box::new(parsear_unaria(parser)?),
        });
    }
    if parser.coincidir(|t| matches!(t, LexToken::Spread)) {
        return Ok(Expresion::Spread(Box::new(parsear_unaria(parser)?)));
    }
    parsear_postfija(parser)
}

fn parsear_postfija(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let mut expr = parsear_primaria(parser)?;

    loop {
        // Verificar si es una llamada a función (identificador seguido de paréntesis)
        if matches!(expr, Expresion::Identificador(_)) 
            && parser.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) 
        {
            let nombre = match expr {
                Expresion::Identificador(n) => n,
                _ => unreachable!(),
            };
            
            let mut argumentos = Vec::new();
            if !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                loop {
                    argumentos.push(parsear_expresion_principal(parser)?);
                    if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
                        continue;
                    }
                    if parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                        break;
                    }
                    return Err(ParseError::nuevo("Se esperaba ',' o ')'", parser.posicion));
                }
            }
            expr = Expresion::LlamadoFuncion {
                nombre,
                argumentos,
            };
            continue;
        }
        
        if parser.coincidir(|t| matches!(t, LexToken::Punto)) {
            let propiedad = parser.parsear_identificador_consumir()?;

            // Verificar si es una llamada a método (tiene paréntesis)
            if parser.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
                let mut argumentos = Vec::new();
                if !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                    loop {
                        argumentos.push(parsear_expresion_principal(parser)?);
                        if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
                            continue;
                        }
                        if parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                            break;
                        }
                        return Err(ParseError::nuevo("Se esperaba ',' o ')'", parser.posicion));
                    }
                }
                expr = Expresion::LlamadoMetodo {
                    objeto: Box::new(expr),
                    metodo: propiedad,
                    argumentos,
                };
            } else {
                expr = Expresion::AccesoPropiedad {
                    objeto: Box::new(expr),
                    propiedad,
                };
            }
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::CorcheteIzq)) {
            let indice = parsear_expresion_principal(parser)?;
            if !parser.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
                return Err(ParseError::nuevo("Se esperaba ']'", parser.posicion));
            }
            expr = Expresion::AccesoIndice {
                objeto: Box::new(expr),
                indice: Box::new(indice),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Incremento)) {
            expr = Expresion::Unaria {
                operador: "++".to_string(),
                expresion: Box::new(expr),
            };
            continue;
        }
        if parser.coincidir(|t| matches!(t, LexToken::Decremento)) {
            expr = Expresion::Unaria {
                operador: "--".to_string(),
                expresion: Box::new(expr),
            };
            continue;
        }
        break;
    }

    Ok(expr)
}

fn parsear_primaria(parser: &mut Parser) -> Result<Expresion, ParseError> {
    let token = parser.peekear().cloned();
    match token {
        Some(LexToken::Numero(ref n)) => {
            parser.avanzar();
            let val = n.parse::<f64>().unwrap_or(0.0);
            if n.contains('.') {
                Ok(Expresion::LiteralFloat(val))
            } else {
                Ok(Expresion::LiteralEntero(val as i64))
            }
        }
        Some(LexToken::Cadena(ref s) | LexToken::CadenaMultilinea(ref s)) => {
            parser.avanzar();
            Ok(Expresion::LiteralCadena(s.clone()))
        }
        Some(LexToken::CadenaLiteral(ref s)) => {
            parser.avanzar();
            Ok(Expresion::LiteralCadenaLiteral(s.clone()))
        }
        Some(LexToken::Verdadero) => {
            parser.avanzar();
            Ok(Expresion::LiteralBool(true))
        }
        Some(LexToken::Falso) => {
            parser.avanzar();
            Ok(Expresion::LiteralBool(false))
        }
        Some(LexToken::Nulo) => {
            parser.avanzar();
            Ok(Expresion::LiteralNulo)
        }
        Some(LexToken::This) => {
            parser.avanzar();
            Ok(Expresion::This)
        }
        Some(LexToken::Instanciar) => {
            parser.avanzar();
            let tipo = parser.parsear_identificador_consumir()?;
            if !parser.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
                return Err(ParseError::nuevo(
                    "Se esperaba '(' después del tipo",
                    parser.posicion,
                ));
            }
            let mut argumentos = Vec::new();
            while !parser.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                argumentos.push(parsear_expresion_principal(parser)?);
                parser.coincidir(|t| matches!(t, LexToken::Coma));
            }
            Ok(Expresion::Instanciacion { tipo, argumentos })
        }
        Some(LexToken::Identificador(_)) => {
            if let Some(LexToken::Identificador(nombre)) = parser.peekear() {
                let nombre = nombre.clone();
                parser.avanzar();
                return Ok(Expresion::Identificador(nombre));
            }
            Err(ParseError::nuevo("Expresion no valida", parser.posicion))
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
        let exp = parsear_expresion_principal(parser)?;
        elementos.push(exp);

        if parser.coincidir(|t| matches!(t, LexToken::Coma)) {
            if matches!(parser.peekear(), Some(LexToken::LlaveDer)) {
                parser.avanzar();
                break;
            }
            continue;
        }

        if parser.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
            break;
        }

        return Err(ParseError::nuevo(
            "Se esperaba ',' o '}' despues de un elemento de la lista",
            parser.posicion,
        ));
    }

    Ok(Expresion::Array(elementos))
}

fn crear_binaria(izquierda: Expresion, derecha: Expresion, operador: &str) -> Expresion {
    Expresion::Binaria {
        izquierda: Box::new(izquierda),
        operador: operador.to_string(),
        derecha: Box::new(derecha),
    }
}
