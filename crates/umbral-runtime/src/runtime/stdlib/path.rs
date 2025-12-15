use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::path::Path;

fn registrar_funcion(mapa: &mut HashMap<String, Valor>, nombre: &str, funcion: fn(Vec<Valor>) -> Valor) {
    mapa.insert(
        nombre.to_string(),
        Valor::FuncionNativa(nombre.to_string(), funcion),
    );
}

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    registrar_funcion(&mut mapa, "join", unir);
    registrar_funcion(&mut mapa, "basename", nombre_archivo);
    registrar_funcion(&mut mapa, "dirname", nombre_directorio);
    registrar_funcion(&mut mapa, "extension", extension);

    Valor::Diccionario(mapa)
}

fn unir(argumentos: Vec<Valor>) -> Valor {
    let mut ruta = std::path::PathBuf::new();

    for argumento in argumentos {
        if let Valor::Texto(segmento) = argumento {
            ruta.push(segmento);
        }
    }

    Valor::Texto(ruta.to_string_lossy().to_string())
}

fn obtener_ruta(argumentos: &[Valor]) -> Option<String> {
    match argumentos.get(0) {
        Some(Valor::Texto(ruta)) => Some(ruta.clone()),
        _ => None,
    }
}

fn extraer_nombre_archivo(ruta: &Path) -> Valor {
    match ruta.file_name() {
        Some(nombre) => Valor::Texto(nombre.to_string_lossy().to_string()),
        None => Valor::Texto(String::new()),
    }
}

fn nombre_archivo(argumentos: Vec<Valor>) -> Valor {
    match obtener_ruta(&argumentos) {
        Some(texto_ruta) => {
            let ruta = Path::new(&texto_ruta);
            extraer_nombre_archivo(ruta)
        }
        None => Valor::Nulo,
    }
}

fn extraer_directorio_padre(ruta: &Path) -> Valor {
    match ruta.parent() {
        Some(padre) => Valor::Texto(padre.to_string_lossy().to_string()),
        None => Valor::Texto(String::new()),
    }
}

fn nombre_directorio(argumentos: Vec<Valor>) -> Valor {
    match obtener_ruta(&argumentos) {
        Some(texto_ruta) => {
            let ruta = Path::new(&texto_ruta);
            extraer_directorio_padre(ruta)
        }
        None => Valor::Nulo,
    }
}

fn extraer_extension(ruta: &Path) -> Valor {
    match ruta.extension() {
        Some(ext) => Valor::Texto(ext.to_string_lossy().to_string()),
        None => Valor::Texto(String::new()),
    }
}

fn extension(argumentos: Vec<Valor>) -> Valor {
    match obtener_ruta(&argumentos) {
        Some(texto_ruta) => {
            let ruta = Path::new(&texto_ruta);
            extraer_extension(ruta)
        }
        None => Valor::Nulo,
    }
}
