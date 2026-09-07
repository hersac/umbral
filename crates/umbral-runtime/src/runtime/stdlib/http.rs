use super::json::{parsear_texto_json, valor_a_json};
use crate::runtime::valores::{SharedPromesa, Valor};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const CLAVE_MARCA: &str = "__pulse__";
const ESPERA_DEFECTO: Duration = Duration::from_secs(30);

const CLAVES_OPCIONES: &[&str] = &[
    "headers",
    "timeout",
    "timeout_ms",
    "timeout_secs",
    "timeout_sec",
    "query",
    "params",
    "auth",
    "bearer",
    "token",
    "body",
];

const METODOS_SIN_CUERPO: &[&str] = &["GET", "HEAD", "OPTIONS"];

/// Crea la función global `pulse(url, metodo, [cuerpo], [opciones])`.
pub fn crear_funcion_pulse() -> Valor {
    Valor::FuncionNativa("pulse".to_string(), pulse)
}

fn es_opciones(valor: &Valor) -> bool {
    match valor {
        Valor::Diccionario(mapa) => mapa
            .keys()
            .any(|k| CLAVES_OPCIONES.contains(&k.to_lowercase().as_str())),
        _ => false,
    }
}

/// Indica si un diccionario es una respuesta generada por `pulse`.
pub fn es_respuesta_pulse(mapa: &HashMap<String, Valor>) -> bool {
    matches!(mapa.get(CLAVE_MARCA), Some(Valor::Booleano(true)))
}

/// Entrada: cuerpo JSON `{"prop": "valor"}` a sintaxis Umbral `["prop" => "valor"]`.
/// Retorna `Nulo` si el cuerpo no es JSON válido.
pub fn respuesta_parse(mapa: &HashMap<String, Valor>) -> Valor {
    match mapa.get("body") {
        Some(Valor::Texto(texto)) => parsear_texto_json(texto),
        _ => Valor::Nulo,
    }
}

/// Salida: cuerpo en sintaxis Umbral a texto JSON canónico.
/// Retorna `Nulo` si el cuerpo no es JSON válido.
pub fn respuesta_json(mapa: &HashMap<String, Valor>) -> Valor {
    let texto = match mapa.get("body") {
        Some(Valor::Texto(t)) => t,
        _ => return Valor::Nulo,
    };
    let json: serde_json::Value = match serde_json::from_str(texto) {
        Ok(v) => v,
        Err(_) => return Valor::Nulo,
    };
    serde_json::to_string(&json)
        .map(Valor::Texto)
        .unwrap_or(Valor::Nulo)
}

/// Cuerpo tal cual llegó, sin transformar.
pub fn respuesta_texto(mapa: &HashMap<String, Valor>) -> Valor {
    match mapa.get("body") {
        Some(Valor::Texto(texto)) => Valor::Texto(texto.clone()),
        _ => Valor::Texto(String::new()),
    }
}

/// Salida genérica: cualquier `Valor` Umbral a texto JSON.
pub fn valor_a_json_texto(valor: &Valor) -> Valor {
    serde_json::to_string(&valor_a_json(valor))
        .map(Valor::Texto)
        .unwrap_or(Valor::Nulo)
}

/// Entrada genérica: texto JSON a `Valor` Umbral.
pub fn texto_a_valor_umbral(texto: &str) -> Valor {
    parsear_texto_json(texto)
}

fn metodo_normalizado(metodo: &str) -> Option<String> {
    let mayus = metodo.trim().to_uppercase();
    let valido = matches!(
        mayus.as_str(),
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "OPTIONS" | "OPTION" | "HEAD"
    );
    if !valido {
        return None;
    }
    Some(alias_option(mayus))
}

fn alias_option(metodo: String) -> String {
    if metodo == "OPTION" {
        return "OPTIONS".to_string();
    }
    metodo
}

fn texto_de_valor(valor: &Valor) -> Option<String> {
    match valor {
        Valor::Texto(s) => Some(s.clone()),
        Valor::Entero(i) => Some(i.to_string()),
        Valor::Flotante(f) => Some(f.to_string()),
        Valor::Booleano(b) => Some(b.to_string()),
        _ => None,
    }
}

