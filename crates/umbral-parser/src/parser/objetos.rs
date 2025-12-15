use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn obtener_clave_objeto(parseador: &Parser) -> Option<String> {
    match parseador.peekear() {
        Some(LexToken::Identificador(nombre)) => Some(nombre.clone()),
        Some(LexToken::Cadena(texto) | LexToken::CadenaLiteral(texto)) => Some(texto.clone()),
        _ => None,
    }
}

fn parsear_par_clave_valor(parseador: &mut Parser) -> Result<(String, Expresion), ParseError> {
    let clave = match obtener_clave_objeto(parseador) {
        Some(c) => c,
        None => return Err(parseador.crear_error("Clave de objeto esperada")),
    };

    parseador.avanzar();

    if !parseador.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
        return Err(parseador.crear_error("Se esperaba '=>' en objeto"));
    }

    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    Ok((clave, valor))
}

fn parsear_pares_objeto(parseador: &mut Parser) -> Result<Vec<(String, Expresion)>, ParseError> {
    let mut pares = Vec::new();

    loop {
        let par = parsear_par_clave_valor(parseador)?;
        pares.push(par);

        if parseador.coincidir(|t| matches!(t, LexToken::Coma)) {
            continue;
        }

        if parseador.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
            break;
        }

        return Err(parseador.crear_error("Se esperaba ',' o ']' en objeto"));
    }

    Ok(pares)
}

pub fn parsear_objeto_principal(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    if parseador.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
        return Ok(Expresion::Objeto(Vec::new()));
    }

    let pares = parsear_pares_objeto(parseador)?;
    Ok(Expresion::Objeto(pares))
}
