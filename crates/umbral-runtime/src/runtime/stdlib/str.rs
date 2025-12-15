use crate::runtime::valores::Valor;
use std::collections::HashMap;

fn registrar_funcion(mapa: &mut HashMap<String, Valor>, nombre: &str, funcion: fn(Vec<Valor>) -> Valor) {
    mapa.insert(
        nombre.to_string(),
        Valor::FuncionNativa(nombre.to_string(), funcion),
    );
}

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    registrar_funcion(&mut mapa, "trim", recortar);
    registrar_funcion(&mut mapa, "split", dividir);
    registrar_funcion(&mut mapa, "replace", reemplazar);
    registrar_funcion(&mut mapa, "to_upper", mayusculas);
    registrar_funcion(&mut mapa, "to_lower", minusculas);
    registrar_funcion(&mut mapa, "search", buscar);
    registrar_funcion(&mut mapa, "contains", contiene);
    registrar_funcion(&mut mapa, "starts_with", inicia_con);
    registrar_funcion(&mut mapa, "ends_with", termina_con);

    Valor::Diccionario(mapa)
}

fn obtener_texto(argumentos: &[Valor]) -> Option<String> {
    match argumentos.get(0) {
        Some(Valor::Texto(texto)) => Some(texto.clone()),
        _ => None,
    }
}

fn recortar(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto(&argumentos) {
        Some(texto) => Valor::Texto(texto.trim().to_string()),
        None => Valor::Nulo,
    }
}

fn obtener_separador(argumentos: &[Valor]) -> &str {
    match argumentos.get(1) {
        Some(Valor::Texto(separador)) => separador.as_str(),
        _ => " ",
    }
}

fn dividir(argumentos: Vec<Valor>) -> Valor {
    let texto = match obtener_texto(&argumentos) {
        Some(t) => t,
        None => return Valor::Nulo,
    };

    let separador = obtener_separador(&argumentos);
    let partes: Vec<Valor> = texto.split(separador).map(|p| Valor::Texto(p.to_string())).collect();

    Valor::Lista(partes)
}

fn obtener_patron_reemplazo(argumentos: &[Valor]) -> Option<(String, String)> {
    let antiguo = match argumentos.get(1) {
        Some(Valor::Texto(a)) => a.clone(),
        _ => return None,
    };

    let nuevo = match argumentos.get(2) {
        Some(Valor::Texto(n)) => n.clone(),
        _ => return None,
    };

    Some((antiguo, nuevo))
}

fn reemplazar(argumentos: Vec<Valor>) -> Valor {
    let texto = match obtener_texto(&argumentos) {
        Some(t) => t,
        None => return Valor::Nulo,
    };

    match obtener_patron_reemplazo(&argumentos) {
        Some((antiguo, nuevo)) => Valor::Texto(texto.replace(&antiguo, &nuevo)),
        None => Valor::Texto(texto),
    }
}

fn mayusculas(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto(&argumentos) {
        Some(texto) => Valor::Texto(texto.to_uppercase()),
        None => Valor::Nulo,
    }
}

fn minusculas(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto(&argumentos) {
        Some(texto) => Valor::Texto(texto.to_lowercase()),
        None => Valor::Nulo,
    }
}

fn obtener_texto_y_patron(argumentos: &[Valor]) -> Option<(String, String)> {
    let texto = match argumentos.get(0) {
        Some(Valor::Texto(t)) => t.clone(),
        _ => return None,
    };

    let patron = match argumentos.get(1) {
        Some(Valor::Texto(p)) => p.clone(),
        _ => return None,
    };

    Some((texto, patron))
}

fn buscar(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto_y_patron(&argumentos) {
        Some((texto, patron)) => {
            match texto.find(&patron) {
                Some(indice) => Valor::Entero(indice as i64),
                None => Valor::Entero(-1),
            }
        }
        None => Valor::Entero(-1),
    }
}

fn contiene(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto_y_patron(&argumentos) {
        Some((texto, patron)) => Valor::Booleano(texto.contains(&patron)),
        None => Valor::Booleano(false),
    }
}

fn inicia_con(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto_y_patron(&argumentos) {
        Some((texto, patron)) => Valor::Booleano(texto.starts_with(&patron)),
        None => Valor::Booleano(false),
    }
}

fn termina_con(argumentos: Vec<Valor>) -> Valor {
    match obtener_texto_y_patron(&argumentos) {
        Some((texto, patron)) => Valor::Booleano(texto.ends_with(&patron)),
        None => Valor::Booleano(false),
    }
}
