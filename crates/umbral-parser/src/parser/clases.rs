use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn parsear_extensiones(p: &mut Parser) -> Result<Vec<String>, ParseError> {
    let mut extensiones = Vec::new();
    if !p.coincidir(|t| matches!(t, LexToken::Extension)) {
        return Ok(extensiones);
    }
    loop {
        extensiones.push(p.parsear_identificador_consumir()?);
        if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
            break;
        }
    }
    Ok(extensiones)
}

fn parsear_implementaciones(p: &mut Parser) -> Result<Vec<String>, ParseError> {
    let mut implementaciones = Vec::new();
    if !p.coincidir(|t| matches!(t, LexToken::Implementacion)) {
        return Ok(implementaciones);
    }
    loop {
        implementaciones.push(p.parsear_identificador_consumir()?);
        if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
            break;
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

fn resolver_doc_clase(a: Option<UmDoc>, b: Option<UmDoc>) -> Option<UmDoc> {
    if b.is_some() {
        return b;
    }
    a
}

fn procesar_miembro_clase(p: &mut Parser) -> Result<(Option<Propiedad>, Option<Metodo>), ParseError> {
    let doc_a = crate::parser::umdocs::consumir_doc(p);
    p.coincidir(|t| matches!(t, LexToken::Out));
    let doc = resolver_doc_clase(doc_a, crate::parser::umdocs::consumir_doc(p));
    let publico = determinar_visibilidad(p)?;
    let es_async = p.coincidir(|t| matches!(t, LexToken::Asy));
    if p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
        let metodo = crate::parser::funciones::parsear_funcion_interna(p, publico, es_async, doc)?;
        return Ok((None, Some(metodo)));
    }
    if es_async {
        return Err(p.crear_error("Se esperaba 'f' despues de 'asy'"));
    }
    let nombre = p.parsear_identificador_consumir()?;
    let tipo = recolectar_tipo(p)?;
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

fn recolectar_tipo(p: &mut Parser) -> Result<Option<Tipo>, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        return Ok(None);
    }
    p.parsear_tipo()
}

pub fn parsear_declaracion_clase(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    parsear_declaracion_clase_con_doc(p, exportado, None)
}

pub fn parsear_declaracion_clase_con_doc(
    p: &mut Parser,
    exportado: bool,
    doc: Option<UmDoc>,
) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let extensiones = parsear_extensiones(p)?;
    let implementaciones = parsear_implementaciones(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
    }
    let (propiedades, metodos) = recolectar_miembros(p)?;
    Ok(Sentencia::Clase(DeclaracionClase {
        nombre,
        extensiones,
        implementaciones,
        propiedades,
        metodos,
        exportado,
        doc,
    }))
}

fn recolectar_miembros(p: &mut Parser) -> Result<(Vec<Propiedad>, Vec<Metodo>), ParseError> {
    let mut propiedades = Vec::new();
    let mut metodos = Vec::new();
    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        if es_doc_huerfano(p) {
            let _ = crate::parser::umdocs::consumir_doc(p);
            continue;
        }
        if p.esta_fin() {
            return Err(p.crear_error("Clase sin cerrar, se esperaba '}'"));
        }
        let (prop, met) = procesar_miembro_clase(p)?;
        if let Some(m) = met {
            metodos.push(m);
        }
        if let Some(pr) = prop {
            propiedades.push(pr);
        }
    }
    Ok((propiedades, metodos))
}

fn es_doc_huerfano(p: &Parser) -> bool {
    if !matches!(p.peekear(), Some(LexToken::UmDoc(_))) {
        return false;
    }
    matches!(p.tokens.get(p.posicion + 1), Some(LexToken::LlaveDer))
}
