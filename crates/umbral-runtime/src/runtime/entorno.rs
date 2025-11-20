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
            return true;
        }

        self.parent
            .as_mut()
            .map_or(false, |parent| parent.asignar(nombre, valor))
    }

    pub fn obtener(&self, nombre: &str) -> Option<Valor> {
        self.variables
            .get(nombre)
            .or_else(|| self.constantes.get(nombre))
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.obtener(nombre)))
    }
    
    pub fn existe(&self, nombre: &str) -> bool {
        self.variables.contains_key(nombre) 
            || self.constantes.contains_key(nombre)
            || self.parent.as_ref().map_or(false, |p| p.existe(nombre))
    }
}
