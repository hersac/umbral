use crate::runtime::valores::Valor;
use std::collections::HashMap;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "parse".to_string(),
        Valor::FuncionNativa("parse".to_string(), parse),
    );
    mapa.insert(
        "stringify".to_string(),
        Valor::FuncionNativa("stringify".to_string(), stringify),
    );

    Valor::Diccionario(mapa)
}

fn parse(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(json_str)) = args.get(0) {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(json_val) => json_to_valor(&json_val),
            Err(_) => Valor::Nulo,
        }
    } else {
        Valor::Nulo
    }
}

fn stringify(args: Vec<Valor>) -> Valor {
    if let Some(val) = args.get(0) {
        let json_val = valor_to_json(val);
        match serde_json::to_string(&json_val) {
            Ok(s) => Valor::Texto(s),
            Err(_) => Valor::Nulo,
        }
    } else {
        Valor::Nulo
    }
}

fn json_to_valor(json: &serde_json::Value) -> Valor {
    match json {
        serde_json::Value::Null => Valor::Nulo,
        serde_json::Value::Bool(b) => Valor::Booleano(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Valor::Entero(i)
            } else if let Some(f) = n.as_f64() {
                Valor::Flotante(f)
            } else {
                Valor::Nulo
            }
        }
        serde_json::Value::String(s) => Valor::Texto(s.clone()),
        serde_json::Value::Array(arr) => {
            let lista: Vec<Valor> = arr.iter().map(json_to_valor).collect();
            Valor::Lista(lista)
        }
        serde_json::Value::Object(obj) => {
            let mut mapa = HashMap::new();
            for (k, v) in obj {
                mapa.insert(k.clone(), json_to_valor(v));
            }
            Valor::Diccionario(mapa)
        }
    }
}

fn valor_to_json(val: &Valor) -> serde_json::Value {
    match val {
        Valor::Nulo => serde_json::Value::Null,
        Valor::Booleano(b) => serde_json::Value::Bool(*b),
        Valor::Entero(i) => serde_json::Value::Number((*i).into()),
        Valor::Flotante(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Valor::Texto(s) => serde_json::Value::String(s.clone()),
        Valor::Lista(l) => {
            let arr: Vec<serde_json::Value> = l.iter().map(valor_to_json).collect();
            serde_json::Value::Array(arr)
        }
        Valor::Diccionario(d) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in d {
                obj.insert(k.clone(), valor_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}
