use crate::runtime::interpretador::Interpretador;
use crate::runtime::valores::Valor;
use std::collections::HashMap;

pub mod collections;
pub mod dns;
pub mod errores;
pub mod fs;
pub mod net;
pub mod http;
pub mod json;
pub mod num;
pub mod path;
pub mod proc;
pub mod str;
pub mod time;
pub mod umdocs;

pub fn registrar_stdlib(interpretador: &mut Interpretador) {
    let mut std_map = HashMap::new();

    if let Valor::Diccionario(str_funcs) = str::crear_modulo() {
        std_map.extend(str_funcs);
    }

    if let Valor::Diccionario(num_funcs) = num::crear_modulo() {
        std_map.extend(num_funcs);
    }

    if let Valor::Diccionario(fs_funcs) = fs::crear_modulo() {
        std_map.extend(fs_funcs);
    }

    if let Valor::Diccionario(coll_funcs) = collections::crear_modulo() {
        std_map.extend(coll_funcs);
    }

    if let Valor::Diccionario(path_funcs) = path::crear_modulo() {
        std_map.extend(path_funcs);
    }

    if let Valor::Diccionario(time_funcs) = time::crear_modulo() {
        std_map.extend(time_funcs);
    }

    if let Valor::Diccionario(json_funcs) = json::crear_modulo() {
        std_map.extend(json_funcs);
    }

    if let Valor::Diccionario(proc_funcs) = proc::crear_modulo() {
        std_map.extend(proc_funcs);
    }

    if let Valor::Diccionario(doc_funcs) = umdocs::crear_modulo() {
        std_map.extend(doc_funcs);
    }

    interpretador
        .entorno_actual
        .definir_variable("Std".to_string(), Valor::Diccionario(std_map));

    interpretador
        .entorno_actual
        .definir_variable("pulse".to_string(), http::crear_funcion_pulse());

    interpretador
        .entorno_actual
        .definir_variable("Net".to_string(), net::crear_modulo());

    interpretador
        .entorno_actual
        .definir_variable("Dns".to_string(), dns::crear_modulo());

    let error_class = errores::crear_clase_error();
    interpretador.gestor_clases.registrar_clase(error_class);
}
