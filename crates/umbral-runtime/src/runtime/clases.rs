use crate::runtime::valores::{Instancia, Valor};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use umbral_parser::ast::{DeclaracionClase, Metodo};

#[derive(Debug, Clone)]
pub struct Clase {
    pub nombre: String,
    pub parametros_tipo: Vec<String>,
    pub propiedades: HashMap<String, Valor>,
    pub tipos_propiedades: HashMap<String, String>,
    pub metodos: HashMap<String, Metodo>,
    pub constructor: Option<Metodo>,
    pub doc: Option<umbral_parser::ast::UmDoc>,
    /// Captura de los globales del módulo donde se definió la clase,
    /// para que sus métodos puedan verlos aunque la instancia se use
    /// desde otro módulo.
    pub entorno_capturado: Option<HashMap<String, Valor>>,
}

impl Clase {
    pub fn nueva(nombre: &str) -> Self {
        Self {
            nombre: nombre.to_string(),
            parametros_tipo: Vec::new(),
            propiedades: HashMap::new(),
            tipos_propiedades: HashMap::new(),
            metodos: HashMap::new(),
            constructor: None,
            doc: None,
            entorno_capturado: None,
        }
    }

    pub fn desde_declaracion(decl: &DeclaracionClase) -> Self {
        let mut clase = Self::nueva(&decl.nombre);
        clase.doc = decl.doc.clone();
        clase.parametros_tipo = decl.parametros_tipo.clone();
        clase.registrar_propiedades(&decl.propiedades);
        clase.registrar_metodos(&decl.metodos, &decl.nombre);
        clase
    }

    pub fn texto_ayuda(&self) -> String {
        let base = match &self.doc {
            Some(d) => d.formatear(&self.nombre, ""),
            None => format!("{} (sin documentación umdocs)\n", self.nombre),
        };
        let lista = self.lista_metodos();
        if lista.is_empty() {
            return base;
        }
        format!("{}{}", base, lista)
    }

    fn lista_metodos(&self) -> String {
        if self.metodos.is_empty() {
            return String::new();
        }
        let nombres = self.nombres_ordenados();
        let cuerpo: String = nombres
            .iter()
            .map(|n| format!("    - {}\n", n))
            .collect();
        format!("  Métodos:\n{}", cuerpo)
    }

    fn nombres_ordenados(&self) -> Vec<&String> {
        let mut nombres: Vec<&String> = self.metodos.keys().collect();
        nombres.sort();
        nombres
    }

    fn registrar_propiedades(&mut self, propiedades: &[umbral_parser::ast::Propiedad]) {
        let pares = propiedades.iter().map(|p| (p.nombre.clone(), Valor::Nulo));
        self.propiedades.extend(pares);
        let tipos = propiedades.iter().filter_map(|p| {
            p.tipo
                .as_ref()
                .map(|t| (p.nombre.clone(), t.nombre.clone()))
        });
        self.tipos_propiedades.extend(tipos);
    }

    fn registrar_metodos(&mut self, metodos: &[Metodo], nombre_clase: &str) {
        let filtrados = metodos.iter().filter(|m| m.nombre != nombre_clase);
        let pares = filtrados.map(|m| (m.nombre.clone(), m.clone()));
        self.metodos.extend(pares);
        let ctor = metodos.iter().find(|m| m.nombre == nombre_clase).cloned();
        self.constructor = ctor;
    }

    pub fn crear_instancia(&self) -> Instancia {
        Instancia {
            clase: self.nombre.clone(),
            propiedades: Arc::new(Mutex::new(self.propiedades.clone())),
        }
    }

    pub fn obtener_metodo(&self, nombre: &str) -> Option<&Metodo> {
        self.metodos.get(nombre)
    }
}

#[derive(Clone)]
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
