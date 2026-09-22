use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::net::{TcpListener, TcpStream, UdpSocket};
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
    Enchufe(ManejadorEnchufe),
    Clase(String),
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

    pub fn nombre_tipo(&self) -> String {
        match self {
            Valor::Entero(_) => "Int".to_string(),
            Valor::Flotante(_) => "Flo".to_string(),
            Valor::Booleano(_) => "Bool".to_string(),
            Valor::Texto(_) => "Str".to_string(),
            Valor::Lista(_) => "Array".to_string(),
            Valor::Diccionario(_) => "Objeto".to_string(),
            Valor::Objeto(inst) => inst.clase.clone(),
            Valor::Funcion(f) => format!("Func({})", f.nombre),
            Valor::FuncionNativa(n, _) => format!("FuncNativa({})", n),
            Valor::Promesa(_) => "Promesa".to_string(),
            Valor::Enchufe(_) => "Enchufe".to_string(),
            Valor::Clase(n) => format!("Clase({})", n),
            Valor::Nulo => "Null".to_string(),
        }
    }

    pub fn es_tipo_compatible(&self, tipo: &str) -> bool {
        let t = tipo.trim();
        if t == "Any" {
            return true;
        }
        if es_parametro_tipo(t) {
            return true;
        }
        let base = base_de_tipo(t);
        match self {
            Valor::Entero(_) => base == "Int" || base == "Flo",
            Valor::Flotante(_) => base == "Flo",
            Valor::Booleano(_) => base == "Bool",
            Valor::Texto(_) => base == "Str",
            Valor::Lista(_) => base.starts_with("[]") || base == "Array",
            Valor::Diccionario(_) => base == "Objeto" || base == "Obj",
            Valor::Objeto(inst) => {
                base == "Objeto" || base == "Obj" || base == inst.clase
            }
            Valor::Funcion(_) | Valor::FuncionNativa(..) => base == "Func",
            Valor::Promesa(_) => base == "Promesa",
            Valor::Enchufe(_) => base == "Enchufe",
            Valor::Clase(_) => base == "Clase",
            Valor::Nulo => true,
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
                write!(f, "[")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\" => {}", key, value)?;
                    first = false;
                }
                write!(f, "]")
            }
            Valor::Objeto(inst) => write!(f, "{}", inst),
            Valor::Funcion(func) => write!(f, "<función {}>", func.nombre),
            Valor::FuncionNativa(nombre, _) => write!(f, "<función nativa {}>", nombre),
            Valor::Promesa(_) => write!(f, "<promesa>"),
            Valor::Enchufe(_) => write!(f, "<enchufe>"),
            Valor::Clase(nombre) => write!(f, "<clase {}>", nombre),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaseEnchufe {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub struct InteriorEnchufe {
    pub clase: ClaseEnchufe,
    pub pendiente: Option<socket2::Socket>,
    pub oyente: Option<TcpListener>,
    pub flujo: Option<TcpStream>,
    pub datagrama: Option<UdpSocket>,
    pub lectura_ms: Option<u64>,
    pub escritura_ms: Option<u64>,
    pub reuso: bool,
    pub cerrado: bool,
}

#[derive(Debug, Clone)]
pub struct ManejadorEnchufe(pub Arc<Mutex<InteriorEnchufe>>);

#[derive(Debug, Clone)]
pub struct Instancia {
    pub clase: String,
    pub propiedades: Arc<Mutex<HashMap<String, Valor>>>,
}

impl fmt::Display for Instancia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}([", self.clase)?;
        let props = self.propiedades.lock().unwrap();
        let mut first = true;
        for (key, value) in props.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"{}\" => {}", key, value)?;
            first = false;
        }
        write!(f, "])")
    }
}

pub fn base_de_tipo(tipo: &str) -> &str {
    match tipo.find('<') {
        Some(idx) => tipo[..idx].trim(),
        None => tipo.trim(),
    }
}

pub fn es_parametro_tipo(tipo: &str) -> bool {
    let t = tipo.trim();
    t.len() == 1 && t.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Documenta un parámetro rest para mostrarlo en firmas de ayuda.
/// Con tipo produce `...nombre->Tipo`; sin tipo produce `...nombre`.
pub fn formatear_rest(rest: &ParametroRest) -> String {
    rest.tipo.as_ref().map_or_else(
        || format!("...{}", rest.nombre),
        |t| format!("...{}->{}", rest.nombre, t),
    )
}

#[derive(Debug, Clone)]
pub struct ParametroRest {
    pub nombre: String,
    pub tipo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Funcion {
    pub nombre: String,
    pub parametros: Vec<String>,
    pub parametro_rest: Option<ParametroRest>,
    pub cuerpo: Vec<umbral_parser::ast::Sentencia>,
    pub es_async: bool,
    pub doc: Option<umbral_parser::ast::UmDoc>,
    pub entorno_capturado: Option<HashMap<String, Valor>>,
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
            parametro_rest: None,
            cuerpo,
            es_async,
            doc: None,
            entorno_capturado: None,
        }
    }

    pub fn con_doc(
        nombre: String,
        parametros: Vec<String>,
        cuerpo: Vec<umbral_parser::ast::Sentencia>,
        es_async: bool,
        doc: Option<umbral_parser::ast::UmDoc>,
    ) -> Self {
        Self {
            nombre,
            parametros,
            parametro_rest: None,
            cuerpo,
            es_async,
            doc,
            entorno_capturado: None,
        }
    }

    pub fn con_rest(
        nombre: String,
        parametros: Vec<String>,
        parametro_rest: Option<ParametroRest>,
        cuerpo: Vec<umbral_parser::ast::Sentencia>,
        es_async: bool,
        doc: Option<umbral_parser::ast::UmDoc>,
    ) -> Self {
        Self {
            nombre,
            parametros,
            parametro_rest,
            cuerpo,
            es_async,
            doc,
            entorno_capturado: None,
        }
    }

    /// Crea una copia de la función con su entorno de origen adjunto.
    pub fn con_captura(self, captura: HashMap<String, Valor>) -> Self {
        Self {
            entorno_capturado: Some(captura),
            ..self
        }
    }

    pub fn firma(&self) -> String {
        let fijos = self.parametros.iter().cloned();
        let resto = self.parametro_rest.iter().map(formatear_rest);
        let partes: Vec<String> = fijos.chain(resto).collect();
        format!("({})", partes.join(", "))
    }

    pub fn texto_ayuda(&self) -> String {
        match &self.doc {
            Some(d) => d.formatear(&self.nombre, &self.firma()),
            None => format!("{} {}\n  (sin documentación umdocs)", self.nombre, self.firma()),
        }
    }
}
