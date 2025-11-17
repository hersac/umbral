#[derive(Debug, Clone)]
pub struct Programa {
    pub sentencias: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub enum Sentencia {
    DeclaracionVariable(DeclaracionVariable),
    DeclaracionConstante(DeclaracionConstante),
    Asignacion(Asignacion),
    LlamadoTPrint(LlamadoTPrint),
    Expresion(Expresion),
    Funcion(DeclaracionFuncion),
    Clase(DeclaracionClase),
    Interfaz(DeclaracionInterfaz),
    Enum(DeclaracionEnum),
    LlamadoFuncion(LlamadoFuncion),
}

#[derive(Debug, Clone)]
pub struct DeclaracionVariable {
    pub nombre: String,
    pub tipo: Option<Tipo>,
    pub valor: Expresion,
}

#[derive(Debug, Clone)]
pub struct DeclaracionConstante {
    pub nombre: String,
    pub tipo: Tipo,
    pub valor: Expresion,
}

#[derive(Debug, Clone)]
pub struct Asignacion {
    pub nombre: String,
    pub valor: Expresion,
}

#[derive(Debug, Clone)]
pub struct LlamadoTPrint {
    pub valor: Expresion,
}

#[derive(Debug, Clone)]
pub struct Tipo {
    pub nombre: String,
}

#[derive(Debug, Clone)]
pub struct Parametro {
    pub nombre: String,
    pub tipo: Option<Tipo>,
}

#[derive(Debug, Clone)]
pub struct DeclaracionFuncion {
    pub nombre: String,
    pub parametros: Vec<Parametro>,
    pub tipo_retorno: Option<Tipo>,
    pub cuerpo: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct LlamadoFuncion {
    pub nombre: String,
    pub argumentos: Vec<Expresion>,
}

#[derive(Debug, Clone)]
pub struct Clase {
    pub nombre: String,
    pub cuerpo: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct DeclaracionClase {
    pub nombre: String,
    pub propiedades: Vec<Propiedad>,
    pub metodos: Vec<Metodo>,
}

#[derive(Debug, Clone)]
pub struct Propiedad {
    pub nombre: String,
    pub tipo: Option<Tipo>,
    pub publico: bool,
    pub valor_inicial: Option<Expresion>,
}

#[derive(Debug, Clone)]
pub struct Metodo {
    pub nombre: String,
    pub parametros: Vec<Parametro>,
    pub tipo_retorno: Option<Tipo>,
    pub cuerpo: Vec<Sentencia>,
    pub publico: bool,
}

#[derive(Debug, Clone)]
pub struct DeclaracionInterfaz {
    pub nombre: String,
    pub metodos: Vec<Metodo>,
}

#[derive(Debug, Clone)]
pub struct DeclaracionEnum {
    pub nombre: String,
    pub variantes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Expresion {
    LiteralEntero(i64),
    LiteralFloat(f64),
    LiteralCadena(String),
    LiteralBool(bool),
    Identificador(String),
    Binaria {
        izquierda: Box<Expresion>,
        operador: String,
        derecha: Box<Expresion>,
    },
    Unaria {
        operador: String,
        expresion: Box<Expresion>,
    },
    Agrupada(Box<Expresion>),
}
