use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn validar_parentesis_apertura(parseador: &mut Parser, contexto: &str) -> Result<(), ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(parseador.crear_error(&format!("Se esperaba '(' despues de {}", contexto)));
    }
    Ok(())
}

fn validar_parentesis_cierre(parseador: &mut Parser) -> Result<(), ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(parseador.crear_error("Se esperaba ')'"));
    }
    Ok(())
}

pub fn parsear_tprint(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    validar_parentesis_apertura(parseador, "tprint")?;
    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    validar_parentesis_cierre(parseador)?;
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::LlamadoTPrint(LlamadoTPrint { valor }))
}

fn parsear_lista_argumentos(parseador: &mut Parser) -> Result<Vec<Expresion>, ParseError> {
    let mut argumentos = Vec::new();

    if parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Ok(argumentos);
    }

    loop {
        argumentos.push(crate::parser::expresiones::parsear_expresion_principal(
            parseador,
        )?);

        if parseador.coincidir(|t| matches!(t, LexToken::Coma)) {
            continue;
        }

        if parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            break;
        }

        return Err(parseador.crear_error("Se esperaba ',' o ')'"));
    }

    Ok(argumentos)
}

pub fn parsear_llamado_funcion(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = parseador.parsear_identificador_consumir()?;
    validar_parentesis_apertura(parseador, "llamada")?;
    let argumentos = parsear_lista_argumentos(parseador)?;
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::LlamadoFuncion(LlamadoFuncion {
        nombre,
        argumentos,
    }))
}

fn determinar_objetivo_asignacion(parseador: &mut Parser, nombre: String) -> ObjetivoAsignacion {
    if parseador.coincidir(|t| matches!(t, LexToken::Punto)) {
        let propiedad = parseador.parsear_identificador_consumir().unwrap();
        return ObjetivoAsignacion::Propiedad {
            objeto: Box::new(Expresion::Identificador(nombre)),
            propiedad,
        };
    }

    ObjetivoAsignacion::Variable(nombre)
}

pub fn parsear_asignacion(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = parseador.parsear_identificador_consumir()?;
    let objetivo = determinar_objetivo_asignacion(parseador, nombre);

    if !parseador.coincidir(|t| matches!(t, LexToken::Asignacion)) {
        return Err(parseador.crear_error("Se esperaba '=' en asignacion"));
    }

    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Asignacion(Asignacion { objetivo, valor }))
}

pub fn parsear_return(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    validar_parentesis_apertura(parseador, "return")?;
    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    validar_parentesis_cierre(parseador)?;
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Return(valor))
}

pub fn parsear_throw(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    // tw doesn't strictly need semicolon based on user example `tw: Error(...)`,
    // but usually statements end with semicolon.
    // Example: `tw: Error("...")` inside a block.
    // Standard Umbral statements use semicolon. I will enforce it for consistency unless user example implies otherwise.
    // User example:
    // tw: Error("instalacion fallida")
    // }
    // It is inside `i: ... {}`.
    // Usually blocks in Umbral don't force semicolons on the last statement?
    // `parsear_bloque` calls `parsear_sentencia` which calls specific parsers.
    // `parsear_return` checks for semicolon. `parsear_asignacion` checks for semicolon.
    // I should check for semicolon.
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Sentencia::Throw(Throw { valor }))
}
