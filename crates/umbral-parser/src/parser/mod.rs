pub mod clases;
pub mod constantes;
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

        if let Some(LexToken::Identificador(_)) = self.peekear() {
            if self.posicion + 1 < self.tokens.len() {
                if matches!(self.tokens[self.posicion + 1], LexToken::ParentesisIzq) {
                    return sentencias::parsear_llamado_funcion(self);
                }
                if matches!(self.tokens[self.posicion + 1], LexToken::Asignacion) {
                    return sentencias::parsear_asignacion(self);
                }
            }
        }

        Ok(Sentencia::Expresion(parsear_expresion_principal(self)?))
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
        match self.peekear() {
            Some(LexToken::Tipo(n)) => {
                let nombre = n.clone();
                self.avanzar();
                Ok(Some(Tipo { nombre }))
            }
            Some(LexToken::Identificador(n))
                if n.chars().next().unwrap_or('a').is_ascii_uppercase() =>
            {
                let nombre = n.clone();
                self.avanzar();
                Ok(Some(Tipo { nombre }))
            }
            _ => Ok(None),
        }
    }
}
