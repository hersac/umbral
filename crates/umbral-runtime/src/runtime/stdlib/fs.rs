use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn registrar_funcion(mapa: &mut HashMap<String, Valor>, nombre: &str, funcion: fn(Vec<Valor>) -> Valor) {
    mapa.insert(
        nombre.to_string(),
        Valor::FuncionNativa(nombre.to_string(), funcion),
    );
}

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    registrar_funcion(&mut mapa, "read_file", leer_archivo);
    registrar_funcion(&mut mapa, "write_file", escribir_archivo);
    registrar_funcion(&mut mapa, "exists", existe);

    Valor::Diccionario(mapa)
}

fn obtener_ruta(argumentos: &[Valor]) -> Option<String> {
    match argumentos.get(0) {
        Some(Valor::Texto(ruta)) => Some(ruta.clone()),
        _ => None,
    }
}

fn leer_contenido_archivo(ruta: &str) -> Valor {
    match fs::read_to_string(ruta) {
        Ok(contenido) => Valor::Texto(contenido),
        Err(_) => Valor::Nulo,
    }
}

fn leer_archivo(argumentos: Vec<Valor>) -> Valor {
    match obtener_ruta(&argumentos) {
        Some(ruta) => leer_contenido_archivo(&ruta),
        None => Valor::Nulo,
    }
}

fn obtener_ruta_y_contenido(argumentos: &[Valor]) -> Option<(String, String)> {
    let ruta = match argumentos.get(0) {
        Some(Valor::Texto(r)) => r.clone(),
        _ => return None,
    };

    let contenido = match argumentos.get(1) {
        Some(Valor::Texto(c)) => c.clone(),
        _ => return None,
    };

    Some((ruta, contenido))
}

fn guardar_contenido(ruta: &str, contenido: &str) -> Valor {
    match fs::write(ruta, contenido) {
        Ok(_) => Valor::Booleano(true),
        Err(_) => Valor::Booleano(false),
    }
}

fn escribir_archivo(argumentos: Vec<Valor>) -> Valor {
    match obtener_ruta_y_contenido(&argumentos) {
        Some((ruta, contenido)) => guardar_contenido(&ruta, &contenido),
        None => Valor::Booleano(false),
    }
}

fn verificar_existencia(ruta: &str) -> Valor {
    Valor::Booleano(Path::new(ruta).exists())
}

fn existe(argumentos: Vec<Valor>) -> Valor {
    match obtener_ruta(&argumentos) {
        Some(ruta) => verificar_existencia(&ruta),
        None => Valor::Booleano(false),
    }
}