fn serializar_cuerpo(cuerpo: &Valor) -> Option<(String, bool)> {
    match cuerpo {
        Valor::Nulo => None,
        Valor::Texto(s) => Some((s.clone(), false)),
        otro => serde_json::to_string(&valor_a_json(otro))
            .map(|json| (json, true))
            .ok(),
    }
}

fn mapa_opciones(opciones: &Valor) -> Option<&HashMap<String, Valor>> {
    match opciones {
        Valor::Diccionario(m) => Some(m),
        _ => None,
    }
}

fn obtener_clave<'a>(mapa: &'a HashMap<String, Valor>, clave: &str) -> Option<&'a Valor> {
    mapa.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(clave))
        .map(|(_, v)| v)
}

fn extraer_cabeceras(opciones: &Valor) -> Vec<(String, String)> {
    let Some(mapa) = mapa_opciones(opciones) else {
        return Vec::new();
    };
    fusionar_cabeceras(autorizacion(mapa), cabeceras_directas(mapa))
}

fn autorizacion(mapa: &HashMap<String, Valor>) -> Vec<(String, String)> {
    let directa = obtener_clave(mapa, "auth").and_then(texto_de_valor);
    match directa {
        Some(valor) => vec![("authorization".to_string(), valor)],
        None => portador(mapa),
    }
}

fn portador(mapa: &HashMap<String, Valor>) -> Vec<(String, String)> {
    let token = obtener_clave(mapa, "bearer")
        .or_else(|| obtener_clave(mapa, "token"))
        .and_then(texto_de_valor);
    match token {
        Some(valor) => vec![("authorization".to_string(), format!("Bearer {}", valor))],
        None => Vec::new(),
    }
}

fn cabeceras_directas(mapa: &HashMap<String, Valor>) -> Vec<(String, String)> {
    let Some(Valor::Diccionario(cabeceras)) = obtener_clave(mapa, "headers") else {
        return Vec::new();
    };
    cabeceras
        .iter()
        .filter_map(|(nombre, valor)| texto_de_valor(valor).map(|v| (nombre.clone(), v)))
        .collect()
}

fn fusionar_cabeceras(
    base: Vec<(String, String)>,
    extras: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let nombres: HashSet<String> = extras.iter().map(|(n, _)| n.to_lowercase()).collect();
    base.into_iter()
        .filter(|(n, _)| !nombres.contains(&n.to_lowercase()))
        .chain(extras)
        .collect()
}

fn extraer_espera(opciones: &Valor) -> Duration {
    let Some(mapa) = mapa_opciones(opciones) else {
        return ESPERA_DEFECTO;
    };
    espera_milis(mapa)
        .or(espera_segundos(mapa))
        .unwrap_or(ESPERA_DEFECTO)
}

fn espera_milis(mapa: &HashMap<String, Valor>) -> Option<Duration> {
    let valor = obtener_clave(mapa, "timeout_ms").or_else(|| obtener_clave(mapa, "timeout"))?;
    match valor {
        Valor::Entero(i) if *i > 0 => Some(Duration::from_millis(*i as u64)),
        Valor::Flotante(f) if *f > 0.0 => Some(Duration::from_millis(*f as u64)),
        _ => None,
    }
}

fn espera_segundos(mapa: &HashMap<String, Valor>) -> Option<Duration> {
    let valor = obtener_clave(mapa, "timeout_secs").or_else(|| obtener_clave(mapa, "timeout_sec"))?;
    match valor {
        Valor::Entero(i) if *i > 0 => Some(Duration::from_secs(*i as u64)),
        Valor::Flotante(f) if *f > 0.0 => Some(Duration::from_millis((*f * 1000.0) as u64)),
        _ => None,
    }
}

fn extraer_consulta(opciones: &Valor) -> Vec<(String, String)> {
    let Some(mapa) = mapa_opciones(opciones) else {
        return Vec::new();
    };
    let dict = obtener_clave(mapa, "query").or_else(|| obtener_clave(mapa, "params"));
    let Some(Valor::Diccionario(pares)) = dict else {
        return Vec::new();
    };
    pares
        .iter()
        .filter_map(|(clave, valor)| texto_de_valor(valor).map(|v| (clave.clone(), v)))
        .collect()
}

fn repartir_argumentos(args: &[Valor]) -> (Valor, Valor) {
    let tercero = args.get(2).cloned().unwrap_or(Valor::Nulo);
    let cuarto = args.get(3).cloned().unwrap_or(Valor::Nulo);
    ordenar_par(tercero, cuarto)
}

