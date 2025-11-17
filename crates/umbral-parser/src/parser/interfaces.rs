use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_interfaz(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }

    let mut metodos = Vec::new();
    
    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        p.coincidir(|t| matches!(t, LexToken::PropPublica));
        
        if !p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
            return Err(ParseError::nuevo("Se esperaba declaración de función en interfaz", p.posicion));
        }
        
        let nombre_metodo = p.parsear_identificador_consumir()?;
        
        if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
            return Err(ParseError::nuevo("Se esperaba '('", p.posicion));
        }
        
        let parametros = crate::parser::funciones::parsear_parametros(p)?;
        
        if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
        }
        
        let tipo_retorno = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
            p.parsear_tipo()?
        } else {
            None
        };
        
        if !p.coincidir(|t| matches!(t, LexToken::PuntoYComa)) {
            return Err(ParseError::nuevo("Se esperaba ';' después de la firma del método", p.posicion));
        }
        
        metodos.push(Metodo {
            nombre: nombre_metodo,
            parametros,
            tipo_retorno,
            cuerpo: Vec::new(),
            publico: true,
        });
    }
    
    Ok(Sentencia::Interfaz(DeclaracionInterfaz { nombre, metodos }))
}
