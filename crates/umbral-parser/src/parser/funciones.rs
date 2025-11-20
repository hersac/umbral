use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

fn parsear_parametro(p: &mut Parser) -> Result<Parametro, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let tipo = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        p.parsear_tipo()?
    } else {
        None
    };
    Ok(Parametro { nombre, tipo })
}

pub fn parsear_parametros(p: &mut Parser) -> Result<Vec<Parametro>, ParseError> {
    let mut lista = Vec::new();
    if matches!(p.peekear(), Some(LexToken::ParentesisDer)) {
        return Ok(lista);
    }
    loop {
        lista.push(parsear_parametro(p)?);
        if p.coincidir(|t| matches!(t, LexToken::Coma)) {
            continue;
        }
        break;
    }
    Ok(lista)
}

fn parsear_lista_parametros(p: &mut Parser) -> Result<Vec<Parametro>, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo(
            "Se esperaba '(' en definición de función",
            p.posicion,
        ));
    }
    let mut lista = Vec::new();
    if p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Ok(lista);
    }
    loop {
        lista.push(parsear_parametro(p)?);
        if p.coincidir(|t| matches!(t, LexToken::Coma)) {
            continue;
        }
        if p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            break;
        }
        return Err(ParseError::nuevo(
            "Se esperaba ',' o ')' en lista de parámetros",
            p.posicion,
        ));
    }
    Ok(lista)
}

pub fn parsear_declaracion_funcion(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let parametros = parsear_lista_parametros(p)?;
    let tipo_retorno = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        p.parsear_tipo()?
    } else {
        None
    };
    let cuerpo = p.parsear_bloque()?;
    Ok(Sentencia::Funcion(DeclaracionFuncion {
        nombre,
        parametros,
        tipo_retorno,
        cuerpo,
        exportado,
    }))
}

pub fn parsear_funcion_interna(p: &mut Parser, es_publico: bool) -> Result<Metodo, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let parametros = parsear_lista_parametros(p)?;
    let tipo_retorno = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        p.parsear_tipo()?
    } else {
        None
    };
    
    p.coincidir(|t| matches!(t, LexToken::Asignacion));
    
    let cuerpo = p.parsear_bloque()?;
    Ok(Metodo {
        nombre,
        parametros,
        tipo_retorno,
        cuerpo,
        publico: es_publico,
    })
}

impl Parser {
    pub fn parsear_bloque(&mut self) -> Result<Vec<Sentencia>, ParseError> {
        if !self.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
            return Err(ParseError::nuevo(
                "Se esperaba '{' para bloque",
                self.posicion,
            ));
        }
        let mut sentencias = Vec::new();
        while !self.esta_fin() {
            if self.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
                break;
            }
            sentencias.push(self.parsear_sentencia()?);
        }
        Ok(sentencias)
    }
}
