use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn parsear_parametro(p: &mut Parser) -> Result<Parametro, ParseError> {
    let es_rest = p.coincidir(|t| matches!(t, LexToken::PuntosSuspensivos));
    let nombre = p.parsear_identificador_consumir()?;
    let mut tipo = None;
    if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        tipo = p.parsear_tipo()?;
    }
    Ok(Parametro {
        nombre,
        tipo,
        es_rest,
    })
}

pub fn parsear_parametros(p: &mut Parser) -> Result<Vec<Parametro>, ParseError> {
    let mut lista = Vec::new();
    if matches!(p.peekear(), Some(LexToken::ParentesisDer)) {
        return Ok(lista);
    }
    loop {
        lista.push(parsear_parametro(p)?);
        if lista.last().is_some_and(|param: &Parametro| param.es_rest)
            && !matches!(p.peekear(), Some(LexToken::ParentesisDer))
        {
            return Err(p.crear_error("El parámetro rest ('...') debe ser el último"));
        }
        if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
            break;
        }
        if lista.last().is_some_and(|param: &Parametro| param.es_rest) {
            return Err(p.crear_error("El parámetro rest ('...') debe ser el último"));
        }
    }
    Ok(lista)
}

fn parsear_lista_parametros(p: &mut Parser) -> Result<Vec<Parametro>, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(p.crear_error("Se esperaba '(' en definición de función"));
    }
    let lista = parsear_parametros(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(p.crear_error("Se esperaba ')' tras parámetros"));
    }
    Ok(lista)
}

fn validar_inicio_funcion(p: &mut Parser) -> Result<bool, ParseError> {
    let es_async = p.coincidir(|t| matches!(t, LexToken::Asy));
    if !es_async && !p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
        return Err(p.crear_error("Se esperaba 'f'"));
    }
    if es_async && !p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
        return Err(p.crear_error("Se esperaba 'f' despues de 'asy'"));
    }
    Ok(es_async)
}

pub fn parsear_declaracion_funcion(
    p: &mut Parser,
    exportado: bool,
    doc: Option<UmDoc>,
) -> Result<Sentencia, ParseError> {
    let es_async = validar_inicio_funcion(p)?;
    let nombre = p.parsear_identificador_consumir()?;
    let parametros_tipo = p.parsear_parametros_tipo()?;
    let parametros = parsear_lista_parametros(p)?;

    let mut tipo_retorno = None;
    if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        tipo_retorno = p.parsear_tipo()?;
    }

    Ok(Sentencia::Funcion(DeclaracionFuncion {
        nombre,
        parametros_tipo,
        parametros,
        tipo_retorno,
        cuerpo: p.parsear_bloque()?,
        exportado,
        es_async,
        doc,
    }))
}

pub fn parsear_funcion_interna(
    p: &mut Parser,
    publico: bool,
    es_async_externo: bool,
    doc: Option<UmDoc>,
) -> Result<Metodo, ParseError> {
    let es_async = es_async_externo || p.coincidir(|t| matches!(t, LexToken::Asy));

    if es_async && !matches!(p.peekear(), Some(LexToken::Identificador(_))) {
        return Err(p.crear_error("Se esperaba nombre de método"));
    }

    let nombre = p.parsear_identificador_consumir()?;
    let parametros_tipo = p.parsear_parametros_tipo()?;
    let parametros = parsear_lista_parametros(p)?;

    let mut tipo_retorno = None;
    if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        tipo_retorno = p.parsear_tipo()?;
    }

    p.coincidir(|t| matches!(t, LexToken::Asignacion));

    Ok(Metodo {
        nombre,
        parametros_tipo,
        parametros,
        tipo_retorno,
        cuerpo: p.parsear_bloque()?,
        publico,
        es_async,
        doc,
    })
}

impl Parser {
    pub fn parsear_bloque(&mut self) -> Result<Vec<Sentencia>, ParseError> {
        if !self.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
            return Err(self.crear_error("Se esperaba '{'"));
        }
        let mut sentencias = Vec::new();
        while !self.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
            if self.esta_fin() {
                return Err(self.crear_error("Bloque sin cerrar, se esperaba '}'"));
            }
            sentencias.push(self.parsear_sentencia()?);
        }
        Ok(sentencias)
    }
}
