use crate::ast::*;
use crate::error::ParseError;
use crate::parser::{Parser, objetos};
use umbral_lexer::Token as LexToken;

pub fn parsear_expresion_principal(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parsear_or(parseador)
}

fn parsear_or(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_and(parseador)?;
    while parseador.coincidir(|t| matches!(t, LexToken::Or)) {
        let derecha = parsear_and(parseador)?;
        izquierda = crear_binaria(izquierda, derecha, "||");
    }
    Ok(izquierda)
}

fn parsear_and(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_igualdad(parseador)?;
    while parseador.coincidir(|t| matches!(t, LexToken::And)) {
        let derecha = parsear_igualdad(parseador)?;
        izquierda = crear_binaria(izquierda, derecha, "&&");
    }
    Ok(izquierda)
}

fn parsear_igualdad(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_comparacion(parseador)?;
    loop {
        if parseador.coincidir(|t| matches!(t, LexToken::IgualIgual)) {
            izquierda = crear_binaria(izquierda, parsear_comparacion(parseador)?, "==");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::Diferente)) {
            izquierda = crear_binaria(izquierda, parsear_comparacion(parseador)?, "!=");
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_comparacion(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_termino(parseador)?;
    loop {
        if parseador.coincidir(|t| matches!(t, LexToken::Menor)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parseador)?, "<");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::MenorIgual)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parseador)?, "<=");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::Mayor)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parseador)?, ">");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::MayorIgual)) {
            izquierda = crear_binaria(izquierda, parsear_termino(parseador)?, ">=");
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_termino(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_factor(parseador)?;
    loop {
        if parseador.coincidir(|t| matches!(t, LexToken::Suma)) {
            izquierda = crear_binaria(izquierda, parsear_factor(parseador)?, "+");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::Resta)) {
            izquierda = crear_binaria(izquierda, parsear_factor(parseador)?, "-");
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_factor(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut izquierda = parsear_unaria(parseador)?;
    loop {
        if parseador.coincidir(|t| matches!(t, LexToken::Multiplicacion)) {
            izquierda = crear_binaria(izquierda, parsear_unaria(parseador)?, "*");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::Division)) {
            izquierda = crear_binaria(izquierda, parsear_unaria(parseador)?, "/");
            continue;
        }
        if parseador.coincidir(|t| matches!(t, LexToken::Modulo)) {
            izquierda = crear_binaria(izquierda, parsear_unaria(parseador)?, "%");
            continue;
        }
        break;
    }
    Ok(izquierda)
}

fn parsear_unaria(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    if parseador.coincidir(|t| matches!(t, LexToken::Not)) {
        return Ok(Expresion::Unaria {
            operador: "!".to_string(),
            expresion: Box::new(parsear_unaria(parseador)?),
        });
    }
    if parseador.coincidir(|t| matches!(t, LexToken::Resta)) {
        return Ok(Expresion::Unaria {
            operador: "-".to_string(),
            expresion: Box::new(parsear_unaria(parseador)?),
        });
    }
    if parseador.coincidir(|t| matches!(t, LexToken::Spread)) {
        return Ok(Expresion::Spread(Box::new(parsear_unaria(parseador)?)));
    }
    parsear_postfija(parseador)
}

fn parsear_postfija(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut expresion = parsear_primaria(parseador)?;

    loop {
        let procesado = procesar_llamado_funcion(parseador, &mut expresion)?
            || procesar_acceso_punto(parseador, &mut expresion)?
            || procesar_acceso_indice(parseador, &mut expresion)?
            || procesar_incremento(parseador, &mut expresion)?
            || procesar_decremento(parseador, &mut expresion)?;
        
        if !procesado {
            break;
        }
    }

    Ok(expresion)
}

fn procesar_llamado_funcion(parseador: &mut Parser, expresion: &mut Expresion) -> Result<bool, ParseError> {
    if !matches!(expresion, Expresion::Identificador(_)) {
        return Ok(false);
    }
    
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Ok(false);
    }
    
    let nombre = match expresion.clone() {
        Expresion::Identificador(n) => n,
        _ => unreachable!(),
    };
    
    let argumentos = parsear_lista_argumentos(parseador)?;
    *expresion = Expresion::LlamadoFuncion { nombre, argumentos };
    
    Ok(true)
}

fn parsear_lista_argumentos(parseador: &mut Parser) -> Result<Vec<Expresion>, ParseError> {
    let mut argumentos = Vec::new();
    
    if parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Ok(argumentos);
    }
    
    loop {
        argumentos.push(parsear_expresion_principal(parseador)?);
        
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

fn procesar_acceso_punto(parseador: &mut Parser, expresion: &mut Expresion) -> Result<bool, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::Punto)) {
        return Ok(false);
    }
    
    let propiedad = parseador.parsear_identificador_consumir()?;

    if parseador.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        let argumentos = parsear_lista_argumentos(parseador)?;
        *expresion = Expresion::LlamadoMetodo {
            objeto: Box::new(expresion.clone()),
            metodo: propiedad,
            argumentos,
        };
        return Ok(true);
    }
    
    *expresion = Expresion::AccesoPropiedad {
        objeto: Box::new(expresion.clone()),
        propiedad,
    };
    
    Ok(true)
}

