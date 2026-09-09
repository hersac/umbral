use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_interfaz(
    p: &mut Parser,
    exportado: bool,
) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let parametros_tipo = p.parsear_parametros_tipo()?;

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
    }

    let mut metodos = Vec::new();
    let mut propiedades = Vec::new();

    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        if p.esta_fin() {
            return Err(p.crear_error("Interfaz sin cerrar, se esperaba '}'"));
        }
        parsear_miembro(p, &mut metodos, &mut propiedades)?;
    }

    Ok(Sentencia::Interfaz(DeclaracionInterfaz {
        nombre,
        parametros_tipo,
        metodos,
        propiedades,
        exportado,
    }))
}

fn parsear_miembro(
    p: &mut Parser,
    metodos: &mut Vec<Metodo>,
    propiedades: &mut Vec<Propiedad>,
) -> Result<(), ParseError> {
    let publico = !p.coincidir(|t| matches!(t, LexToken::PropPrivada));
    p.coincidir(|t| matches!(t, LexToken::PropPublica));
    if p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
        metodos.push(parsear_metodo(p)?);
        return Ok(());
    }
    propiedades.push(parsear_propiedad(p, publico)?);
    Ok(())
}

fn parsear_metodo(p: &mut Parser) -> Result<Metodo, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let parametros_tipo = p.parsear_parametros_tipo()?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(p.crear_error("Se esperaba '('"));
    }
    let parametros = crate::parser::funciones::parsear_parametros(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(p.crear_error("Se esperaba ')'"));
    }
    let tipo_retorno = parsear_retorno(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::PuntoYComa)) {
        return Err(p.crear_error("Se esperaba ';' después de la firma del método"));
    }
    Ok(Metodo {
        nombre,
        parametros_tipo,
        parametros,
        tipo_retorno,
        cuerpo: Vec::new(),
        publico: true,
        es_async: false,
        doc: None,
    })
}

fn parsear_retorno(p: &mut Parser) -> Result<Option<Tipo>, ParseError> {
    if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        return p.parsear_tipo();
    }
    Ok(None)
}

fn parsear_propiedad(p: &mut Parser, publico: bool) -> Result<Propiedad, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let tipo = parsear_retorno(p)?;
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
    Ok(Propiedad {
        nombre,
        tipo,
        publico,
        valor_inicial: None,
    })
}
