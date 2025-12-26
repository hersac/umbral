use crate::runtime::entorno::Entorno;
use crate::runtime::valores::{Funcion, Valor};

#[derive(Clone)]
pub struct GestorFunciones {}

impl GestorFunciones {
    pub fn nuevo() -> Self {
        Self {}
    }

    pub async fn ejecutar_funcion(
        funcion: &Funcion,
        argumentos: Vec<Valor>,
        interprete: &mut crate::runtime::interpretador::Interpretador,
    ) -> Valor {
        let anterior = std::mem::replace(&mut interprete.entorno_actual, Entorno::nuevo(None));
        interprete.entorno_actual = Entorno::nuevo(Some(anterior));

        for (i, param) in funcion.parametros.iter().enumerate() {
            let valor = argumentos.get(i).cloned().unwrap_or(Valor::Nulo);
            interprete
                .entorno_actual
                .definir_variable(param.clone(), valor);
        }

        let mut resultado = Valor::Nulo;
        for sentencia in &funcion.cuerpo {
            if let Some(valor) = interprete.ejecutar_sentencia(sentencia.clone()).await {
                resultado = valor;
                break;
            }
        }

        if let Some(parent) = interprete.entorno_actual.parent.take() {
            interprete.entorno_actual = *parent;
        }

        resultado
    }
}
