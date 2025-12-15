pub mod clases;
pub mod constantes;
pub mod controles;
pub mod enums;
pub mod expresiones;
pub mod funciones;
pub mod importaciones;
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
use umbral_lexer::{Token as LexToken, TokenConPosicion};

pub struct Parser {
    pub tokens: Vec<LexToken>,
    pub posiciones: Vec<usize>,
    pub posicion: usize,
    pub codigo_fuente: String,
}

impl Parser {
    pub fn nuevo(tokens: Vec<LexToken>) -> Self {
        Self {
            tokens,
            posiciones: Vec::new(),
            posicion: 0,
            codigo_fuente: String::new(),
        }
    }

    pub fn nuevo_con_codigo(tokens: Vec<LexToken>, codigo_fuente: String) -> Self {
        Self {
            tokens,
            posiciones: Vec::new(),
            posicion: 0,
            codigo_fuente,
        }
    }

    pub fn nuevo_con_posiciones(tokens_con_pos: Vec<TokenConPosicion>, codigo_fuente: String) -> Self {
        let tokens: Vec<LexToken> = tokens_con_pos.iter().map(|tp| tp.token.clone()).collect();
        let posiciones: Vec<usize> = tokens_con_pos.iter().map(|tp| tp.posicion).collect();
        Self {
            tokens,
            posiciones,
            posicion: 0,
            codigo_fuente,
        }
    }

    pub fn crear_error(&self, mensaje: impl Into<String>) -> ParseError {
        match self.codigo_fuente.is_empty() {
            true => ParseError::nuevo(mensaje, self.posicion),
            false => {
                let tiene_posiciones_validas = !self.posiciones.is_empty() 
                    && self.posicion < self.posiciones.len();
                
                let posicion_char = match tiene_posiciones_validas {
                    true => self.posiciones[self.posicion],
                    false => self.estimar_posicion_caracter(),
                };
                
                ParseError::con_contexto(mensaje, posicion_char, &self.codigo_fuente)
            }
        }
    }

    fn estimar_posicion_caracter(&self) -> usize {
        let mut pos = 0;
        for i in 0..self.posicion.min(self.tokens.len()) {
            pos += self.estimar_longitud_token(&self.tokens[i]);
            pos += 1;
        }
        pos.min(self.codigo_fuente.len().saturating_sub(1))
    }

    fn estimar_longitud_token(&self, token: &LexToken) -> usize {
        use LexToken::*;
        match token {
            Numero(s) | Cadena(s) | CadenaLiteral(s) | CadenaMultilinea(s) | Identificador(s) | Tipo(s) => s.len(),
            DeclararVariable => 2,
            DeclararConstante => 2,
            DeclararFuncion => 2,
            Instanciar => 2,
            DeclararClase => 3,
            DeclararInterfaz => 3,
            DeclararEnum => 3,
            _ => 2,
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
        let exportado = self.coincidir(|t| matches!(t, LexToken::Out));
        
        let resultado = self.intentar_parsear_declaraciones(exportado)
            .or_else(|| self.intentar_parsear_controles())
            .or_else(|| self.intentar_parsear_comandos());
        
        if let Some(sentencia) = resultado {
            return sentencia;
        }
        
        self.parsear_expresion_o_asignacion()
    }
    
    fn intentar_parsear_declaraciones(&mut self, exportado: bool) -> Option<Result<Sentencia, ParseError>> {
        if self.coincidir(|t| matches!(t, LexToken::Equip)) || self.coincidir(|t| matches!(t, LexToken::Origin)) {
            self.posicion -= 1;
            return Some(importaciones::parsear_importacion(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DeclararVariable)) {
            return Some(variables::parsear_declaracion_variable(self, exportado));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DeclararConstante)) {
            return Some(constantes::parsear_declaracion_constante(self, exportado));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
            return Some(funciones::parsear_declaracion_funcion(self, exportado));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DeclararClase)) {
            return Some(clases::parsear_declaracion_clase(self, exportado));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DeclararInterfaz)) {
            return Some(interfaces::parsear_declaracion_interfaz(self, exportado));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DeclararEnum)) {
            return Some(enums::parsear_declaracion_enum(self, exportado));
        }
        
        None
    }
    
    fn intentar_parsear_controles(&mut self) -> Option<Result<Sentencia, ParseError>> {
        if self.coincidir(|t| matches!(t, LexToken::If)) {
            return Some(controles::parsear_if(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::Switch)) {
            return Some(controles::parsear_switch(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::For)) {
            return Some(controles::parsear_for(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::ForEach)) {
            return Some(controles::parsear_foreach(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::While)) {
            return Some(controles::parsear_while(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::DoWhile)) {
            return Some(controles::parsear_dowhile(self));
        }
        
        None
    }
    
    fn intentar_parsear_comandos(&mut self) -> Option<Result<Sentencia, ParseError>> {
        if self.coincidir(|t| matches!(t, LexToken::TPrint)) {
            return Some(sentencias::parsear_tprint(self));
        }
        
        if self.coincidir(|t| matches!(t, LexToken::Return)) {
            return Some(sentencias::parsear_return(self));
        }
        
        None
    }
    
    fn parsear_expresion_o_asignacion(&mut self) -> Result<Sentencia, ParseError> {
        let expresion = parsear_expresion_principal(self)?;
        
        if !self.coincidir(|t| matches!(t, LexToken::Asignacion)) {
            self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
            return Ok(Sentencia::Expresion(expresion));
        }
        
        self.parsear_asignacion_con_objetivo(expresion)
    }
    
    fn parsear_asignacion_con_objetivo(&mut self, expresion: Expresion) -> Result<Sentencia, ParseError> {
        let valor = parsear_expresion_principal(self)?;
        self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
        
        let objetivo = match expresion {
            Expresion::Identificador(nombre) => ObjetivoAsignacion::Variable(nombre),
            Expresion::AccesoPropiedad { objeto, propiedad } => {
                ObjetivoAsignacion::Propiedad { objeto, propiedad }
            }
            _ => return Err(self.crear_error("Objetivo de asignación inválido")),
        };
        
        Ok(Sentencia::Asignacion(Asignacion { objetivo, valor }))
    }

    fn parsear_identificador_consumir(&mut self) -> Result<String, ParseError> {
        match self.peekear() {
            Some(LexToken::Identificador(n)) => {
                let nombre = n.clone();
                self.avanzar();
                Ok(nombre)
            }
            _ => Err(self.crear_error("Se esperaba identificador")),
        }
    }

    fn parsear_tipo(&mut self) -> Result<Option<Tipo>, ParseError> {
        let mut prefijo = String::new();
        while self.coincidir(|t| matches!(t, LexToken::CorcheteIzq)) {
            if !self.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
                return Err(self.crear_error("Se esperaba ']'"));
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
                Err(self.crear_error("Se esperaba nombre de tipo después de []"))
            }
            _ => Ok(None),
        }
    }
}
