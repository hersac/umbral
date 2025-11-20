use crate::runtime::entorno::Entorno;
use crate::runtime::valores::{Funcion, Valor};

pub struct GestorFunciones {
    // Aquí se pueden almacenar funciones adicionales
}

impl GestorFunciones {
    pub fn nuevo() -> Self {
        Self {}
    }

    pub fn ejecutar_funcion(
        funcion: &Funcion,
        argumentos: Vec<Valor>,
        interprete: &mut crate::runtime::interpretador::Interpretador,
    ) -> Valor {
        let parent = interprete.entorno_actual.clone();
        let entorno_anterior =
            std::mem::replace(&mut interprete.entorno_actual, Entorno::nuevo(Some(parent)));

        for (i, param) in funcion.parametros.iter().enumerate() {
            let valor = argumentos.get(i).cloned().unwrap_or(Valor::Nulo);
            interprete
                .entorno_actual
                .definir_variable(param.clone(), valor);
        }

        let mut resultado = Valor::Nulo;
        for sentencia in &funcion.cuerpo {
            if let Some(valor) = interprete.ejecutar_sentencia(sentencia.clone()) {
                resultado = valor;
                break;
            }
        }

        interprete.entorno_actual = entorno_anterior;

        resultado
    }
}
