use crate::runtime::valores::Valor;
use std::collections::HashMap;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "len".to_string(),
        Valor::FuncionNativa("len".to_string(), len),
    );
    mapa.insert(
        "push".to_string(),
        Valor::FuncionNativa("push".to_string(), push),
    );
    mapa.insert(
        "pop".to_string(),
        Valor::FuncionNativa("pop".to_string(), pop),
    );
    mapa.insert(
        "keys".to_string(),
        Valor::FuncionNativa("keys".to_string(), keys),
    );
    mapa.insert(
        "values".to_string(),
        Valor::FuncionNativa("values".to_string(), values),
    );
    mapa.insert(
        "sort".to_string(),
        Valor::FuncionNativa("sort".to_string(), sort),
    );
    mapa.insert(
        "reverse".to_string(),
        Valor::FuncionNativa("reverse".to_string(), reverse),
    );

    Valor::Diccionario(mapa)
}

fn len(args: Vec<Valor>) -> Valor {
    match args.get(0) {
        Some(Valor::Lista(l)) => Valor::Entero(l.len() as i64),
        Some(Valor::Texto(s)) => Valor::Entero(s.len() as i64),
        Some(Valor::Diccionario(d)) => Valor::Entero(d.len() as i64),
        _ => Valor::Entero(0),
    }
}

fn push(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Lista(mut l)) = args.get(0).cloned() {
        if let Some(val) = args.get(1) {
            l.push(val.clone());
            Valor::Lista(l)
        } else {
            Valor::Lista(l)
        }
    } else {
        Valor::Nulo
    }
}

fn pop(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Lista(mut l)) = args.get(0).cloned() {
        l.pop();
        Valor::Lista(l)
    } else {
        Valor::Nulo
    }
}

fn keys(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Diccionario(d)) = args.get(0) {
        let keys_list: Vec<Valor> = d.keys().map(|k| Valor::Texto(k.clone())).collect();
        Valor::Lista(keys_list)
    } else {
        Valor::Lista(vec![])
    }
}

fn values(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Diccionario(d)) = args.get(0) {
        let values_list: Vec<Valor> = d.values().cloned().collect();
        Valor::Lista(values_list)
    } else {
        Valor::Lista(vec![])
    }
}

fn sort(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Lista(mut l)) = args.get(0).cloned() {
        l.sort_by(|a, b| match (a, b) {
            (Valor::Entero(x), Valor::Entero(y)) => x.cmp(y),
            (Valor::Flotante(x), Valor::Flotante(y)) => {
                x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Valor::Texto(x), Valor::Texto(y)) => x.cmp(y),
            _ => std::cmp::Ordering::Equal,
        });
        Valor::Lista(l)
    } else {
        Valor::Nulo
    }
}

fn reverse(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Lista(mut l)) = args.get(0).cloned() {
        l.reverse();
        Valor::Lista(l)
    } else {
        Valor::Nulo
    }
}
