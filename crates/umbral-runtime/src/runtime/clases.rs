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
        
        // Agregar propiedades con valores iniciales
        for prop in &decl.propiedades {
            let valor_inicial = prop.valor_inicial.as_ref()
                .map(|_| Valor::Nulo) // Se evaluará después
                .unwrap_or(Valor::Nulo);
            clase.propiedades.insert(prop.nombre.clone(), valor_inicial);
        }
        
        // Agregar métodos
        for metodo in &decl.metodos {
            if metodo.nombre == decl.nombre {
                // Es el constructor
                clase.constructor = Some(metodo.clone());
            } else {
                clase.metodos.insert(metodo.nombre.clone(), metodo.clone());
            }
        }
        
        clase
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
