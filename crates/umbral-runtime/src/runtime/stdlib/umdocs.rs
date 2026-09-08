use crate::runtime::valores::Valor;
use std::collections::HashMap;

fn registrar_funcion(mapa: &mut HashMap<String, Valor>, nombre: &str, fun: fn(Vec<Valor>) -> Valor) {
    mapa.insert(
        nombre.to_string(),
        Valor::FuncionNativa(nombre.to_string(), fun),
    );
}

pub fn crear_modulo() -> Valor {
    let mapa = crear_mapa();
    Valor::Diccionario(mapa)
}

fn crear_mapa() -> HashMap<String, Valor> {
    let mut mapa = HashMap::new();
    registrar_funcion(&mut mapa, "doc", doc);
    registrar_funcion(&mut mapa, "help", doc);
    mapa
}

fn doc(argumentos: Vec<Valor>) -> Valor {
    let primero = argumentos.first();
    if primero.is_none() {
        return Valor::Texto("Uso: Std.doc(funcion) | Std.help(funcion)".to_string());
    }
    Valor::Texto(texto_simple(primero.unwrap()))
}

fn texto_simple(valor: &Valor) -> String {
    match valor {
        Valor::Funcion(f) => f.texto_ayuda(),
        Valor::FuncionNativa(n, _) => format!("{} (función nativa, sin umdocs)", n),
        Valor::Clase(n) => format!("{} (usa help() global o .doc para ver umdocs)", n),
        Valor::Objeto(inst) => format!("{} (usa help() global o .doc para ver umdocs)", inst.clase),
        otro => format!("{} (sin documentación umdocs)", otro.nombre_tipo()),
    }
}