fn ordenar_par(tercero: Valor, cuarto: Valor) -> (Valor, Valor) {
    let invertido = es_opciones(&tercero) && !es_opciones(&cuarto);
    if invertido {
        return (cuarto, tercero);
    }
    (tercero, cuarto)
}

fn respuesta_error(url: &str, mensaje: &str) -> Valor {
    let pares = [
        ("ok", Valor::Booleano(false)),
        ("status", Valor::Entero(0)),
        ("headers", Valor::Diccionario(HashMap::new())),
        ("body", Valor::Texto(String::new())),
        ("url", Valor::Texto(url.to_string())),
        ("error", Valor::Texto(mensaje.to_string())),
        (CLAVE_MARCA, Valor::Booleano(true)),
    ];
    Valor::Diccionario(pares.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

fn pulse(argumentos: Vec<Valor>) -> Valor {
    let Some((url, metodo)) = validar_argumentos(&argumentos) else {
        return Valor::Nulo;
    };
    let (cuerpo, opciones) = repartir_argumentos(&argumentos);
    let cuerpo = cuerpo_final(&metodo, cuerpo, &opciones);
    let tarea = tokio::spawn(async move { ejecutar_peticion(url, metodo, cuerpo, opciones).await });
    Valor::Promesa(SharedPromesa(Arc::new(Mutex::new(Some(tarea)))))
}

fn validar_argumentos(args: &[Valor]) -> Option<(String, String)> {
    let url = match args.first() {
        Some(Valor::Texto(u)) => u.clone(),
        _ => {
            eprintln!("Error: pulse() requiere la url Str como primer argumento.");
            return None;
        }
    };
    let crudo = match args.get(1) {
        Some(Valor::Texto(m)) => m.clone(),
        _ => {
            eprintln!("Error: pulse() requiere el metodo Str como segundo argumento.");
            return None;
        }
    };
    let metodo = match metodo_normalizado(&crudo) {
        Some(m) => m,
        None => {
            eprintln!(
                "Error: metodo '{}' no soportado. Usa GET, POST, PUT, DELETE, PATCH u OPTIONS.",
                crudo
            );
            return None;
        }
    };
    Some((url, metodo))
}

fn cuerpo_final(metodo: &str, cuerpo: Valor, opciones: &Valor) -> Valor {
    let cuerpo = cuerpo_de_opciones(cuerpo, opciones);
    if METODOS_SIN_CUERPO.contains(&metodo) {
        return Valor::Nulo;
    }
    cuerpo
}

fn cuerpo_de_opciones(cuerpo: Valor, opciones: &Valor) -> Valor {
    if !matches!(cuerpo, Valor::Nulo) {
        return cuerpo;
    }
    mapa_opciones(opciones)
        .and_then(|m| obtener_clave(m, "body").cloned())
        .unwrap_or(Valor::Nulo)
}

async fn ejecutar_peticion(url: String, metodo: String, cuerpo: Valor, opciones: Valor) -> Valor {
    let cliente = match reqwest::Client::builder()
        .timeout(extraer_espera(&opciones))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return respuesta_error(&url, &format!("No se pudo crear el cliente HTTP: {}", e))
        }
    };
    let metodo_req = match reqwest::Method::from_bytes(metodo.as_bytes()) {
        Ok(m) => m,
        Err(_) => return respuesta_error(&url, &format!("Método HTTP inválido: {}", metodo)),
    };
    let respuesta = match enviar(cliente, metodo_req, &url, &cuerpo, &opciones).await {
        Ok(r) => r,
        Err(mensaje) => return respuesta_error(&url, &mensaje),
    };
    leer_respuesta(respuesta).await
}

async fn enviar(
    cliente: reqwest::Client,
    metodo: reqwest::Method,
    url: &str,
    cuerpo: &Valor,
    opciones: &Valor,
) -> Result<reqwest::Response, String> {
    let peticion = cliente.request(metodo, url);
    let peticion = con_consulta(peticion, opciones);
    let (peticion, tipo_fijado) = con_cabeceras(peticion, opciones);
    let peticion = con_cuerpo(peticion, cuerpo, tipo_fijado);
    peticion
        .send()
        .await
        .map_err(|e| format!("Error de red: {}", e))
}

