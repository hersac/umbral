pub mod clases;
pub mod entorno;
pub mod errores;
pub mod funciones;
pub mod interpretador;
pub mod valores;

use crate::runtime::interpretador::Interpretador;
use umbral_parser::ast::Programa;

pub struct Runtime {
    pub interpretador: Interpretador,
}

impl Runtime {
    pub fn nuevo() -> Self {
        Self {
            interpretador: Interpretador::nuevo(),
        }
    }

    pub fn ejecutar(&mut self, programa: Programa) {
        for sentencia in programa.sentencias {
            if self.interpretador.ejecutar_sentencia(sentencia).is_some() {
                break;
            }
        }
    }
}
