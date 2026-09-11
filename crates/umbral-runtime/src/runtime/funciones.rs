use crate::runtime::entorno::Entorno;
use crate::runtime::valores::{Funcion, ParametroRest, Valor};
use std::collections::HashMap;

pub fn dividir_argumentos(
    parametros_fijos: usize,
    tiene_rest: bool,
    argumentos: Vec<Valor>,
) -> (Vec<Valor>, Vec<Valor>) {
    if !tiene_rest {
        return (argumentos, Vec::new());
    }
    if argumentos.len() <= parametros_fijos {
        return (argumentos, Vec::new());
    }
    let resto = argumentos[parametros_fijos..].to_vec();
    let fijos = argumentos[..parametros_fijos].to_vec();
    (fijos, resto)
}

pub fn validar_elementos_rest(
    nombre_funcion: &str,
    rest: &ParametroRest,
    elementos: &[Valor],
) {
    let Some(tipo) = rest.tipo.as_deref() else {
        return;
    };
    elementos
        .iter()
        .enumerate()
        .filter(|(_, valor)| !valor.es_tipo_compatible(tipo))
        .for_each(|(indice, valor)| {
            eprintln!(
                "Error: Tipo incompatible en '{}' para '...{}->{}': argumento #{} es {} en lugar de '{}'.",
                nombre_funcion,
                rest.nombre,
                tipo,
                indice + 1,
                valor.nombre_tipo(),
                tipo
            );
        });
}

fn inyectar_captura(captura: Option<&HashMap<String, Valor>>, entorno: &mut Entorno) {
    let Some(mapa) = captura else {
        return;
    };
    mapa.iter().for_each(|(nombre, valor)| {
        entorno.definir_variable(nombre.clone(), valor.clone());
    });
}

fn vincular_fijos(funcion: &Funcion, fijos: &[Valor], entorno: &mut Entorno) {
    funcion
        .parametros
        .iter()
        .enumerate()
        .for_each(|(indice, param)| {
            let valor = fijos.get(indice).cloned().unwrap_or(Valor::Nulo);
            entorno.definir_variable(param.clone(), valor);
        });
}

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
        Self::preparar_llamada(funcion, argumentos, interprete).await
    }

    async fn preparar_llamada(
        funcion: &Funcion,
        argumentos: Vec<Valor>,
        interprete: &mut crate::runtime::interpretador::Interpretador,
    ) -> Valor {
        let anterior = std::mem::replace(&mut interprete.entorno_actual, Entorno::nuevo(None));
        interprete.entorno_actual = Entorno::nuevo(Some(anterior));
        inyectar_captura(funcion.entorno_capturado.as_ref(), &mut interprete.entorno_actual);
        Self::vincular_llamada(funcion, argumentos, interprete);
        let salida = Self::correr_cuerpo(funcion, interprete).await;
        Self::restaurar_llamada(interprete);
        salida
    }

    fn vincular_llamada(
        funcion: &Funcion,
        argumentos: Vec<Valor>,
        interprete: &mut crate::runtime::interpretador::Interpretador,
    ) {
        let (fijos, resto) = dividir_argumentos(
            funcion.parametros.len(),
            funcion.parametro_rest.is_some(),
            argumentos,
        );
        vincular_fijos(funcion, &fijos, &mut interprete.entorno_actual);
        Self::vincular_resto(funcion, resto, interprete);
    }

    fn vincular_resto(
        funcion: &Funcion,
        resto: Vec<Valor>,
        interprete: &mut crate::runtime::interpretador::Interpretador,
    ) {
        let Some(param) = &funcion.parametro_rest else {
            return;
        };
        validar_elementos_rest(&funcion.nombre, param, &resto);
        interprete
            .entorno_actual
            .definir_variable(param.nombre.clone(), Valor::Lista(resto));
    }

    async fn correr_cuerpo(
        funcion: &Funcion,
        interprete: &mut crate::runtime::interpretador::Interpretador,
    ) -> Valor {
        for sentencia in &funcion.cuerpo {
            if let Some(valor) = interprete.ejecutar_sentencia(sentencia.clone()).await {
                return valor;
            }
        }
        Valor::Nulo
    }

    fn restaurar_llamada(interprete: &mut crate::runtime::interpretador::Interpretador) {
        let Some(padre) = interprete.entorno_actual.parent.take() else {
            return;
        };
        interprete.entorno_actual = *padre;
    }
}
