pub mod clases;
pub mod constantes;
pub mod controles;
pub mod enums;
pub mod expresiones;
pub mod funciones;
pub mod instancias;
pub mod interfaces;
pub mod objetos;
pub mod sentencias;
pub mod tokens;
pub mod utilidades;
pub mod variables;

use crate::ast::*;
use crate::error::ParseError;
use expresiones::parsear_expresion_principal;
use umbral_lexer::Token as LexToken;

pub struct Parser {
    pub tokens: Vec<LexToken>,
    pub posicion: usize,
}

fn expr_a_string(expr: &Expresion) -> String {
    match expr {
        Expresion::Identificador(n) => n.clone(),
        Expresion::AccesoPropiedad { objeto, propiedad } => {
            format!("{}.{}", expr_a_string(objeto), propiedad)
        }
        Expresion::AccesoIndice { objeto, indice } => {
            format!("{}[{}]", expr_a_string(objeto), expr_a_string(indice))
        }
        Expresion::LiteralEntero(n) => n.to_string(),
        Expresion::LiteralCadena(s) => s.clone(),
        _ => "expr".to_string(),
    }
}

impl Parser {
    pub fn nuevo(tokens: Vec<LexToken>) -> Self {
        Self {
            tokens,
            posicion: 0,
        }
    }

    pub fn parsear_programa(&mut self) -> Result<Programa, ParseError> {
        let mut sentencias = Vec::new();
        while !self.esta_fin() {
            sentencias.push(self.parsear_sentencia()?);
        }
        Ok(Programa { sentencias })
    }

    fn esta_fin(&self) -> bool {
        self.posicion >= self.tokens.len()
    }

    fn peekear(&self) -> Option<&LexToken> {
        self.tokens.get(self.posicion)
    }

    fn avanzar(&mut self) -> Option<&LexToken> {
        if self.esta_fin() {
            None
        } else {
            let t = &self.tokens[self.posicion];
            self.posicion += 1;
            Some(t)
        }
    }

    fn coincidir<F>(&mut self, pred: F) -> bool
    where
        F: FnOnce(&LexToken) -> bool,
    {
        if let Some(t) = self.peekear() {
            if pred(t) {
                self.avanzar();
                return true;
            }
        }
        false
    }

    fn parsear_sentencia(&mut self) -> Result<Sentencia, ParseError> {
        if self.coincidir(|t| matches!(t, LexToken::DeclararVariable)) {
            return variables::parsear_declaracion_variable(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararConstante)) {
            return constantes::parsear_declaracion_constante(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
            return funciones::parsear_declaracion_funcion(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararClase)) {
            return clases::parsear_declaracion_clase(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararInterfaz)) {
            return interfaces::parsear_declaracion_interfaz(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararEnum)) {
            return enums::parsear_declaracion_enum(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::TPrint)) {
            return sentencias::parsear_tprint(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::Return)) {
            return sentencias::parsear_return(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::If)) {
            return controles::parsear_if(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::Switch)) {
            return controles::parsear_switch(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::For)) {
            return controles::parsear_for(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::ForEach)) {
            return controles::parsear_foreach(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::While)) {
            return controles::parsear_while(self);
        }
        if self.coincidir(|t| matches!(t, LexToken::DoWhile)) {
            return controles::parsear_dowhile(self);
        }

        let expr = parsear_expresion_principal(self)?;
        
        if self.coincidir(|t| matches!(t, LexToken::Asignacion)) {
            let valor = parsear_expresion_principal(self)?;
            self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
            
            let nombre = match expr {
                Expresion::Identificador(n) => n,
                Expresion::AccesoPropiedad { objeto, propiedad } => {
                    format!("{}.{}", expr_a_string(&objeto), propiedad)
                }
                Expresion::AccesoIndice { objeto, indice } => {
                    format!("{}[{}]", expr_a_string(&objeto), expr_a_string(&indice))
                }
                _ => return Err(ParseError::nuevo("Objetivo de asignación inválido", self.posicion)),
            };
            
            return Ok(Sentencia::Asignacion(Asignacion { nombre, valor }));
        }
        
        self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
        Ok(Sentencia::Expresion(expr))
    }

    fn parsear_identificador_consumir(&mut self) -> Result<String, ParseError> {
        match self.peekear() {
            Some(LexToken::Identificador(n)) => {
                let nombre = n.clone();
                self.avanzar();
                Ok(nombre)
            }
            _ => Err(ParseError::nuevo(
                "Se esperaba identificador",
                self.posicion,
            )),
        }
    }

    fn parsear_tipo(&mut self) -> Result<Option<Tipo>, ParseError> {
        let mut prefijo = String::new();
        while self.coincidir(|t| matches!(t, LexToken::CorcheteIzq)) {
            if !self.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
                return Err(ParseError::nuevo("Se esperaba ']'", self.posicion));
            }
            prefijo.push_str("[]");
        }
        
        match self.peekear() {
            Some(LexToken::Tipo(n)) => {
                let nombre = format!("{}{}", prefijo, n);
                self.avanzar();
                Ok(Some(Tipo { nombre }))
            }
            Some(LexToken::Identificador(n))
                if n.chars().next().unwrap_or('a').is_ascii_uppercase() =>
            {
                let nombre = format!("{}{}", prefijo, n);
                self.avanzar();
                Ok(Some(Tipo { nombre }))
            }
            _ if !prefijo.is_empty() => {
                Err(ParseError::nuevo("Se esperaba nombre de tipo después de []", self.posicion))
            }
            _ => Ok(None),
        }
    }
}
