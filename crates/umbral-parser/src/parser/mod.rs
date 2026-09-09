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
pub mod umdocs;
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

    pub fn nuevo_con_posiciones(
        tokens_con_pos: Vec<TokenConPosicion>,
        codigo_fuente: String,
    ) -> Self {
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
                let tiene_posiciones_validas =
                    !self.posiciones.is_empty() && self.posicion < self.posiciones.len();

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
            Numero(s) | Cadena(s) | CadenaLiteral(s) | CadenaMultilinea(s) | Identificador(s)
            | Tipo(s) => s.len(),
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

    pub fn esta_fin(&self) -> bool {
        self.posicion >= self.tokens.len()
    }

    pub fn peekear(&self) -> Option<&LexToken> {
        self.tokens.get(self.posicion)
    }

    pub fn avanzar(&mut self) -> Option<&LexToken> {
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
        let Some(t) = self.peekear() else {
            return false;
        };
        if !pred(t) {
            return false;
        }
        self.avanzar();
        true
    }

    fn resolver_doc(a: Option<UmDoc>, b: Option<UmDoc>) -> Option<UmDoc> {
        if b.is_some() {
            return b;
        }
        a
    }

    fn parsear_sentencia(&mut self) -> Result<Sentencia, ParseError> {
        let doc_previo = umdocs::consumir_doc(self);
        let exportado = self.coincidir(|t| matches!(t, LexToken::Out));
        let doc = Self::resolver_doc(doc_previo, umdocs::consumir_doc(self));
        if let Some(res) = self.intentar_parsear_declaraciones(exportado, doc.clone()) {
            return res;
        }
        let _ = doc;

        if exportado {
            let nombre = self.parsear_identificador_consumir()?;
            self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
            return Ok(Sentencia::Exportacion(nombre));
        }

        let resultado = self
            .intentar_parsear_controles()
            .or_else(|| self.intentar_parsear_comandos());

        if let Some(sentencia) = resultado {
            return sentencia;
        }

        self.parsear_expresion_o_asignacion()
    }

    fn intentar_parsear_declaraciones(
        &mut self,
        exportado: bool,
        doc: Option<UmDoc>,
    ) -> Option<Result<Sentencia, ParseError>> {
        if self.coincidir(|t| matches!(t, LexToken::Equip))
            || self.coincidir(|t| matches!(t, LexToken::Origin))
        {
            self.posicion -= 1;
            return Some(importaciones::parsear_importacion(self));
        }

        if self.coincidir(|t| matches!(t, LexToken::DeclararVariable)) {
            return Some(variables::parsear_declaracion_variable(self, exportado));
        }

        if self.coincidir(|t| matches!(t, LexToken::DeclararConstante)) {
            return Some(constantes::parsear_declaracion_constante(self, exportado));
        }

        if let Some(LexToken::DeclararFuncion) = self.peekear() {
            return Some(funciones::parsear_declaracion_funcion(self, exportado, doc));
        }

        if let Some(LexToken::Asy) = self.peekear() {
            return Some(funciones::parsear_declaracion_funcion(self, exportado, doc));
        }

        if self.coincidir(|t| matches!(t, LexToken::DeclararClase)) {
            return Some(clases::parsear_declaracion_clase_con_doc(self, exportado, doc));
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

        if self.coincidir(|t| matches!(t, LexToken::Try)) {
            return Some(controles::parsear_try_catch(self));
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

        if self.coincidir(|t| matches!(t, LexToken::Throw)) {
            return Some(sentencias::parsear_throw(self));
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

    fn parsear_asignacion_con_objetivo(
        &mut self,
        expresion: Expresion,
    ) -> Result<Sentencia, ParseError> {
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

        let base = match self.peekear() {
            Some(LexToken::Tipo(n)) => {
                let n = n.clone();
                self.avanzar();
                n
            }
            Some(LexToken::Identificador(n))
                if n.chars().next().unwrap_or('a').is_ascii_uppercase() =>
            {
                let n = n.clone();
                self.avanzar();
                n
            }
            _ if !prefijo.is_empty() => {
                return Err(self.crear_error("Se esperaba nombre de tipo después de []"));
            }
            _ => return Ok(None),
        };

        let nombre_base = format!("{}{}", prefijo, base);
        if self.coincidir(|t| matches!(t, LexToken::Menor)) {
            let args = self.parsear_argumentos_genericos()?;
            return Ok(Some(Tipo::generico(nombre_base, args)));
        }

        Ok(Some(Tipo::simple(nombre_base)))
    }

    pub fn parsear_argumentos_genericos(&mut self) -> Result<Vec<Tipo>, ParseError> {
        let mut args = Vec::new();
        loop {
            let siguiente = self.parsear_tipo()?;
            let Some(tipo) = siguiente else {
                return Err(self.crear_error("Se esperaba un tipo dentro de '<>'"));
            };
            args.push(tipo);
            if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                continue;
            }
            break;
        }
        if !self.coincidir(|t| matches!(t, LexToken::Mayor)) {
            return Err(self.crear_error("Se esperaba '>' para cerrar los argumentos genéricos"));
        }
        Ok(args)
    }

    pub fn parsear_parametros_tipo(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.coincidir(|t| matches!(t, LexToken::Menor)) {
            return Ok(Vec::new());
        }
        let mut params = Vec::new();
        loop {
            self.consumir_parametro_tipo(&mut params)?;
            if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                continue;
            }
            break;
        }
        if !self.coincidir(|t| matches!(t, LexToken::Mayor)) {
            return Err(self.crear_error("Se esperaba '>' para cerrar los parámetros genéricos"));
        }
        Ok(params)
    }

    fn consumir_parametro_tipo(&mut self, params: &mut Vec<String>) -> Result<(), ParseError> {
        let nombre = self.leer_nombre_tipo()?;
        let Some(nombre) = nombre else {
            return Err(self.crear_error("Se esperaba un parámetro genérico (ej. T)"));
        };
        if params.contains(&nombre) {
            return Err(self.crear_error(format!("Parámetro genérico '{}' duplicado", nombre)));
        }
        params.push(nombre);
        Ok(())
    }

    fn leer_nombre_tipo(&mut self) -> Result<Option<String>, ParseError> {
        match self.peekear() {
            Some(LexToken::Identificador(n)) | Some(LexToken::Tipo(n))
                if n.chars().next().unwrap_or('a').is_ascii_uppercase() =>
            {
                let nombre = n.clone();
                self.avanzar();
                Ok(Some(nombre))
            }
            _ => Ok(None),
        }
    }
}
