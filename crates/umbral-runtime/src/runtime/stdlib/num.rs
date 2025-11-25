use crate::runtime::valores::Valor;
use rand::Rng;
use std::collections::HashMap;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "parse_int".to_string(),
        Valor::FuncionNativa("parse_int".to_string(), parse_int),
    );
    mapa.insert(
        "parse_float".to_string(),
        Valor::FuncionNativa("parse_float".to_string(), parse_float),
    );
    mapa.insert(
        "to_string".to_string(),
        Valor::FuncionNativa("to_string".to_string(), to_string),
    );
    mapa.insert(
        "random".to_string(),
        Valor::FuncionNativa("random".to_string(), random),
    );
    mapa.insert(
        "abs".to_string(),
        Valor::FuncionNativa("abs".to_string(), abs),
    );
    mapa.insert(
        "clamp".to_string(),
        Valor::FuncionNativa("clamp".to_string(), clamp),
    );
    mapa.insert(
        "min".to_string(),
        Valor::FuncionNativa("min".to_string(), min),
    );
    mapa.insert(
        "max".to_string(),
        Valor::FuncionNativa("max".to_string(), max),
    );

    Valor::Diccionario(mapa)
}

fn parse_int(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        match s.parse::<i64>() {
            Ok(n) => Valor::Entero(n),
            Err(_) => Valor::Nulo,
        }
    } else {
        Valor::Nulo
    }
}

fn parse_float(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        match s.parse::<f64>() {
            Ok(n) => Valor::Flotante(n),
            Err(_) => Valor::Nulo,
        }
    } else {
        Valor::Nulo
    }
}

fn to_string(args: Vec<Valor>) -> Valor {
    if let Some(val) = args.get(0) {
        Valor::Texto(format!("{}", val))
    } else {
        Valor::Nulo
    }
}

fn random(_args: Vec<Valor>) -> Valor {
    let mut rng = rand::thread_rng();
    Valor::Flotante(rng.gen::<f64>())
}

fn abs(args: Vec<Valor>) -> Valor {
    match args.get(0) {
        Some(Valor::Entero(n)) => Valor::Entero(n.abs()),
        Some(Valor::Flotante(n)) => Valor::Flotante(n.abs()),
        _ => Valor::Nulo,
    }
}

fn clamp(args: Vec<Valor>) -> Valor {
    let val = args.get(0);
    let min_val = args.get(1);
    let max_val = args.get(2);

    match (val, min_val, max_val) {
        (Some(Valor::Entero(v)), Some(Valor::Entero(min)), Some(Valor::Entero(max))) => {
            Valor::Entero((*v).clamp(*min, *max))
        }
        (Some(Valor::Flotante(v)), Some(Valor::Flotante(min)), Some(Valor::Flotante(max))) => {
            Valor::Flotante(v.clamp(*min, *max))
        }
        _ => Valor::Nulo,
    }
}

fn min(args: Vec<Valor>) -> Valor {
    match (args.get(0), args.get(1)) {
        (Some(Valor::Entero(a)), Some(Valor::Entero(b))) => Valor::Entero(*a.min(b)),
        (Some(Valor::Flotante(a)), Some(Valor::Flotante(b))) => Valor::Flotante(a.min(*b)),
        _ => Valor::Nulo,
    }
}

fn max(args: Vec<Valor>) -> Valor {
    match (args.get(0), args.get(1)) {
        (Some(Valor::Entero(a)), Some(Valor::Entero(b))) => Valor::Entero(*a.max(b)),
        (Some(Valor::Flotante(a)), Some(Valor::Flotante(b))) => Valor::Flotante(a.max(*b)),
        _ => Valor::Nulo,
    }
}
