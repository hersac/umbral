use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::path::Path;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "join".to_string(),
        Valor::FuncionNativa("join".to_string(), join),
    );
    mapa.insert(
        "basename".to_string(),
        Valor::FuncionNativa("basename".to_string(), basename),
    );
    mapa.insert(
        "dirname".to_string(),
        Valor::FuncionNativa("dirname".to_string(), dirname),
    );
    mapa.insert(
        "extension".to_string(),
        Valor::FuncionNativa("extension".to_string(), extension),
    );

    Valor::Diccionario(mapa)
}

fn join(args: Vec<Valor>) -> Valor {
    let mut path = std::path::PathBuf::new();

    for arg in args {
        if let Valor::Texto(s) = arg {
            path.push(s);
        }
    }

    Valor::Texto(path.to_string_lossy().to_string())
}

fn basename(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(path_str)) = args.get(0) {
        let path = Path::new(path_str);
        if let Some(name) = path.file_name() {
            Valor::Texto(name.to_string_lossy().to_string())
        } else {
            Valor::Texto(String::new())
        }
    } else {
        Valor::Nulo
    }
}

fn dirname(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(path_str)) = args.get(0) {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            Valor::Texto(parent.to_string_lossy().to_string())
        } else {
            Valor::Texto(String::new())
        }
    } else {
        Valor::Nulo
    }
}

fn extension(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(path_str)) = args.get(0) {
        let path = Path::new(path_str);
        if let Some(ext) = path.extension() {
            Valor::Texto(ext.to_string_lossy().to_string())
        } else {
            Valor::Texto(String::new())
        }
    } else {
        Valor::Nulo
    }
}
