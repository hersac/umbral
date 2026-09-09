use crate::runtime::entorno::Entorno;
use crate::runtime::valores::{Funcion, ParametroRest, Valor};

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
    for (i, valor) in elementos.iter().enumerate() {
        if !valor.es_tipo_compatible(tipo) {
            eprintln!(
                "Error: Tipo incompatible en '{}' para '...{}->{}': argumento #{} es {} en lugar de '{}'.",
                nombre_funcion,
                rest.nombre,
                tipo,
                i + 1,
                valor.nombre_tipo(),
                tipo
            );
        }
    }
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
        let anterior = std::mem::replace(&mut interprete.entorno_actual, Entorno::nuevo(None));
        interprete.entorno_actual = Entorno::nuevo(Some(anterior));

        let (fijos, resto) = dividir_argumentos(
            funcion.parametros.len(),
            funcion.parametro_rest.is_some(),
            argumentos,
        );

        for (i, param) in funcion.parametros.iter().enumerate() {
            let valor = fijos.get(i).cloned().unwrap_or(Valor::Nulo);
            interprete
                .entorno_actual
                .definir_variable(param.clone(), valor);
        }

        if let Some(rest) = &funcion.parametro_rest {
            validar_elementos_rest(&funcion.nombre, rest, &resto);
            interprete
                .entorno_actual
                .definir_variable(rest.nombre.clone(), Valor::Lista(resto));
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