fn con_consulta(
    peticion: reqwest::RequestBuilder,
    opciones: &Valor,
) -> reqwest::RequestBuilder {
    let consulta = extraer_consulta(opciones);
    if consulta.is_empty() {
        return peticion;
    }
    peticion.query(&consulta)
}

fn con_cabeceras(
    peticion: reqwest::RequestBuilder,
    opciones: &Valor,
) -> (reqwest::RequestBuilder, bool) {
    let cabeceras = extraer_cabeceras(opciones);
    let fijado = cabeceras
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("content-type"));
    let peticion = cabeceras
        .into_iter()
        .fold(peticion, |p, (n, v)| p.header(n.as_str(), v.as_str()));
    (peticion, fijado)
}

fn con_cuerpo(
    peticion: reqwest::RequestBuilder,
    cuerpo: &Valor,
    tipo_fijado: bool,
) -> reqwest::RequestBuilder {
    let Some((texto, es_json)) = serializar_cuerpo(cuerpo) else {
        return peticion;
    };
    if es_json && !tipo_fijado {
        return peticion.header("content-type", "application/json").body(texto);
    }
    peticion.body(texto)
}

async fn leer_respuesta(respuesta: reqwest::Response) -> Valor {
    let estado = respuesta.status().as_u16() as i64;
    let url_final = respuesta.url().to_string();
    let cabeceras = cabeceras_respuesta(&respuesta);
    let cuerpo = match respuesta.text().await {
        Ok(t) => t,
        Err(e) => {
            return respuesta_error(&url_final, &format!("Error al leer el cuerpo: {}", e))
        }
    };
    armar_respuesta(estado, url_final, cabeceras, cuerpo)
}

fn cabeceras_respuesta(respuesta: &reqwest::Response) -> HashMap<String, Valor> {
    respuesta
        .headers()
        .iter()
        .fold(HashMap::new(), |mapa, (nombre, valor)| {
            combinar_cabecera(mapa, nombre.as_str(), valor.to_str().unwrap_or(""))
        })
}

fn combinar_cabecera(mapa: HashMap<String, Valor>, clave: &str, texto: &str) -> HashMap<String, Valor> {
    let combinado = match mapa.get(clave) {
        Some(Valor::Texto(previo)) if !texto.is_empty() => format!("{}, {}", previo, texto),
        Some(Valor::Texto(previo)) => previo.clone(),
        _ => texto.to_string(),
    };
    mapa.into_iter()
        .filter(|(k, _)| k != clave)
        .chain([(clave.to_string(), Valor::Texto(combinado))])
        .collect()
}

