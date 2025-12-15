use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn es_inicio_instancia(parseador: &Parser, nombre: &str) -> bool {
    let primera_letra_mayuscula = nombre.chars().next().map_or(false, |c| c.is_uppercase());
    if !primera_letra_mayuscula {
        return false;
    }

    let hay_parentesis = parseador.posicion + 1 < parseador.tokens.len()
        && matches!(parseador.tokens[parseador.posicion + 1], LexToken::ParentesisIzq);
    
    hay_parentesis
}

fn obtener_nombre_clase(parseador: &mut Parser) -> Option<String> {
    match parseador.peekear().cloned() {
        Some(LexToken::Identificador(nombre)) => {
            if es_inicio_instancia(parseador, &nombre) {
                parseador.avanzar();
                return Some(nombre);
            }
            None
        }
        _ => None,
    }
}

fn parsear_argumentos_instancia(parseador: &mut Parser) -> Result<Vec<Expresion>, ParseError> {
    parseador.coincidir(|t| matches!(t, LexToken::ParentesisIzq));

    let mut argumentos = Vec::new();
    while !parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        argumentos.push(crate::parser::expresiones::parsear_expresion_principal(
            parseador,
        )?);
        parseador.coincidir(|t| matches!(t, LexToken::Coma));
    }

    Ok(argumentos)
}

pub fn intentar_parsear_instancia_inline(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let inicio_pos = parseador.posicion;

    let clase = match obtener_nombre_clase(parseador) {
        Some(nombre) => nombre,
        None => {
            parseador.posicion = inicio_pos;
            return Err(parseador.crear_error("No es una instancia inline"));
        }
    };

    let argumentos = parsear_argumentos_instancia(parseador)?;

    Ok(Expresion::Instanciacion { tipo: clase, argumentos })
}
