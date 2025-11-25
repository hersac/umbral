use crate::runtime::valores::Valor;
use std::collections::HashMap;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "trim".to_string(),
        Valor::FuncionNativa("trim".to_string(), trim),
    );
    mapa.insert(
        "split".to_string(),
        Valor::FuncionNativa("split".to_string(), split),
    );
    mapa.insert(
        "replace".to_string(),
        Valor::FuncionNativa("replace".to_string(), replace),
    );
    mapa.insert(
        "to_upper".to_string(),
        Valor::FuncionNativa("to_upper".to_string(), to_upper),
    );
    mapa.insert(
        "to_lower".to_string(),
        Valor::FuncionNativa("to_lower".to_string(), to_lower),
    );
    mapa.insert(
        "search".to_string(),
        Valor::FuncionNativa("search".to_string(), search),
    );
    mapa.insert(
        "contains".to_string(),
        Valor::FuncionNativa("contains".to_string(), contains),
    );
    mapa.insert(
        "starts_with".to_string(),
        Valor::FuncionNativa("starts_with".to_string(), starts_with),
    );
    mapa.insert(
        "ends_with".to_string(),
        Valor::FuncionNativa("ends_with".to_string(), ends_with),
    );

    Valor::Diccionario(mapa)
}

fn trim(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        Valor::Texto(s.trim().to_string())
    } else {
        Valor::Nulo
    }
}

fn split(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        let sep = if let Some(Valor::Texto(sep)) = args.get(1) {
            sep.as_str()
        } else {
            " "
        };

        let partes: Vec<Valor> = s.split(sep).map(|p| Valor::Texto(p.to_string())).collect();

        Valor::Lista(partes)
    } else {
        Valor::Nulo
    }
}

fn replace(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        if let (Some(Valor::Texto(old)), Some(Valor::Texto(new))) = (args.get(1), args.get(2)) {
            Valor::Texto(s.replace(old, new))
        } else {
            Valor::Texto(s.clone())
        }
    } else {
        Valor::Nulo
    }
}

fn to_upper(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        Valor::Texto(s.to_uppercase())
    } else {
        Valor::Nulo
    }
}

fn to_lower(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(s)) = args.get(0) {
        Valor::Texto(s.to_lowercase())
    } else {
        Valor::Nulo
    }
}

fn search(args: Vec<Valor>) -> Valor {
    if let (Some(Valor::Texto(s)), Some(Valor::Texto(pattern))) = (args.get(0), args.get(1)) {
        match s.find(pattern) {
            Some(idx) => Valor::Entero(idx as i64),
            None => Valor::Entero(-1),
        }
    } else {
        Valor::Entero(-1)
    }
}

fn contains(args: Vec<Valor>) -> Valor {
    if let (Some(Valor::Texto(s)), Some(Valor::Texto(pattern))) = (args.get(0), args.get(1)) {
        Valor::Booleano(s.contains(pattern.as_str()))
    } else {
        Valor::Booleano(false)
    }
}

fn starts_with(args: Vec<Valor>) -> Valor {
    if let (Some(Valor::Texto(s)), Some(Valor::Texto(pattern))) = (args.get(0), args.get(1)) {
        Valor::Booleano(s.starts_with(pattern.as_str()))
    } else {
        Valor::Booleano(false)
    }
}

fn ends_with(args: Vec<Valor>) -> Valor {
    if let (Some(Valor::Texto(s)), Some(Valor::Texto(pattern))) = (args.get(0), args.get(1)) {
        Valor::Booleano(s.ends_with(pattern.as_str()))
    } else {
        Valor::Booleano(false)
    }
}
