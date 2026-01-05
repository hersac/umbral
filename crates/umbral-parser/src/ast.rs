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
    Return(Expresion),
    If(If),
    Switch(Switch),
    For(For),
    ForEach(ForEach),
    While(While),
    DoWhile(DoWhile),
    Expresion(Expresion),
    Funcion(DeclaracionFuncion),
    Clase(DeclaracionClase),
    Interfaz(DeclaracionInterfaz),
    Enum(DeclaracionEnum),
    LlamadoFuncion(LlamadoFuncion),
    Importacion(Importacion),
    TryCatch(TryCatch),
    Throw(Throw),
    Exportacion(String),
}

#[derive(Debug, Clone)]
pub struct DeclaracionVariable {
    pub nombre: String,
    pub tipo: Option<Tipo>,
    pub valor: Expresion,
    pub exportado: bool,
}

#[derive(Debug, Clone)]
pub struct DeclaracionConstante {
    pub nombre: String,
    pub tipo: Option<Tipo>,
    pub valor: Expresion,
    pub exportado: bool,
}

#[derive(Debug, Clone)]
pub struct Asignacion {
    pub objetivo: ObjetivoAsignacion,
    pub valor: Expresion,
}

#[derive(Debug, Clone)]
pub enum ObjetivoAsignacion {
    Variable(String),
    Propiedad {
        objeto: Box<Expresion>,
        propiedad: String,
    },
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
    pub exportado: bool,
    pub es_async: bool,
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
    pub extensiones: Vec<String>,
    pub implementaciones: Vec<String>,
    pub propiedades: Vec<Propiedad>,
    pub metodos: Vec<Metodo>,
    pub exportado: bool,
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
    pub es_async: bool,
}

#[derive(Debug, Clone)]
pub struct DeclaracionInterfaz {
    pub nombre: String,
    pub metodos: Vec<Metodo>,
    pub exportado: bool,
}

#[derive(Debug, Clone)]
pub struct VarianteEnum {
    pub nombre: String,
    pub valor: Option<Expresion>,
}

#[derive(Debug, Clone)]
pub struct DeclaracionEnum {
    pub nombre: String,
    pub variantes: Vec<VarianteEnum>,
    pub exportado: bool,
}

#[derive(Debug, Clone)]
pub enum Expresion {
    LiteralEntero(i64),
    LiteralFloat(f64),
    LiteralCadena(String),
    LiteralCadenaLiteral(String),
    LiteralBool(bool),
    LiteralNulo,
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
    Await(Box<Expresion>),
    Spread(Box<Expresion>),
    This,
    Agrupada(Box<Expresion>),
    Array(Vec<Expresion>),
    Objeto(Vec<(String, Expresion)>),
    Instanciacion {
        tipo: String,
        argumentos: Vec<Expresion>,
    },
    AccesoPropiedad {
        objeto: Box<Expresion>,
        propiedad: String,
    },
    AccesoIndice {
        objeto: Box<Expresion>,
        indice: Box<Expresion>,
    },
    LlamadoMetodo {
        objeto: Box<Expresion>,
        metodo: String,
        argumentos: Vec<Expresion>,
    },
    LlamadoFuncion {
        nombre: String,
        argumentos: Vec<Expresion>,
    },
}

#[derive(Debug, Clone)]
pub struct If {
    pub condicion: Expresion,
    pub bloque_entonces: Vec<Sentencia>,
    pub else_ifs: Vec<ElseIf>,
    pub bloque_else: Option<Vec<Sentencia>>,
}

#[derive(Debug, Clone)]
pub struct ElseIf {
    pub condicion: Expresion,
    pub bloque: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct Switch {
    pub expresion: Expresion,
    pub casos: Vec<Case>,
    pub default: Option<Vec<Sentencia>>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub valor: Expresion,
    pub bloque: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct For {
    pub inicializacion: Box<Sentencia>,
    pub condicion: Expresion,
    pub incremento: Expresion,
    pub bloque: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct ForEach {
    pub variable: String,
    pub tipo: Option<Tipo>,
    pub iterable: Expresion,
    pub bloque: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct While {
    pub condicion: Expresion,
    pub bloque: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct DoWhile {
    pub bloque: Vec<Sentencia>,
    pub condicion: Expresion,
}

#[derive(Debug, Clone)]
pub struct Importacion {
    pub items: Vec<ItemImportacion>,
    pub ruta: String,
}

#[derive(Debug, Clone)]
pub enum ItemImportacion {
    Todo(Option<String>),
    Nombre(String, Option<String>),
    ListaNombres(Vec<ItemImportacion>),
    Modulo(String),
}

#[derive(Debug, Clone)]
pub struct TryCatch {
    pub bloque_try: Vec<Sentencia>,
    pub bloque_catch: Option<Catch>,
    pub bloque_finally: Option<Vec<Sentencia>>,
}

#[derive(Debug, Clone)]
pub struct Catch {
    pub variable: String,
    pub tipo: Option<String>,
    pub bloque: Vec<Sentencia>,
}

#[derive(Debug, Clone)]
pub struct Throw {
    pub valor: Expresion,
}
