use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_clase(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    
    let mut extensiones = Vec::new();
    if p.coincidir(|t| matches!(t, LexToken::Extension)) {
        loop {
            extensiones.push(p.parsear_identificador_consumir()?);
            if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
                break;
            }
        }
    }
    
    let mut implementaciones = Vec::new();
    if p.coincidir(|t| matches!(t, LexToken::Implementacion)) {
        loop {
            implementaciones.push(p.parsear_identificador_consumir()?);
            if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
                break;
            }
        }
    }
    
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }

    let mut propiedades = Vec::new();
    let mut metodos = Vec::new();

    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        let es_publico = if p.coincidir(|t| matches!(t, LexToken::PropPublica)) {
            true
        } else if p.coincidir(|t| matches!(t, LexToken::PropPrivada)) {
            false
        } else {
            return Err(ParseError::nuevo("Se esperaba 'pr' o 'pu' en el cuerpo de la clase", p.posicion));
        };

        if p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
            let metodo = crate::parser::funciones::parsear_funcion_interna(p, es_publico)?;
            metodos.push(metodo);
        } else {
            let nombre_prop = p.parsear_identificador_consumir()?;
            
            let tipo = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
                p.parsear_tipo()?
            } else {
                None
            };

            p.coincidir(|t| matches!(t, LexToken::PuntoYComa));
            
            propiedades.push(Propiedad {
                nombre: nombre_prop,
                tipo,
                publico: es_publico,
                valor_inicial: None,
            });
        }
    }

    Ok(Sentencia::Clase(DeclaracionClase {
        nombre,
        extensiones,
        implementaciones,
        propiedades,
        metodos,
    }))
}
