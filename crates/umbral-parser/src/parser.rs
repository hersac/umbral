use crate::ast::*;
use crate::error::ParseError;
use umbral_lexer::Token as LexToken;

pub struct Parser {
    tokens: Vec<LexToken>,
    posicion: usize,
}

impl Parser {
    pub fn nuevo(tokens: Vec<LexToken>) -> Self {
        Self {
            tokens,
            posicion: 0,
        }
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

    pub fn parsear_programa(&mut self) -> Result<Programa, ParseError> {
        let mut sentencias = Vec::new();
        while !self.esta_fin() {
            sentencias.push(self.parsear_sentencia()?);
        }
        Ok(Programa { sentencias })
    }

    fn parsear_sentencia(&mut self) -> Result<Sentencia, ParseError> {
        if self.coincidir(|t| matches!(t, LexToken::DeclararVariable)) {
            return self.parsear_declaracion_variable();
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararConstante)) {
            return self.parsear_declaracion_constante();
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararFuncion)) {
            return self.parsear_declaracion_funcion();
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararClase)) {
            return self.parsear_declaracion_clase();
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararInterfaz)) {
            return self.parsear_declaracion_interfaz();
        }
        if self.coincidir(|t| matches!(t, LexToken::DeclararEnum)) {
            return self.parsear_declaracion_enum();
        }
        if self.coincidir(|t| matches!(t, LexToken::TPrint)) {
            return self.parsear_tprint();
        }

        if let Some(LexToken::Identificador(_)) = self.peekear() {
            if self.posicion + 1 < self.tokens.len() {
                if matches!(self.tokens[self.posicion + 1], LexToken::ParentesisIzq) {
                    return self.parsear_llamado_funcion();
                }
                if matches!(self.tokens[self.posicion + 1], LexToken::Asignacion) {
                    return self.parsear_asignacion();
                }
            }
        }

        Ok(Sentencia::Expresion(self.parsear_expresion()?))
    }

    fn parsear_identificador_consumir(&mut self) -> Result<String, ParseError> {
        let nombre_opt = match self.peekear() {
            Some(LexToken::Identificador(n)) => Some(n.clone()),
            _ => None,
        };
        if let Some(nombre) = nombre_opt {
            self.avanzar();
            Ok(nombre)
        } else {
            Err(ParseError::nuevo(
                "Se esperaba identificador",
                self.posicion,
            ))
        }
    }

    fn parsear_tipo(&mut self) -> Result<Option<Tipo>, ParseError> {
        let posible = match self.peekear() {
            Some(LexToken::Tipo(n)) => Some(n.clone()),
            Some(LexToken::Identificador(n))
                if n.chars().next().unwrap_or('a').is_ascii_uppercase() =>
            {
                Some(n.clone())
            }
            _ => None,
        };
        if let Some(nombre) = posible {
            self.avanzar();
            Ok(Some(Tipo { nombre }))
        } else {
            Ok(None)
        }
    }

    fn parsear_declaracion_variable(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        let tipo = if self.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
            self.parsear_tipo()?
        } else {
            None
        };
        if !self.coincidir(|t| matches!(t, LexToken::Asignacion)) {
            return Err(ParseError::nuevo(
                "Se esperaba '=' en declaracion variable",
                self.posicion,
            ));
        }
        let valor = self.parsear_expresion()?;
        self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
        Ok(Sentencia::DeclaracionVariable(DeclaracionVariable {
            nombre,
            tipo,
            valor,
        }))
    }

    fn parsear_declaracion_constante(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        if !self.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
            return Err(ParseError::nuevo(
                "Se esperaba '->' en declaracion constante",
                self.posicion,
            ));
        }
        let tipo = match self.parsear_tipo()? {
            Some(t) => t,
            None => {
                return Err(ParseError::nuevo(
                    "Tipo esperado en declaracion constante",
                    self.posicion,
                ));
            }
        };
        if !self.coincidir(|t| matches!(t, LexToken::Asignacion)) {
            return Err(ParseError::nuevo(
                "Se esperaba '=' en declaracion constante",
                self.posicion,
            ));
        }
        let valor = self.parsear_expresion()?;
        self.coincidir(|t| matches!(t, LexToken::PuntoYComa));
        Ok(Sentencia::DeclaracionConstante(DeclaracionConstante {
            nombre,
            tipo,
            valor,
        }))
    }

    fn parsear_parametro(&mut self) -> Result<Parametro, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        let tipo = if self.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
            self.parsear_tipo()?
        } else {
            None
        };
        Ok(Parametro { nombre, tipo })
    }

    fn parsear_lista_parametros(&mut self) -> Result<Vec<Parametro>, ParseError> {
        if !self.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
            return Err(ParseError::nuevo(
                "Se esperaba '(' en definición de función",
                self.posicion,
            ));
        }
        let mut lista = Vec::new();
        if self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            return Ok(lista);
        }
        loop {
            lista.push(self.parsear_parametro()?);
            if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                break;
            }
            return Err(ParseError::nuevo(
                "Se esperaba ',' o ')' en lista de parámetros",
                self.posicion,
            ));
        }
        Ok(lista)
    }

    fn parsear_bloque(&mut self) -> Result<Vec<Sentencia>, ParseError> {
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

    fn parsear_declaracion_funcion(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        let parametros = self.parsear_lista_parametros()?;
        let tipo_retorno = if self.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
            self.parsear_tipo()?
        } else {
            None
        };
        let cuerpo = self.parsear_bloque()?;
        Ok(Sentencia::Funcion(DeclaracionFuncion {
            nombre,
            parametros,
            tipo_retorno,
            cuerpo,
        }))
    }

    fn parsear_llamado_funcion(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        if !self.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
            return Err(ParseError::nuevo(
                "Se esperaba '(' en llamada",
                self.posicion,
            ));
        }
        let mut argumentos = Vec::new();
        if !self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            loop {
                argumentos.push(self.parsear_expresion()?);
                if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                    continue;
                }
                if self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                    break;
                }
                return Err(ParseError::nuevo("Se esperaba ',' o ')'", self.posicion));
            }
        }
        Ok(Sentencia::LlamadoFuncion(LlamadoFuncion {
            nombre,
            argumentos,
        }))
    }

    fn parsear_declaracion_clase(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        let miembros = self.parsear_bloque()?;
        let mut propiedades = Vec::new();
        let mut metodos = Vec::new();
        for s in miembros {
            match s {
                Sentencia::DeclaracionVariable(dv) => propiedades.push(Propiedad {
                    nombre: dv.nombre,
                    tipo: dv.tipo,
                    publico: false,
                    valor_inicial: Some(dv.valor),
                }),
                Sentencia::Funcion(f) => metodos.push(Metodo {
                    nombre: f.nombre,
                    parametros: f.parametros,
                    tipo_retorno: f.tipo_retorno,
                    cuerpo: f.cuerpo,
                    publico: true,
                }),
                _ => {}
            }
        }
        Ok(Sentencia::Clase(DeclaracionClase {
            nombre,
            propiedades,
            metodos,
        }))
    }

    fn parsear_declaracion_interfaz(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        let miembros = self.parsear_bloque()?;
        let mut metodos = Vec::new();
        for s in miembros {
            if let Sentencia::Funcion(f) = s {
                metodos.push(Metodo {
                    nombre: f.nombre,
                    parametros: f.parametros,
                    tipo_retorno: f.tipo_retorno,
                    cuerpo: f.cuerpo,
                    publico: true,
                });
            }
        }
        Ok(Sentencia::Interfaz(DeclaracionInterfaz { nombre, metodos }))
    }

    fn parsear_declaracion_enum(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        if !self.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
            return Err(ParseError::nuevo("Se esperaba '{' en enum", self.posicion));
        }
        let mut variantes = Vec::new();
        loop {
            match self.avanzar() {
                Some(LexToken::Identificador(v)) => {
                    variantes.push(v.clone());
                    if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                        continue;
                    }
                }
                Some(LexToken::LlaveDer) => break,
                Some(_) => {
                    return Err(ParseError::nuevo(
                        "Entrada no valida en enum",
                        self.posicion,
                    ));
                }
                None => return Err(ParseError::nuevo("Fin inesperado en enum", self.posicion)),
            }
        }
        Ok(Sentencia::Enum(DeclaracionEnum { nombre, variantes }))
    }

    fn parsear_tprint(&mut self) -> Result<Sentencia, ParseError> {
        if !self.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
            return Err(ParseError::nuevo(
                "Se esperaba '(' despues de tprint",
                self.posicion,
            ));
        }
        let valor = self.parsear_expresion()?;
        if !self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            return Err(ParseError::nuevo("Se esperaba ')'", self.posicion));
        }
        Ok(Sentencia::LlamadoTPrint(LlamadoTPrint { valor }))
    }

    fn parsear_asignacion(&mut self) -> Result<Sentencia, ParseError> {
        let nombre = self.parsear_identificador_consumir()?;
        if !self.coincidir(|t| matches!(t, LexToken::Asignacion)) {
            return Err(ParseError::nuevo(
                "Se esperaba '=' en asignacion",
                self.posicion,
            ));
        }
        let valor = self.parsear_expresion()?;
        Ok(Sentencia::Asignacion(Asignacion { nombre, valor }))
    }

    fn parsear_expresion(&mut self) -> Result<Expresion, ParseError> {
        self.parsear_or()
    }

    fn parsear_or(&mut self) -> Result<Expresion, ParseError> {
        let mut izquierda = self.parsear_and()?;
        while self.coincidir(|t| matches!(t, LexToken::Or)) {
            let derecha = self.parsear_and()?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "||".to_string(),
                derecha: Box::new(derecha),
            };
        }
        Ok(izquierda)
    }

    fn parsear_and(&mut self) -> Result<Expresion, ParseError> {
        let mut izquierda = self.parsear_igualdad()?;
        while self.coincidir(|t| matches!(t, LexToken::And)) {
            let derecha = self.parsear_igualdad()?;
            izquierda = Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: "&&".to_string(),
                derecha: Box::new(derecha),
            };
        }
        Ok(izquierda)
    }

    fn parsear_igualdad(&mut self) -> Result<Expresion, ParseError> {
        let mut izquierda = self.parsear_comparacion()?;
        loop {
            if self.coincidir(|t| matches!(t, LexToken::IgualIgual)) {
                let derecha = self.parsear_comparacion()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "==".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::Diferente)) {
                let derecha = self.parsear_comparacion()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "!=".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            break;
        }
        Ok(izquierda)
    }

    fn parsear_comparacion(&mut self) -> Result<Expresion, ParseError> {
        let mut izquierda = self.parsear_termino()?;
        loop {
            if self.coincidir(|t| matches!(t, LexToken::Menor)) {
                let derecha = self.parsear_termino()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "<".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::MenorIgual)) {
                let derecha = self.parsear_termino()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "<=".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::Mayor)) {
                let derecha = self.parsear_termino()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: ">".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::MayorIgual)) {
                let derecha = self.parsear_termino()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: ">=".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            break;
        }
        Ok(izquierda)
    }

    fn parsear_termino(&mut self) -> Result<Expresion, ParseError> {
        let mut izquierda = self.parsear_factor()?;
        loop {
            if self.coincidir(|t| matches!(t, LexToken::Suma)) {
                let derecha = self.parsear_factor()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "+".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::Resta)) {
                let derecha = self.parsear_factor()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "-".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            break;
        }
        Ok(izquierda)
    }

    fn parsear_factor(&mut self) -> Result<Expresion, ParseError> {
        let mut izquierda = self.parsear_unaria()?;
        loop {
            if self.coincidir(|t| matches!(t, LexToken::Multiplicacion)) {
                let derecha = self.parsear_unaria()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "*".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::Division)) {
                let derecha = self.parsear_unaria()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "/".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::Modulo)) {
                let derecha = self.parsear_unaria()?;
                izquierda = Expresion::Binaria {
                    izquierda: Box::new(izquierda),
                    operador: "%".to_string(),
                    derecha: Box::new(derecha),
                };
                continue;
            }
            break;
        }
        Ok(izquierda)
    }

    fn parsear_unaria(&mut self) -> Result<Expresion, ParseError> {
        if self.coincidir(|t| matches!(t, LexToken::Not)) {
            let expr = self.parsear_unaria()?;
            return Ok(Expresion::Unaria {
                operador: "!".to_string(),
                expresion: Box::new(expr),
            });
        }
        if self.coincidir(|t| matches!(t, LexToken::Resta)) {
            let expr = self.parsear_unaria()?;
            return Ok(Expresion::Unaria {
                operador: "-".to_string(),
                expresion: Box::new(expr),
            });
        }
        self.parsear_primaria()
    }

    fn parsear_primaria(&mut self) -> Result<Expresion, ParseError> {
        let look = match self.peekear() {
            Some(LexToken::Numero(n)) => Some(("numero", n.clone())),
            Some(LexToken::Cadena(s)) => Some(("cadena", s.clone())),
            Some(LexToken::CadenaMultilinea(s)) => Some(("cadena_multilinea", s.clone())),
            Some(LexToken::Verdadero) => Some(("verdadero", String::new())),
            Some(LexToken::Falso) => Some(("falso", String::new())),
            Some(LexToken::Identificador(n)) => Some(("identificador", n.clone())),
            Some(LexToken::ParentesisIzq) => Some(("par_izq", String::new())),
            Some(LexToken::CorcheteIzq) => Some(("corchete_izq", String::new())),
            Some(LexToken::LlaveIzq) => Some(("llave_izq", String::new())),
            _ => None,
        };

        match look {
            Some(("numero", s)) => {
                self.avanzar();
                if s.contains('.') {
                    Ok(Expresion::LiteralFloat(s.parse::<f64>().unwrap_or(0.0)))
                } else {
                    Ok(Expresion::LiteralEntero(s.parse::<i64>().unwrap_or(0)))
                }
            }
            Some(("cadena", s)) | Some(("cadena_multilinea", s)) => {
                self.avanzar();
                Ok(Expresion::LiteralCadena(s))
            }
            Some(("verdadero", _)) => {
                self.avanzar();
                Ok(Expresion::LiteralBool(true))
            }
            Some(("falso", _)) => {
                self.avanzar();
                Ok(Expresion::LiteralBool(false))
            }
            Some(("identificador", nombre)) => {
                if self.posicion + 1 < self.tokens.len()
                    && matches!(self.tokens[self.posicion + 1], LexToken::ParentesisIzq)
                {
                    self.avanzar();
                    if !self.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
                        return Err(ParseError::nuevo(
                            "Se esperaba '(' despues de identificador",
                            self.posicion,
                        ));
                    }
                    let mut args = Vec::new();
                    if !self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                        loop {
                            args.push(self.parsear_expresion()?);
                            if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                                continue;
                            }
                            if self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                                break;
                            }
                            return Err(ParseError::nuevo("Se esperaba ',' o ')'", self.posicion));
                        }
                    }
                    Ok(Expresion::Instanciacion {
                        tipo: nombre,
                        argumentos: args,
                    })
                } else {
                    self.avanzar();
                    Ok(Expresion::Identificador(nombre))
                }
            }
            Some(("par_izq", _)) => {
                self.avanzar();
                let expr = self.parsear_expresion()?;
                if !self.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
                    return Err(ParseError::nuevo("Se esperaba ')'", self.posicion));
                }
                Ok(Expresion::Agrupada(Box::new(expr)))
            }
            Some(("corchete_izq", _)) => {
                self.avanzar();
                self.parsear_array()
            }
            Some(("llave_izq", _)) => {
                self.avanzar();
                self.parsear_objeto()
            }
            _ => Err(ParseError::nuevo("Expresion no valida", self.posicion)),
        }
    }

    fn parsear_array(&mut self) -> Result<Expresion, ParseError> {
        let mut elementos = Vec::new();
        if self.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
            return Ok(Expresion::Array(elementos));
        }
        loop {
            elementos.push(self.parsear_expresion()?);
            if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                continue;
            }
            if self.coincidir(|t| matches!(t, LexToken::CorcheteDer)) {
                break;
            }
            return Err(ParseError::nuevo("Se esperaba ',' o ']'", self.posicion));
        }
        Ok(Expresion::Array(elementos))
    }

    fn parsear_objeto(&mut self) -> Result<Expresion, ParseError> {
        let mut pares = Vec::new();
        if self.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
            return Ok(Expresion::Objeto(pares));
        }
        loop {
            let clave = match self.peekear() {
                Some(LexToken::Identificador(n)) => Some(n.clone()),
                Some(LexToken::Cadena(s)) => Some(s.clone()),
                _ => None,
            };
            if let Some(k) = clave {
                self.avanzar();
                if !self.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
                    return Err(ParseError::nuevo(
                        "Se esperaba '=>' en objeto",
                        self.posicion,
                    ));
                }
                let valor = self.parsear_expresion()?;
                pares.push((k, valor));
                if self.coincidir(|t| matches!(t, LexToken::Coma)) {
                    continue;
                }
                if self.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
                    break;
                }
                return Err(ParseError::nuevo(
                    "Se esperaba ',' o '}' en objeto",
                    self.posicion,
                ));
            } else {
                return Err(ParseError::nuevo("Clave de objeto esperada", self.posicion));
            }
        }
        Ok(Expresion::Objeto(pares))
    }
}
