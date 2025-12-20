use crate::runtime::clases::Clase;
use crate::runtime::valores::Valor;
use std::collections::HashMap;
use umbral_parser::ast::{Asignacion, Expresion, Metodo, ObjetivoAsignacion, Parametro, Sentencia};

pub fn crear_clase_error() -> Clase {
    let mut propiedades = HashMap::new();
    propiedades.insert("mensaje".to_string(), Valor::Texto("".to_string()));
    propiedades.insert("data".to_string(), Valor::Nulo);

    // Constructor: Error(msg) { th.mensaje = msg; }
    let constructor = Metodo {
        nombre: "Error".to_string(),
        parametros: vec![Parametro {
            nombre: "msg".to_string(),
            tipo: None,
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
    };

    Clase {
        nombre: "Error".to_string(),
        propiedades,
        metodos: HashMap::new(),
        constructor: Some(constructor),
    }
}
