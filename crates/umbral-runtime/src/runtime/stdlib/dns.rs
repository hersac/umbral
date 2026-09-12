use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};

pub fn crear_modulo() -> Valor {
    Valor::Diccionario(HashMap::from([
        ("resolve".to_string(), Valor::FuncionNativa("resolve".to_string(), resolver)),
        ("resolve_ipv4".to_string(), Valor::FuncionNativa("resolve_ipv4".to_string(), resolver_v4)),
        ("resolve_ipv6".to_string(), Valor::FuncionNativa("resolve_ipv6".to_string(), resolver_v6)),
    ]))
}

fn fallar(mensaje: &str) -> Valor {
    eprintln!("Error de red: {}", mensaje);
    Valor::Nulo
}

fn servidor_en(argumentos: &[Valor]) -> Option<String> {
    match argumentos.get(0) {
        Some(Valor::Texto(actual)) => Some(actual.clone()),
        _ => None,
    }
}

fn queda_toda(_direccion: &IpAddr) -> bool {
    true
}

fn queda_v4(direccion: &IpAddr) -> bool {
    direccion.is_ipv4()
}

fn queda_v6(direccion: &IpAddr) -> bool {
    direccion.is_ipv6()
}

fn direcciones_de(servidor: &str) -> Option<Vec<IpAddr>> {
    match (servidor, 0).to_socket_addrs() {
        Ok(todas) => Some(todas.map(|actual| actual.ip()).collect()),
        Err(_) => None,
    }
}

fn lista_filtrada(lista: Vec<IpAddr>, filtro: fn(&IpAddr) -> bool) -> Valor {
    Valor::Lista(
        lista
            .iter()
            .filter(|actual| filtro(actual))
            .map(|actual| Valor::Texto(actual.to_string()))
            .collect(),
    )
}

fn lista_ips(servidor: &str, filtro: fn(&IpAddr) -> bool) -> Valor {
    match direcciones_de(servidor) {
        Some(lista) => lista_filtrada(lista, filtro),
        None => fallar("no se pudo resolver el servidor"),
    }
}

fn resolver(argumentos: Vec<Valor>) -> Valor {
    match servidor_en(&argumentos) {
        Some(servidor) => lista_ips(&servidor, queda_toda),
        None => fallar("resolve requiere servidor Str"),
    }
}

fn resolver_v4(argumentos: Vec<Valor>) -> Valor {
    match servidor_en(&argumentos) {
        Some(servidor) => lista_ips(&servidor, queda_v4),
        None => fallar("resolve_ipv4 requiere servidor Str"),
    }
}

fn resolver_v6(argumentos: Vec<Valor>) -> Valor {
    match servidor_en(&argumentos) {
        Some(servidor) => lista_ips(&servidor, queda_v6),
        None => fallar("resolve_ipv6 requiere servidor Str"),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn resuelve_local() {
        match resolver(vec![Valor::Texto("localhost".to_string())]) {
            Valor::Lista(lista) => assert!(!lista.is_empty()),
            _ => panic!("se esperaba lista"),
        }
    }

    #[test]
    fn filtra_ipv4_local() {
        match resolver_v4(vec![Valor::Texto("localhost".to_string())]) {
            Valor::Lista(lista) => assert!(lista.iter().all(|actual| matches!(actual, Valor::Texto(_)))),
            _ => panic!("se esperaba lista"),
        }
    }

    #[test]
    fn rechaza_servidor_invalido() {
        assert!(matches!(resolver(vec![]), Valor::Nulo));
        assert!(matches!(resolver_v6(vec![Valor::Entero(1)]), Valor::Nulo));
    }
}
