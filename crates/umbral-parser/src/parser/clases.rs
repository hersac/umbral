use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn parsear_extensiones(p: &mut Parser) -> Result<Vec<String>, ParseError> {
    let mut extensiones = Vec::new();
    if p.coincidir(|t| matches!(t, LexToken::Extension)) {
        loop {
            extensiones.push(p.parsear_identificador_consumir()?);
            if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
                break;
            }
        }
    }
    Ok(extensiones)
}

fn parsear_implementaciones(p: &mut Parser) -> Result<Vec<String>, ParseError> {
    let mut implementaciones = Vec::new();
    if p.coincidir(|t| matches!(t, LexToken::Implementacion)) {
        loop {
            implementaciones.push(p.parsear_identificador_consumir()?);
            if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
                break;
            }
        }
    }
    Ok(implementaciones)
}

fn determinar_visibilidad(p: &mut Parser) -> Result<bool, ParseError> {
    if p.coincidir(|t| matches!(t, LexToken::PropPublica)) {
        return Ok(true);
    }
    if p.coincidir(|t| matches!(t, LexToken::PropPrivada)) {
        return Ok(false);
    }
    Err(p.crear_error("Se esperaba 'pr' o 'pu' en el cuerpo de la clase"))
}

fn procesar_miembro_clase(
    p: &mut Parser,
) -> Result<(Option<Propiedad>, Option<Metodo>), ParseError> {
    let publico = determinar_visibilidad(p)?;

    let es_async = p.coincidir(|t| matches!(t, LexToken::Asy));

    if p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
        let metodo = crate::parser::funciones::parsear_funcion_interna(p, publico, es_async)?;
        return Ok((None, Some(metodo)));
    }

    if es_async {
        return Err(p.crear_error("Se esperaba 'f' despues de 'asy'"));
    }

    let nombre = p.parsear_identificador_consumir()?;
    let mut tipo = None;
    if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        tipo = p.parsear_tipo()?;
    }

    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));

    Ok((
        Some(Propiedad {
            nombre,
            tipo,
            publico,
            valor_inicial: None,
        }),
        None,
    ))
}

pub fn parsear_declaracion_clase(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let extensiones = parsear_extensiones(p)?;
    let implementaciones = parsear_implementaciones(p)?;

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
    }

    let mut propiedades = Vec::new();
    let mut metodos = Vec::new();

    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        let (prop, met) = procesar_miembro_clase(p)?;
        if let Some(m) = met {
            metodos.push(m);
        }
        if let Some(pr) = prop {
            propiedades.push(pr);
        }
    }

    Ok(Sentencia::Clase(DeclaracionClase {
        nombre,
        extensiones,
        implementaciones,
        propiedades,
        metodos,
        exportado,
    }))
}
