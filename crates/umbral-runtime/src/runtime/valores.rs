use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub type FutureValor = Pin<Box<dyn Future<Output = Valor> + Send>>;

#[derive(Debug, Clone)]
pub enum Valor {
    Entero(i64),
    Flotante(f64),
    Booleano(bool),
    Texto(String),
    Lista(Vec<Valor>),
    Diccionario(HashMap<String, Valor>),
    Objeto(Instancia),
    Funcion(Funcion),
    FuncionNativa(String, NativeFn),
    Promesa(SharedPromesa),
    Nulo,
}

pub type NativeFn = fn(Vec<Valor>) -> Valor;

#[derive(Debug, Clone)]
pub struct SharedPromesa(pub Arc<Mutex<Option<tokio::task::JoinHandle<Valor>>>>);

impl PartialEq for SharedPromesa {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Valor {
    pub fn es_verdadero(&self) -> bool {
        match self {
            Valor::Booleano(b) => *b,
            Valor::Nulo => false,
            Valor::Entero(i) => *i != 0,
            Valor::Flotante(f) => *f != 0.0,
            Valor::Texto(s) => !s.is_empty(),
            Valor::Lista(v) => !v.is_empty(),
            _ => true,
        }
    }

    pub fn a_numero(&self) -> Option<f64> {
        match self {
            Valor::Entero(i) => Some(*i as f64),
            Valor::Flotante(f) => Some(*f),
            Valor::Texto(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
}

impl fmt::Display for Valor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Valor::Entero(i) => write!(f, "{}", i),
            Valor::Flotante(fl) => write!(f, "{}", fl),
            Valor::Booleano(b) => write!(f, "{}", b),
            Valor::Texto(s) => write!(f, "{}", s),
            Valor::Nulo => write!(f, "null"),
            Valor::Lista(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Valor::Diccionario(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            }
            Valor::Objeto(inst) => write!(f, "{}", inst),
            Valor::Funcion(func) => write!(f, "<función {}>", func.nombre),
            Valor::FuncionNativa(nombre, _) => write!(f, "<función nativa {}>", nombre),
            Valor::Promesa(_) => write!(f, "<promesa>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instancia {
    pub clase: String,
    pub propiedades: Arc<Mutex<HashMap<String, Valor>>>,
}

impl fmt::Display for Instancia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{ ", self.clase)?;
        let props = self.propiedades.lock().unwrap();
        let mut first = true;
        for (key, value) in props.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
            first = false;
        }
        write!(f, " }}")
    }
}

#[derive(Debug, Clone)]
pub struct Funcion {
    pub nombre: String,
    pub parametros: Vec<String>,
    pub cuerpo: Vec<umbral_parser::ast::Sentencia>,
    pub es_async: bool,
}

impl Funcion {
    pub fn nueva(
        nombre: String,
        parametros: Vec<String>,
        cuerpo: Vec<umbral_parser::ast::Sentencia>,
        es_async: bool,
    ) -> Self {
        Self {
            nombre,
            parametros,
            cuerpo,
            es_async,
        }
    }
}