fn armar_respuesta(
    estado: i64,
    url: String,
    cabeceras: HashMap<String, Valor>,
    cuerpo: String,
) -> Valor {
    let pares = [
        ("ok", Valor::Booleano((200..300).contains(&estado))),
        ("status", Valor::Entero(estado)),
        ("headers", Valor::Diccionario(cabeceras)),
        ("body", Valor::Texto(cuerpo)),
        ("url", Valor::Texto(url)),
        ("error", Valor::Nulo),
        (CLAVE_MARCA, Valor::Booleano(true)),
    ];
    Valor::Diccionario(pares.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diccionario(pares: Vec<(&str, Valor)>) -> HashMap<String, Valor> {
        pares.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    #[test]
    fn test_metodos_validos() {
        assert_eq!(metodo_normalizado("GET"), Some("GET".to_string()));
        assert_eq!(metodo_normalizado("post"), Some("POST".to_string()));
        assert_eq!(metodo_normalizado("Put"), Some("PUT".to_string()));
        assert_eq!(metodo_normalizado("delete"), Some("DELETE".to_string()));
        assert_eq!(metodo_normalizado("patch"), Some("PATCH".to_string()));
        assert_eq!(metodo_normalizado("OPTIONS"), Some("OPTIONS".to_string()));
        assert_eq!(metodo_normalizado("OPTION"), Some("OPTIONS".to_string()));
        assert_eq!(metodo_normalizado("head"), Some("HEAD".to_string()));
        assert_eq!(metodo_normalizado("FETCH"), None);
    }

    #[test]
    fn test_detecta_opciones() {
        let mapa = diccionario(vec![(
            "headers",
            Valor::Diccionario(HashMap::new()),
        )]);
        assert!(es_opciones(&Valor::Diccionario(mapa)));

        let cuerpo = diccionario(vec![("id", Valor::Entero(1))]);
        assert!(!es_opciones(&Valor::Diccionario(cuerpo)));
        assert!(!es_opciones(&Valor::Texto("hola".to_string())));
    }

    #[test]
    fn test_reparte_argumentos() {
        let url = Valor::Texto("https://x.test".to_string());
        let get = Valor::Texto("GET".to_string());

        let op = Valor::Diccionario(diccionario(vec![("timeout", Valor::Entero(1000))]));
        let body = Valor::Diccionario(diccionario(vec![("id", Valor::Entero(1))]));

        let (cuerpo, opciones) = repartir_argumentos(&[url.clone(), get.clone(), op.clone()]);
        assert!(matches!(cuerpo, Valor::Nulo));
        assert!(es_opciones(&opciones));

        let (cuerpo, opciones) = repartir_argumentos(&[url.clone(), get.clone(), body.clone()]);
        assert!(!matches!(cuerpo, Valor::Nulo));
        assert!(matches!(opciones, Valor::Nulo));

        let (cuerpo, opciones) =
            repartir_argumentos(&[url.clone(), get.clone(), body.clone(), op.clone()]);
        assert!(!matches!(cuerpo, Valor::Nulo));
        assert!(es_opciones(&opciones));

        let (cuerpo, opciones) =
            repartir_argumentos(&[url.clone(), get.clone(), op.clone(), body.clone()]);
        assert!(!matches!(cuerpo, Valor::Nulo));
        assert!(es_opciones(&opciones));
    }

    #[test]
    fn test_convierte_respuesta() {
        let mapa = diccionario(vec![
            ("body", Valor::Texto("{\"a\": 1}".to_string())),
            (CLAVE_MARCA, Valor::Booleano(true)),
        ]);

        assert!(es_respuesta_pulse(&mapa));
        assert!(matches!(respuesta_texto(&mapa), Valor::Texto(_)));

        match respuesta_parse(&mapa) {
            Valor::Diccionario(m) => assert!(matches!(m.get("a"), Some(Valor::Entero(1)))),
            otro => panic!("se esperaba diccionario, llegó {:?}", otro),
        }

        match respuesta_json(&mapa) {
            Valor::Texto(t) => assert_eq!(t, "{\"a\":1}"),
            otro => panic!("se esperaba texto JSON, llegó {:?}", otro),
        }

        let mapa2 = diccionario(vec![("body", Valor::Texto("no-json".to_string()))]);
        assert!(matches!(respuesta_parse(&mapa2), Valor::Nulo));
        assert!(matches!(respuesta_json(&mapa2), Valor::Nulo));
        match respuesta_texto(&mapa2) {
            Valor::Texto(t) => assert_eq!(t, "no-json"),
            _ => panic!("se esperaba texto"),
        }
    }

    #[test]
    fn test_convierte_valores() {
        let persona = diccionario(vec![
            ("nombre", Valor::Texto("Ana García".to_string())),
            ("edad", Valor::Entero(28)),
        ]);
        match valor_a_json_texto(&Valor::Diccionario(persona)) {
            Valor::Texto(t) => {
                assert!(t.contains("\"nombre\":\"Ana García\""));
                assert!(t.contains("\"edad\":28"));
            }
            _ => panic!("se esperaba texto JSON"),
        }

        match texto_a_valor_umbral("{\"propiedad1\": \"valor1\"}") {
            Valor::Diccionario(m) => assert!(matches!(
                m.get("propiedad1"),
                Some(Valor::Texto(v)) if v == "valor1"
            )),
            otro => panic!("se esperaba diccionario, llegó {:?}", otro),
        }
        assert!(matches!(texto_a_valor_umbral("no-json"), Valor::Nulo));
    }

    #[test]
    fn test_rechaza_argumentos() {
        assert!(matches!(pulse(vec![]), Valor::Nulo));
        assert!(matches!(
            pulse(vec![
                Valor::Texto("https://x.test".to_string()),
                Valor::Texto("FETCH".to_string())
            ]),
            Valor::Nulo
        ));
    }

    #[tokio::test]
    async fn test_retorna_promesa() {
        let valor = pulse(vec![
            Valor::Texto("https://x.test".to_string()),
            Valor::Texto("GET".to_string()),
        ]);
        assert!(matches!(valor, Valor::Promesa(_)));
    }
}
