use crate::ast::{UmDoc, UmDocParam, UmDocReturn};
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn consumir_doc(p: &mut Parser) -> Option<UmDoc> {
    let es_doc = matches!(p.peekear(), Some(LexToken::UmDoc(_)));
    if !es_doc {
        return None;
    }
    let crudo = match p.peekear() {
        Some(LexToken::UmDoc(c)) => c.clone(),
        _ => return None,
    };
    p.avanzar();
    Some(parsear_umdoc(&crudo))
}

pub fn parsear_umdoc(crudo: &str) -> UmDoc {
    let lineas: Vec<String> = crudo.lines().map(limpiar_linea).collect();
    let descripcion = lineas
        .iter()
        .filter(|l| es_descripcion(l))
        .cloned()
        .collect::<Vec<String>>()
        .join("\n");
    let params = lineas.iter().filter_map(|l| parsear_si_param(l)).collect();
    let returns = lineas.iter().find_map(|l| parsear_si_return(l));
    UmDoc {
        descripcion,
        params,
        returns,
        crudo: cruso_a_string(crudo),
    }
}

fn limpiar_linea(linea: &str) -> String {
    let recorte = linea.trim();
    if recorte.starts_with('$') {
        return recorte[1..].trim_start().to_string();
    }
    recorte.to_string()
}

fn es_descripcion(linea: &str) -> bool {
    if linea.is_empty() {
        return false;
    }
    if linea.starts_with("@param") {
        return false;
    }
    if linea.starts_with("@returns") {
        return false;
    }
    if linea.starts_with("@return") {
        return false;
    }
    true
}

fn parsear_si_param(linea: &str) -> Option<UmDocParam> {
    if !linea.starts_with("@param") {
        return None;
    }
    parsear_param(linea)
}

fn parsear_si_return(linea: &str) -> Option<UmDocReturn> {
    let es_ret = linea.starts_with("@returns") || linea.starts_with("@return");
    if !es_ret {
        return None;
    }
    parsear_return(linea)
}

fn cruso_a_string(crudo: &str) -> String {
    crudo.to_string()
}

fn extraer_tipo_prefijo(resto: &str) -> (Option<String>, String) {
    let sin_flecha = quitar_flecha(resto);
    extraer_corchete(sin_flecha)
}

fn quitar_flecha(texto: &str) -> &str {
    let recorte = texto.trim_start();
    if recorte.starts_with("->") {
        return recorte[2..].trim_start();
    }
    recorte
}

fn extraer_corchete(texto: &str) -> (Option<String>, String) {
    if !texto.starts_with('[') {
        return (None, texto.to_string());
    }
    let fin = texto.find(']');
    if fin.is_none() {
        return (None, texto.to_string());
    }
    let indice = fin.unwrap();
    let tipo = texto[1..indice].trim().to_string();
    let resto = texto[indice + 1..].trim_start().to_string();
    let opt = match tipo.is_empty() {
        true => None,
        false => Some(tipo),
    };
    (opt, resto)
}

fn parsear_param(linea: &str) -> Option<UmDocParam> {
    let resto = linea["@param".len()..].trim_start();
    let (tipo, resto) = extraer_tipo_prefijo(resto);
    if resto.is_empty() {
        return None;
    }
    let (nombre, descripcion) = dividir_nombre_desc(&resto);
    if nombre.is_empty() {
        return None;
    }
    Some(UmDocParam {
        nombre,
        tipo,
        descripcion,
    })
}

fn dividir_nombre_desc(texto: &str) -> (String, String) {
    if let Some((n, d)) = separar_por(texto, " - ") {
        return (n, d);
    }
    if let Some((n, d)) = separar_por(texto, "-") {
        if !n.is_empty() {
            return (n, d);
        }
    }
    separar_por_espacio(texto)
}

fn separar_por(texto: &str, sep: &str) -> Option<(String, String)> {
    let idx = texto.find(sep)?;
    let nombre = texto[..idx].trim().to_string();
    let desc = texto[idx + sep.len()..].trim().to_string();
    Some((nombre, desc))
}

fn separar_por_espacio(texto: &str) -> (String, String) {
    let mut partes = texto.splitn(2, char::is_whitespace);
    let nombre = partes.next().unwrap_or("").to_string();
    let desc = partes.next().unwrap_or("").trim().to_string();
    (nombre, desc)
}

fn parsear_return(linea: &str) -> Option<UmDocReturn> {
    let resto = prefijo_return(linea);
    let (tipo, descripcion) = extraer_tipo_prefijo(resto);
    Some(UmDocReturn {
        tipo,
        descripcion: descripcion.trim().to_string(),
    })
}

fn prefijo_return(linea: &str) -> &str {
    if linea.starts_with("@returns") {
        return linea["@returns".len()..].trim_start();
    }
    linea["@return".len()..].trim_start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basico_con_tipos() {
        let crudo = " $ Suma dos números.\n $ @param->[Int] a - El primero.\n $ @param->[Int] b - El segundo.\n $ @returns->[Int] La suma.\n";
        let doc = parsear_umdoc(crudo);
        assert_eq!(doc.descripcion, "Suma dos números.");
        assert_eq!(doc.params.len(), 2);
        assert_eq!(doc.params[0].nombre, "a");
        assert_eq!(doc.params[0].tipo.as_deref(), Some("Int"));
        assert_eq!(doc.returns.unwrap().tipo.as_deref(), Some("Int"));
    }

    #[test]
    fn test_tipos_opcionales() {
        let crudo = " $ Suma.\n $ @param a - primero\n $ @returns la suma\n";
        let doc = parsear_umdoc(crudo);
        assert_eq!(doc.params[0].tipo, None);
        assert_eq!(doc.returns.unwrap().tipo, None);
    }
}
