use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "now".to_string(),
        Valor::FuncionNativa("now".to_string(), now),
    );
    mapa.insert(
        "timestamp".to_string(),
        Valor::FuncionNativa("timestamp".to_string(), timestamp),
    );

    Valor::Diccionario(mapa)
}

fn now(_args: Vec<Valor>) -> Valor {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Valor::Entero(duration.as_secs() as i64),
        Err(_) => Valor::Nulo,
    }
}

fn timestamp(_args: Vec<Valor>) -> Valor {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Valor::Entero(duration.as_millis() as i64),
        Err(_) => Valor::Nulo,
    }
}
