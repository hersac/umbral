use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::expresiones;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_constante(parseador: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = parseador.parsear_identificador_consumir()?;

    let tipo = obtener_tipo_explicito(parseador)?;

    if !parseador.coincidir(|t| matches!(t, LexToken::Asignacion)) {
        return Err(parseador.crear_error("Se esperaba '=' en declaracion constante"));
    }

    let valor = expresiones::parsear_expresion_principal(parseador)?;
    parseador.coincidir(|t| matches!(t, LexToken::PuntoYComa));

    let tipo = tipo.or_else(|| Some(inferir_tipo(&valor)));

    Ok(Sentencia::DeclaracionConstante(DeclaracionConstante {
        nombre,
        tipo,
        valor,
        exportado,
    }))
}

fn obtener_tipo_explicito(parseador: &mut Parser) -> Result<Option<Tipo>, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        return Ok(None);
    }
    
    let tipo = parseador.parsear_tipo()?.ok_or_else(|| {
        parseador.crear_error("Tipo esperado despues de '->' en constante")
    })?;
    
    Ok(Some(tipo))
}

fn crear_tipo(nombre: &str) -> Tipo {
    Tipo {
        nombre: nombre.to_string(),
    }
}

fn inferir_tipo(expresion: &Expresion) -> Tipo {
    match expresion {
        Expresion::LiteralEntero(_) => crear_tipo("Int"),
        Expresion::LiteralFloat(_) => crear_tipo("Flo"),
        Expresion::LiteralCadena(_) | Expresion::LiteralCadenaLiteral(_) => crear_tipo("Str"),
        Expresion::LiteralBool(_) => crear_tipo("Bool"),
        Expresion::Objeto(_) => crear_tipo("Obj"),
        Expresion::Array(_) => crear_tipo("Array"),
        Expresion::Binaria { izquierda, derecha, operador } => {
            inferir_tipo_binario(izquierda, derecha, operador)
        }
        Expresion::Unaria { operador, expresion } => inferir_tipo_unario(operador, expresion),
        Expresion::Agrupada(expr) => inferir_tipo(expr),
        _ => crear_tipo("Any"),
    }
}

fn inferir_tipo_binario(izquierda: &Expresion, derecha: &Expresion, operador: &str) -> Tipo {
    let es_aritmetico = matches!(operador, "+" | "-" | "*" | "/" | "%");
    let es_comparacion = matches!(operador, "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||");
    
    if es_comparacion {
        return crear_tipo("Bool");
    }
    
    if es_aritmetico {
        return inferir_tipo_aritmetico(izquierda, derecha);
    }
    
    crear_tipo("Any")
}

fn inferir_tipo_aritmetico(izquierda: &Expresion, derecha: &Expresion) -> Tipo {
    let tipo_izq = inferir_tipo(izquierda);
    let tipo_der = inferir_tipo(derecha);
    
    let es_flotante = tipo_izq.nombre == "Flo" || tipo_der.nombre == "Flo";
    
    if es_flotante {
        return crear_tipo("Flo");
    }
    
    crear_tipo("Int")
}

fn inferir_tipo_unario(operador: &str, expresion: &Expresion) -> Tipo {
    match operador {
        "-" => inferir_tipo(expresion),
        "!" => crear_tipo("Bool"),
        _ => crear_tipo("Any"),
    }
}
