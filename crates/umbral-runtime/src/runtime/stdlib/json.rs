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

    registrar_funcion(&mut mapa, "parse", parsear);
    registrar_funcion(&mut mapa, "stringify", convertir_texto);

    Valor::Diccionario(mapa)
}

fn parsear_texto_json(texto: &str) -> Valor {
    match serde_json::from_str::<serde_json::Value>(texto) {
        Ok(valor_json) => json_a_valor(&valor_json),
        Err(_) => Valor::Nulo,
    }
}

fn parsear(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Texto(texto)) => parsear_texto_json(texto),
        _ => Valor::Nulo,
    }
}

fn serializar_a_texto(valor_json: &serde_json::Value) -> Valor {
    match serde_json::to_string(valor_json) {
        Ok(texto) => Valor::Texto(texto),
        Err(_) => Valor::Nulo,
    }
}

fn convertir_texto(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(valor) => {
            let valor_json = valor_a_json(valor);
            serializar_a_texto(&valor_json)
        }
        None => Valor::Nulo,
    }
}

fn convertir_numero_json(numero: &serde_json::Number) -> Valor {
    if let Some(entero) = numero.as_i64() {
        return Valor::Entero(entero);
    }
    
    if let Some(flotante) = numero.as_f64() {
        return Valor::Flotante(flotante);
    }
    
    Valor::Nulo
}

fn convertir_array_json(array: &[serde_json::Value]) -> Valor {
    let lista: Vec<Valor> = array.iter().map(json_a_valor).collect();
    Valor::Lista(lista)
}

fn convertir_objeto_json(objeto: &serde_json::Map<String, serde_json::Value>) -> Valor {
    let mut mapa = HashMap::new();
    for (clave, valor) in objeto {
        mapa.insert(clave.clone(), json_a_valor(valor));
    }
    Valor::Diccionario(mapa)
}

fn json_a_valor(json: &serde_json::Value) -> Valor {
    match json {
        serde_json::Value::Null => Valor::Nulo,
        serde_json::Value::Bool(booleano) => Valor::Booleano(*booleano),
        serde_json::Value::Number(numero) => convertir_numero_json(numero),
        serde_json::Value::String(texto) => Valor::Texto(texto.clone()),
        serde_json::Value::Array(array) => convertir_array_json(array),
        serde_json::Value::Object(objeto) => convertir_objeto_json(objeto),
    }
}

fn convertir_flotante_json(flotante: f64) -> serde_json::Value {
    serde_json::Number::from_f64(flotante)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

fn convertir_lista_json(lista: &[Valor]) -> serde_json::Value {
    let array: Vec<serde_json::Value> = lista.iter().map(valor_a_json).collect();
    serde_json::Value::Array(array)
}

fn convertir_diccionario_json(diccionario: &HashMap<String, Valor>) -> serde_json::Value {
    let mut objeto = serde_json::Map::new();
    for (clave, valor) in diccionario {
        objeto.insert(clave.clone(), valor_a_json(valor));
    }
    serde_json::Value::Object(objeto)
}

fn valor_a_json(valor: &Valor) -> serde_json::Value {
    match valor {
        Valor::Nulo => serde_json::Value::Null,
        Valor::Booleano(booleano) => serde_json::Value::Bool(*booleano),
        Valor::Entero(entero) => serde_json::Value::Number((*entero).into()),
        Valor::Flotante(flotante) => convertir_flotante_json(*flotante),
        Valor::Texto(texto) => serde_json::Value::String(texto.clone()),
        Valor::Lista(lista) => convertir_lista_json(lista),
        Valor::Diccionario(diccionario) => convertir_diccionario_json(diccionario),
        _ => serde_json::Value::Null,
    }
}
