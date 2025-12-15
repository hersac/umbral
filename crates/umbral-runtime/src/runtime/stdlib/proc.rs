use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::process::Command;

fn registrar_funcion(mapa: &mut HashMap<String, Valor>, nombre: &str, funcion: fn(Vec<Valor>) -> Valor) {
    mapa.insert(
        nombre.to_string(),
        Valor::FuncionNativa(nombre.to_string(), funcion),
    );
}

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    registrar_funcion(&mut mapa, "exec", ejecutar);

    Valor::Diccionario(mapa)
}

fn obtener_comando(argumentos: &[Valor]) -> Option<String> {
    match argumentos.get(0) {
        Some(Valor::Texto(comando)) => Some(comando.clone()),
        _ => None,
    }
}

fn extraer_argumentos_texto(argumentos: &[Valor]) -> Vec<String> {
    argumentos
        .iter()
        .skip(1)
        .filter_map(|valor| {
            if let Valor::Texto(texto) = valor {
                Some(texto.clone())
            } else {
                None
            }
        })
        .collect()
}

fn crear_diccionario_salida(salida: &std::process::Output) -> Valor {
    let stdout = String::from_utf8_lossy(&salida.stdout).to_string();
    let stderr = String::from_utf8_lossy(&salida.stderr).to_string();
    let codigo = salida.status.code().unwrap_or(-1) as i64;

    let mut resultado = HashMap::new();
    resultado.insert("stdout".to_string(), Valor::Texto(stdout));
    resultado.insert("stderr".to_string(), Valor::Texto(stderr));
    resultado.insert("code".to_string(), Valor::Entero(codigo));

    Valor::Diccionario(resultado)
}

fn ejecutar_comando(comando: &str, argumentos_comando: &[String]) -> Valor {
    match Command::new(comando).args(argumentos_comando).output() {
        Ok(salida) => crear_diccionario_salida(&salida),
        Err(_) => Valor::Nulo,
    }
}

fn ejecutar(argumentos: Vec<Valor>) -> Valor {
    let comando = match obtener_comando(&argumentos) {
        Some(cmd) => cmd,
        None => return Valor::Nulo,
    };

    let argumentos_comando = extraer_argumentos_texto(&argumentos);
    ejecutar_comando(&comando, &argumentos_comando)
}
