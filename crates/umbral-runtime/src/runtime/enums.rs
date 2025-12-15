use std::collections::HashMap;
use umbral_parser::ast::{DeclaracionEnum, VarianteEnum};

#[derive(Debug, Clone)]
pub struct Enum {
    pub nombre: String,
    pub variantes: Vec<VarianteEnum>,
}

impl Enum {
    pub fn desde_declaracion(decl: &DeclaracionEnum) -> Self {
        Self {
            nombre: decl.nombre.clone(),
            variantes: decl.variantes.clone(),
        }
    }
}

pub struct GestorEnums {
    enums: HashMap<String, Enum>,
}

impl GestorEnums {
    pub fn nuevo() -> Self {
        Self {
            enums: HashMap::new(),
        }
    }

    pub fn registrar(&mut self, enum_obj: Enum) {
        self.enums.insert(enum_obj.nombre.clone(), enum_obj);
    }

    pub fn obtener(&self, nombre: &str) -> Option<&Enum> {
        self.enums.get(nombre)
    }
}
