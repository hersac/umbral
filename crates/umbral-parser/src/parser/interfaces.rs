use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_interfaz(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
    }

    let mut metodos = Vec::new();
    
    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        p.coincidir(|t| matches!(t, LexToken::PropPublica));
        
        if !p.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
            return Err(p.crear_error("Se esperaba declaración de función en interfaz"));
        }
        
        let nombre_metodo = p.parsear_identificador_consumir()?;
        
        if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
            return Err(p.crear_error("Se esperaba '('"));
        }
        
        let parametros = crate::parser::funciones::parsear_parametros(p)?;
        
        if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            return Err(p.crear_error("Se esperaba ')'"));
        }
        
        let tipo_retorno = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
            p.parsear_tipo()?
        } else {
            None
        };
        
        if !p.coincidir(|t| matches!(t, LexToken::PuntoYComa)) {
            return Err(p.crear_error("Se esperaba ';' después de la firma del método"));
        }
        
        metodos.push(Metodo {
            nombre: nombre_metodo,
            parametros,
            tipo_retorno,
            cuerpo: Vec::new(),
            publico: true,
        });
    }
    
    Ok(Sentencia::Interfaz(DeclaracionInterfaz { nombre, metodos, exportado }))
}
