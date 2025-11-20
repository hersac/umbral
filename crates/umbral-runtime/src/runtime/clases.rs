use crate::runtime::valores::{Instancia, Valor};
use std::collections::HashMap;
use umbral_parser::ast::{DeclaracionClase, Metodo};

#[derive(Debug, Clone)]
pub struct Clase {
    pub nombre: String,
    pub propiedades: HashMap<String, Valor>,
    pub metodos: HashMap<String, Metodo>,
    pub constructor: Option<Metodo>,
}

impl Clase {
    pub fn nueva(nombre: &str) -> Self {
        Self {
            nombre: nombre.to_string(),
            propiedades: HashMap::new(),
            metodos: HashMap::new(),
            constructor: None,
        }
    }

    pub fn desde_declaracion(decl: &DeclaracionClase) -> Self {
        let mut clase = Self::nueva(&decl.nombre);

        clase.registrar_propiedades(&decl.propiedades);
        clase.registrar_metodos(&decl.metodos, &decl.nombre);

        clase
    }

    fn registrar_propiedades(&mut self, propiedades: &[umbral_parser::ast::Propiedad]) {
        for prop in propiedades {
            let valor_inicial = prop.valor_inicial.as_ref().map_or(Valor::Nulo, |_| Valor::Nulo);
            self.propiedades.insert(prop.nombre.clone(), valor_inicial);
        }
    }

    fn registrar_metodos(&mut self, metodos: &[Metodo], nombre_clase: &str) {
        for metodo in metodos {
            if metodo.nombre == nombre_clase {
                self.constructor = Some(metodo.clone());
                continue;
            }
            self.metodos.insert(metodo.nombre.clone(), metodo.clone());
        }
    }

    pub fn crear_instancia(&self) -> Instancia {
        Instancia {
            clase: self.nombre.clone(),
            propiedades: self.propiedades.clone(),
        }
    }

    pub fn obtener_metodo(&self, nombre: &str) -> Option<&Metodo> {
        self.metodos.get(nombre)
    }
}

pub struct GestorClases {
    pub clases: HashMap<String, Clase>,
}

impl GestorClases {
    pub fn nuevo() -> Self {
        Self {
            clases: HashMap::new(),
        }
    }

    pub fn registrar_clase(&mut self, clase: Clase) {
        self.clases.insert(clase.nombre.clone(), clase);
    }

    pub fn obtener_clase(&self, nombre: &str) -> Option<&Clase> {
        self.clases.get(nombre)
    }

    pub fn crear_instancia(&self, nombre: &str) -> Option<Instancia> {
        self.clases.get(nombre).map(|c| c.crear_instancia())
    }
}
