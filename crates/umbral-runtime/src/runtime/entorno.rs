use crate::runtime::valores::Valor;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Entorno {
    pub variables: HashMap<String, Valor>,
    pub constantes: HashMap<String, Valor>,
    pub parent: Option<Box<Entorno>>,
}

impl Entorno {
    pub fn nuevo(parent: Option<Entorno>) -> Self {
        Self {
            variables: HashMap::new(),
            constantes: HashMap::new(),
            parent: parent.map(Box::new),
        }
    }

    pub fn definir_variable(&mut self, nombre: String, valor: Valor) {
        self.variables.insert(nombre, valor);
    }

    pub fn definir_constante(&mut self, nombre: String, valor: Valor) {
        self.constantes.insert(nombre, valor);
    }
    
    pub fn asignar(&mut self, nombre: &str, valor: Valor) -> bool {
        if self.variables.contains_key(nombre) {
            self.variables.insert(nombre.to_string(), valor);
            true
        } else if let Some(parent) = &mut self.parent {
            parent.asignar(nombre, valor)
        } else {
            false
        }
    }

    pub fn obtener(&self, nombre: &str) -> Option<Valor> {
        if let Some(v) = self.variables.get(nombre) {
            Some(v.clone())
        } else if let Some(c) = self.constantes.get(nombre) {
            Some(c.clone())
        } else if let Some(parent) = &self.parent {
            parent.obtener(nombre)
        } else {
            None
        }
    }
    
    pub fn existe(&self, nombre: &str) -> bool {
        self.variables.contains_key(nombre) 
            || self.constantes.contains_key(nombre)
            || self.parent.as_ref().map_or(false, |p| p.existe(nombre))
    }
}
