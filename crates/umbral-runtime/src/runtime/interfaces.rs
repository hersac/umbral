use std::collections::HashMap;
use umbral_parser::ast::{DeclaracionInterfaz, Metodo, Propiedad};

#[derive(Debug, Clone)]
pub struct Interfaz {
    pub nombre: String,
    pub parametros_tipo: Vec<String>,
    pub metodos: HashMap<String, Metodo>,
    pub propiedades: HashMap<String, Propiedad>,
}

impl Interfaz {
    pub fn desde_declaracion(decl: &DeclaracionInterfaz) -> Self {
        let mut metodos = HashMap::new();
        for metodo in &decl.metodos {
            metodos.insert(metodo.nombre.clone(), metodo.clone());
        }
        let mut propiedades = HashMap::new();
        for prop in &decl.propiedades {
            propiedades.insert(prop.nombre.clone(), prop.clone());
        }
        Self {
            nombre: decl.nombre.clone(),
            parametros_tipo: decl.parametros_tipo.clone(),
            metodos,
            propiedades,
        }
    }
}

#[derive(Clone)]
pub struct GestorInterfaces {
    interfaces: HashMap<String, Interfaz>,
}

impl GestorInterfaces {
    pub fn nuevo() -> Self {
        Self {
            interfaces: HashMap::new(),
        }
    }

    pub fn registrar(&mut self, interfaz: Interfaz) {
        self.interfaces.insert(interfaz.nombre.clone(), interfaz);
    }

    pub fn obtener(&self, nombre: &str) -> Option<&Interfaz> {
        self.interfaces.get(nombre)
    }
}