fn procesar_acceso_indice(parseador: &mut Parser, expresion: &mut Expresion) -> Result<bool, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::CorcheteIzq)) {
        return Ok(false);
    }
    
    let indice = parsear_expresion_principal(parseador)?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
        return Err(parseador.crear_error("Se esperaba ']'"));
    }
    
    *expresion = Expresion::AccesoIndice {
        objeto: Box::new(expresion.clone()),
        indice: Box::new(indice),
    };
    
    Ok(true)
}

fn procesar_incremento(parseador: &mut Parser, expresion: &mut Expresion) -> Result<bool, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::Incremento)) {
        return Ok(false);
    }
    
    *expresion = Expresion::Unaria {
        operador: "++".to_string(),
        expresion: Box::new(expresion.clone()),
    };
    
    Ok(true)
}

fn procesar_decremento(parseador: &mut Parser, expresion: &mut Expresion) -> Result<bool, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::Decremento)) {
        return Ok(false);
    }
    
    *expresion = Expresion::Unaria {
        operador: "--".to_string(),
        expresion: Box::new(expresion.clone()),
    };
    
    Ok(true)
}

fn parsear_primaria(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let token = parseador.peekear().cloned();
    
    match token {
        Some(LexToken::Numero(ref n)) => parsear_numero(parseador, n),
        Some(LexToken::Cadena(ref s) | LexToken::CadenaMultilinea(ref s)) => parsear_cadena(parseador, s),
        Some(LexToken::CadenaLiteral(ref s)) => parsear_cadena_literal(parseador, s),
        Some(LexToken::Verdadero) => parsear_booleano(parseador, true),
        Some(LexToken::Falso) => parsear_booleano(parseador, false),
        Some(LexToken::Nulo) => parsear_nulo(parseador),
        Some(LexToken::This) => parsear_this(parseador),
        Some(LexToken::Instanciar) => parsear_instanciacion(parseador),
        Some(LexToken::Identificador(_)) => parsear_identificador(parseador),
        Some(LexToken::ParentesisIzq) => parsear_agrupada(parseador),
        Some(LexToken::CorcheteIzq) => parsear_objeto(parseador),
        Some(LexToken::LlaveIzq) => parsear_array(parseador),
        _ => Err(parseador.crear_error("Expresion no valida")),
    }
}

fn parsear_numero(parseador: &mut Parser, numero: &str) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    let valor = numero.parse::<f64>().unwrap_or(0.0);
    
    if numero.contains('.') {
        return Ok(Expresion::LiteralFloat(valor));
    }
    
    Ok(Expresion::LiteralEntero(valor as i64))
}

fn parsear_cadena(parseador: &mut Parser, texto: &str) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    Ok(Expresion::LiteralCadena(texto.to_string()))
}

fn parsear_cadena_literal(parseador: &mut Parser, texto: &str) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    Ok(Expresion::LiteralCadenaLiteral(texto.to_string()))
}

fn parsear_booleano(parseador: &mut Parser, valor: bool) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    Ok(Expresion::LiteralBool(valor))
}

fn parsear_nulo(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    Ok(Expresion::LiteralNulo)
}

fn parsear_this(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    Ok(Expresion::This)
}

fn parsear_instanciacion(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    let tipo = parseador.parsear_identificador_consumir()?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(parseador.crear_error("Se esperaba '(' después del tipo"));
    }
    
    let argumentos = parsear_argumentos_instancia(parseador)?;
    
    Ok(Expresion::Instanciacion { tipo, argumentos })
}

fn parsear_argumentos_instancia(parseador: &mut Parser) -> Result<Vec<Expresion>, ParseError> {
    let mut argumentos = Vec::new();
    
    while !parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        argumentos.push(parsear_expresion_principal(parseador)?);
        parseador.coincidir(|t| matches!(t, LexToken::Coma));
    }
    
    Ok(argumentos)
}

fn parsear_identificador(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    if let Some(LexToken::Identificador(nombre)) = parseador.peekear() {
        let nombre = nombre.clone();
        parseador.avanzar();
        return Ok(Expresion::Identificador(nombre));
    }
    
    Err(parseador.crear_error("Expresion no valida"))
}

fn parsear_agrupada(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    let expresion = parsear_expresion_principal(parseador)?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(parseador.crear_error("Se esperaba ')'"));
    }
    
    Ok(Expresion::Agrupada(Box::new(expresion)))
}

fn parsear_objeto(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    objetos::parsear_objeto_principal(parseador)
}

fn parsear_array(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    parseador.avanzar();
    parsear_array_principal(parseador)
}

fn parsear_array_principal(parseador: &mut Parser) -> Result<Expresion, ParseError> {
    let mut elementos = Vec::new();

    if parseador.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        return Ok(Expresion::Array(elementos));
    }

    loop {
        let expresion = parsear_expresion_principal(parseador)?;
        elementos.push(expresion);

        if parseador.coincidir(|t| matches!(t, LexToken::Coma)) {
            if matches!(parseador.peekear(), Some(LexToken::LlaveDer)) {
                parseador.avanzar();
                break;
            }
            continue;
        }

        if parseador.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
            break;
        }

        return Err(parseador.crear_error("Se esperaba ',' o '}' despues de un elemento de la lista"));
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
