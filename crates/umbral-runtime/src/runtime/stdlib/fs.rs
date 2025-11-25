use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "read_file".to_string(),
        Valor::FuncionNativa("read_file".to_string(), read_file),
    );
    mapa.insert(
        "write_file".to_string(),
        Valor::FuncionNativa("write_file".to_string(), write_file),
    );
    mapa.insert(
        "exists".to_string(),
        Valor::FuncionNativa("exists".to_string(), exists),
    );

    Valor::Diccionario(mapa)
}

fn read_file(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(path)) = args.get(0) {
        match fs::read_to_string(path) {
            Ok(content) => Valor::Texto(content),
            Err(_) => Valor::Nulo,
        }
    } else {
        Valor::Nulo
    }
}

fn write_file(args: Vec<Valor>) -> Valor {
    if let (Some(Valor::Texto(path)), Some(Valor::Texto(content))) = (args.get(0), args.get(1)) {
        match fs::write(path, content) {
            Ok(_) => Valor::Booleano(true),
            Err(_) => Valor::Booleano(false),
        }
    } else {
        Valor::Booleano(false)
    }
}

fn exists(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(path)) = args.get(0) {
        Valor::Booleano(Path::new(path).exists())
    } else {
        Valor::Booleano(false)
    }
}
