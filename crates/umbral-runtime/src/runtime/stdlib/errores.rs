use crate::runtime::clases::Clase;
use crate::runtime::valores::Valor;
use std::collections::HashMap;
use umbral_parser::ast::{Asignacion, Expresion, Metodo, ObjetivoAsignacion, Parametro, Sentencia};

pub fn crear_clase_error() -> Clase {
    let propiedades: HashMap<String, Valor> = [
        ("mensaje", Valor::Texto(String::new())),
        ("data", Valor::Nulo),
    ]
    .into_iter()
    .map(|(clave, valor)| (clave.to_string(), valor))
    .collect();

    let constructor = Metodo {
        nombre: "Error".to_string(),
        parametros_tipo: Vec::new(),
        parametros: vec![Parametro {
            nombre: "msg".to_string(),
            tipo: None,
            es_rest: false,
        }],
        tipo_retorno: None,
        cuerpo: vec![Sentencia::Asignacion(Asignacion {
            objetivo: ObjetivoAsignacion::Propiedad {
                objeto: Box::new(Expresion::This),
                propiedad: "mensaje".to_string(),
            },
            valor: Expresion::Identificador("msg".to_string()),
        })],
        publico: true,
        es_async: false,
        doc: None,
    };

    Clase {
        nombre: "Error".to_string(),
        parametros_tipo: Vec::new(),
        propiedades,
        tipos_propiedades: HashMap::new(),
        metodos: HashMap::new(),
        constructor: Some(constructor),
        doc: None,
        entorno_capturado: None,
    }
}
