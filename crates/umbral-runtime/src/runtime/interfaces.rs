use std::collections::HashMap;
use umbral_parser::ast::{DeclaracionInterfaz, Metodo};

#[derive(Debug, Clone)]
pub struct Interfaz {
    pub nombre: String,
    pub metodos: HashMap<String, Metodo>,
}

impl Interfaz {
    pub fn desde_declaracion(decl: &DeclaracionInterfaz) -> Self {
        let mut metodos = HashMap::new();
        for metodo in &decl.metodos {
            metodos.insert(metodo.nombre.clone(), metodo.clone());
        }
        Self {
            nombre: decl.nombre.clone(),
            metodos,
        }
    }
}

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
