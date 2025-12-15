use crate::runtime::valores::Valor;
use rand::Rng;
use std::collections::HashMap;

fn registrar_funcion(mapa: &mut HashMap<String, Valor>, nombre: &str, funcion: fn(Vec<Valor>) -> Valor) {
    mapa.insert(
        nombre.to_string(),
        Valor::FuncionNativa(nombre.to_string(), funcion),
    );
}

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    registrar_funcion(&mut mapa, "parse_int", parsear_entero);
    registrar_funcion(&mut mapa, "parse_float", parsear_flotante);
    registrar_funcion(&mut mapa, "to_string", convertir_texto);
    registrar_funcion(&mut mapa, "random", aleatorio);
    registrar_funcion(&mut mapa, "abs", absoluto);
    registrar_funcion(&mut mapa, "clamp", limitar);
    registrar_funcion(&mut mapa, "min", minimo);
    registrar_funcion(&mut mapa, "max", maximo);

    Valor::Diccionario(mapa)
}

fn parsear_texto_entero(texto: &str) -> Valor {
    match texto.parse::<i64>() {
        Ok(numero) => Valor::Entero(numero),
        Err(_) => Valor::Nulo,
    }
}

fn parsear_entero(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Texto(texto)) => parsear_texto_entero(texto),
        _ => Valor::Nulo,
    }
}

fn parsear_texto_flotante(texto: &str) -> Valor {
    match texto.parse::<f64>() {
        Ok(numero) => Valor::Flotante(numero),
        Err(_) => Valor::Nulo,
    }
}

fn parsear_flotante(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Texto(texto)) => parsear_texto_flotante(texto),
        _ => Valor::Nulo,
    }
}

fn convertir_texto(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(valor) => Valor::Texto(format!("{}", valor)),
        None => Valor::Nulo,
    }
}

fn aleatorio(_argumentos: Vec<Valor>) -> Valor {
    let mut generador = rand::thread_rng();
    Valor::Flotante(generador.gen::<f64>())
}

fn absoluto(argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Entero(numero)) => Valor::Entero(numero.abs()),
        Some(Valor::Flotante(numero)) => Valor::Flotante(numero.abs()),
        _ => Valor::Nulo,
    }
}

fn obtener_tres_enteros(argumentos: &[Valor]) -> Option<(i64, i64, i64)> {
    let valor = match argumentos.get(0) {
        Some(Valor::Entero(v)) => *v,
        _ => return None,
    };

    let minimo = match argumentos.get(1) {
        Some(Valor::Entero(m)) => *m,
        _ => return None,
    };

    let maximo = match argumentos.get(2) {
        Some(Valor::Entero(m)) => *m,
        _ => return None,
    };

    Some((valor, minimo, maximo))
}

fn obtener_tres_flotantes(argumentos: &[Valor]) -> Option<(f64, f64, f64)> {
    let valor = match argumentos.get(0) {
        Some(Valor::Flotante(v)) => *v,
        _ => return None,
    };

    let minimo = match argumentos.get(1) {
        Some(Valor::Flotante(m)) => *m,
        _ => return None,
    };

    let maximo = match argumentos.get(2) {
        Some(Valor::Flotante(m)) => *m,
        _ => return None,
    };

    Some((valor, minimo, maximo))
}

fn limitar(argumentos: Vec<Valor>) -> Valor {
    if let Some((valor, minimo, maximo)) = obtener_tres_enteros(&argumentos) {
        return Valor::Entero(valor.clamp(minimo, maximo));
    }

    if let Some((valor, minimo, maximo)) = obtener_tres_flotantes(&argumentos) {
        return Valor::Flotante(valor.clamp(minimo, maximo));
    }

    Valor::Nulo
}

fn obtener_dos_enteros(argumentos: &[Valor]) -> Option<(i64, i64)> {
    let primero = match argumentos.get(0) {
        Some(Valor::Entero(a)) => *a,
        _ => return None,
    };

    let segundo = match argumentos.get(1) {
        Some(Valor::Entero(b)) => *b,
        _ => return None,
    };

    Some((primero, segundo))
}

fn obtener_dos_flotantes(argumentos: &[Valor]) -> Option<(f64, f64)> {
    let primero = match argumentos.get(0) {
        Some(Valor::Flotante(a)) => *a,
        _ => return None,
    };

    let segundo = match argumentos.get(1) {
        Some(Valor::Flotante(b)) => *b,
        _ => return None,
    };

    Some((primero, segundo))
}

fn minimo(argumentos: Vec<Valor>) -> Valor {
    if let Some((primero, segundo)) = obtener_dos_enteros(&argumentos) {
        return Valor::Entero(primero.min(segundo));
    }

    if let Some((primero, segundo)) = obtener_dos_flotantes(&argumentos) {
        return Valor::Flotante(primero.min(segundo));
    }

    Valor::Nulo
}

fn maximo(argumentos: Vec<Valor>) -> Valor {
    if let Some((primero, segundo)) = obtener_dos_enteros(&argumentos) {
        return Valor::Entero(primero.max(segundo));
    }

    if let Some((primero, segundo)) = obtener_dos_flotantes(&argumentos) {
        return Valor::Flotante(primero.max(segundo));
    }

    Valor::Nulo
}
