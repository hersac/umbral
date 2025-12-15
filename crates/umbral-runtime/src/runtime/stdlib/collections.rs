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

    registrar_funcion(&mut mapa, "len", len);
    registrar_funcion(&mut mapa, "push", push);
    registrar_funcion(&mut mapa, "pop", pop);
    registrar_funcion(&mut mapa, "keys", keys);
    registrar_funcion(&mut mapa, "values", values);
    registrar_funcion(&mut mapa, "sort", sort);
    registrar_funcion(&mut mapa, "reverse", reverse);

    Valor::Diccionario(mapa)
}

fn obtener_longitud_lista(lista: &[Valor]) -> Valor {
    Valor::Entero(lista.len() as i64)
}

fn obtener_longitud_texto(texto: &str) -> Valor {
    Valor::Entero(texto.len() as i64)
}

fn obtener_longitud_diccionario(diccionario: &HashMap<String, Valor>) -> Valor {
    Valor::Entero(diccionario.len() as i64)
}

fn len(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Lista(lista)) => obtener_longitud_lista(lista),
        Some(Valor::Texto(texto)) => obtener_longitud_texto(texto),
        Some(Valor::Diccionario(diccionario)) => obtener_longitud_diccionario(diccionario),
        _ => Valor::Entero(0),
    }
}

fn agregar_elemento_lista(mut lista: Vec<Valor>, elemento: Valor) -> Valor {
    lista.push(elemento);
    Valor::Lista(lista)
}

fn push(argumentos: Vec<Valor>) -> Valor {
    let lista = match argumentos.get(0).cloned() {
        Some(Valor::Lista(l)) => l,
        _ => return Valor::Nulo,
    };

    let elemento = match argumentos.get(1) {
        Some(valor) => valor.clone(),
        None => return Valor::Lista(lista),
    };

    agregar_elemento_lista(lista, elemento)
}

fn pop(argumentos: Vec<Valor>) -> Valor {
    let mut lista = match argumentos.get(0).cloned() {
        Some(Valor::Lista(l)) => l,
        _ => return Valor::Nulo,
    };

    lista.pop();
    Valor::Lista(lista)
}

fn extraer_claves(diccionario: &HashMap<String, Valor>) -> Valor {
    let claves: Vec<Valor> = diccionario.keys().map(|c| Valor::Texto(c.clone())).collect();
    Valor::Lista(claves)
}

fn keys(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Diccionario(diccionario)) => extraer_claves(diccionario),
        _ => Valor::Lista(vec![]),
    }
}

fn extraer_valores(diccionario: &HashMap<String, Valor>) -> Valor {
    let valores: Vec<Valor> = diccionario.values().cloned().collect();
    Valor::Lista(valores)
}

fn values(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Diccionario(diccionario)) => extraer_valores(diccionario),
        _ => Valor::Lista(vec![]),
    }
}

fn comparar_valores(a: &Valor, b: &Valor) -> std::cmp::Ordering {
    match (a, b) {
        (Valor::Entero(x), Valor::Entero(y)) => x.cmp(y),
        (Valor::Flotante(x), Valor::Flotante(y)) => {
            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Valor::Texto(x), Valor::Texto(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

fn sort(argumentos: Vec<Valor>) -> Valor {
    let mut lista = match argumentos.get(0).cloned() {
        Some(Valor::Lista(l)) => l,
        _ => return Valor::Nulo,
    };

    lista.sort_by(comparar_valores);
    Valor::Lista(lista)
}

fn reverse(argumentos: Vec<Valor>) -> Valor {
    let mut lista = match argumentos.get(0).cloned() {
        Some(Valor::Lista(l)) => l,
        _ => return Valor::Nulo,
    };

    lista.reverse();
    Valor::Lista(lista)
}
