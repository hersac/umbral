use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct TokenConPosicion {
    pub token: Token,
    pub posicion: usize,
}

#[derive(Debug, Clone)]
pub enum Token {
    DeclararVariable,
    DeclararConstante,
    DeclararFuncion,
    Instanciar,
    DeclararClase,
    PropPrivada,
    PropPublica,
    DeclararInterfaz,
    Implementacion,
    Extension,
    DeclararEnum,
    Equip,
    Origin,
    As,
    Out,
    Asterisco,
    If,
    ElseIf,
    Else,
    Switch,
    Case,
    Default,
    For,
    ForEach,
    While,
    DoWhile,
    Return,
    This,
    TPrint,
    Try,
    Catch,
    Finally,
    Throw,
    Asy,
    Awa,
    OperadorTipo,
    FlechaDoble,
    Asignacion,
    IgualIgual,
    Diferente,
    MenorIgual,
    MayorIgual,
    And,
    Or,
    Incremento,
    Decremento,
    Punto,
    Interpolacion,
    Numero(String),
    Cadena(String),
    CadenaLiteral(String),
    CadenaMultilinea(String),
    Identificador(String),
    Tipo(String),
    ParentesisIzq,
    ParentesisDer,
    LlaveIzq,
    LlaveDer,
    CorcheteIzq,
    CorcheteDer,
    PuntoYComa,
    Coma,
    DosPuntos,
    Flecha,
    Suma,
    Resta,
    Multiplicacion,
    Division,
    Modulo,
    Menor,
    Mayor,
    Not,
    Rango,
    RangoIncluyente,
    Spread,
    Verdadero,
    Falso,
    Nulo,
    UmDoc(String),
    Desconocido(char),
}

fn procesar_escape(caracter: char) -> char {
    match caracter {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        _ => caracter,
    }
}

fn agregar_escape(texto: &mut String, iter: &mut Peekable<Chars>) {
    let siguiente = iter.next();
    if siguiente.is_none() {
        return;
    }
    let caracter = siguiente.unwrap();
    let valido = matches!(caracter, 'n' | 't' | 'r' | '\\' | '\'' | '"');
    if valido {
        texto.push(procesar_escape(caracter));
        return;
    }
    texto.push('\\');
    texto.push(caracter);
}

fn leer_cadena_simple(iter: &mut Peekable<Chars>) -> String {
    let mut texto = String::new();
    while let Some(caracter) = iter.next() {
        if caracter == '\'' {
            break;
        }
        if caracter == '\\' {
            agregar_escape(&mut texto, iter);
            continue;
        }
        texto.push(caracter);
    }
    texto
}

fn leer_cadena_doble(iter: &mut Peekable<Chars>) -> String {
    let mut texto = String::new();
    while let Some(caracter) = iter.next() {
        if caracter == '"' {
            break;
        }
        if caracter == '\\' {
            agregar_escape(&mut texto, iter);
            continue;
        }
        texto.push(caracter);
    }
    texto
}

fn fin_triple(iter: &mut Peekable<Chars>) -> bool {
    iter.peek().copied() == Some('\'') && iter.clone().nth(1) == Some('\'')
}

fn consumir_triple(iter: &mut Peekable<Chars>) {
    iter.next();
    iter.next();
}

fn leer_triple_simple(iter: &mut Peekable<Chars>) -> String {
    let mut texto = String::new();
    while let Some(caracter) = iter.next() {
        if caracter == '\'' && fin_triple(iter) {
            consumir_triple(iter);
            break;
        }
        if caracter == '\\' {
            agregar_escape(&mut texto, iter);
            continue;
        }
        texto.push(caracter);
    }
    texto
}

fn leer_numero(iter: &mut Peekable<Chars>, primero: char) -> String {
    let mut numero = primero.to_string();
    let mut punto = false;
    while let Some(&s) = iter.peek() {
        let es_digito = s.is_ascii_digit();
        let es_punto = s == '.' && !punto;
        if !es_digito && !es_punto {
            break;
        }
        if es_punto {
            punto = true;
        }
        numero.push(s);
        iter.next();
    }
    numero
}

fn leer_palabra(iter: &mut Peekable<Chars>, primero: char) -> String {
    let mut palabra = primero.to_string();
    while let Some(&s) = iter.peek() {
        if !s.is_ascii_alphanumeric() && s != '_' {
            break;
        }
        palabra.push(s);
        iter.next();
    }
    palabra
}

fn cierre_en_linea(linea: &str) -> Option<(String, String)> {
    let bytes = linea.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r') {
                j += 1;
            }
            if j + 1 < bytes.len() + 1 && linea[j..].starts_with("!!") {
                return Some((linea[..i].to_string(), linea[j + 2..].to_string()));
            }
        }
        i += 1;
    }
    None
}

fn tiene_cierre_adelante(iter: &Peekable<Chars>) -> bool {
    let mut clon = iter.clone();
    while let Some(&c) = clon.peek() {
        if c == ' ' || c == '\t' || c == '\r' {
            clon.next();
            continue;
        }
        break;
    }
    if clon.peek() != Some(&'$') {
        return false;
    }
    clon.next();
    let mut primera = String::new();
    while let Some(&c) = clon.peek() {
        if c == '\n' {
            clon.next();
            break;
        }
        primera.push(c);
        clon.next();
    }
    if cierre_en_linea(&primera).is_some() {
        return true;
    }
    buscar_cierre_lineas(&mut clon)
}

fn buscar_cierre_lineas(clon: &mut Peekable<Chars>) -> bool {
    for _ in 0..500 {
        let mut linea = String::new();
        let mut avanzo = false;
        while let Some(&c) = clon.peek() {
            avanzo = true;
            if c == '\n' {
                clon.next();
                break;
            }
            linea.push(c);
            clon.next();
        }
        if cierre_en_linea(&linea).is_some() {
            return true;
        }
        let recorte = linea.trim();
        if recorte.starts_with('$') && recorte[1..].trim_start().starts_with("!!") {
            return true;
        }
        if !avanzo {
            break;
        }
    }
    false
}

fn leer_bloque(iter: &mut Peekable<Chars>) -> String {
    let mut contenido = String::new();
    loop {
        let mut linea = String::new();
        let mut hubo_salto = false;
        while let Some(&c) = iter.peek() {
            if c == '\n' {
                iter.next();
                hubo_salto = true;
                break;
            }
            linea.push(c);
            iter.next();
        }
        if let Some((antes, _)) = cierre_en_linea(&linea) {
            if !antes.trim().is_empty() {
                contenido.push_str(&antes);
                contenido.push('\n');
            }
            break;
        }
        let recorte = linea.trim();
        let es_cierre = recorte.starts_with('$') && recorte[1..].trim_start().starts_with("!!");
        if es_cierre {
            break;
        }
        contenido.push_str(&linea);
        contenido.push('\n');
        if !hubo_salto && iter.peek().is_none() {
            break;
        }
    }
    contenido
}

fn intentar_umdoc(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    let mut clon = iter.clone();
    let mut es_umdoc = false;
    while let Some(&c) = clon.peek() {
        if c == ' ' || c == '\t' || c == '\r' {
            clon.next();
            continue;
        }
        if c == '$' {
            es_umdoc = true;
        }
        break;
    }
    if !es_umdoc {
        return false;
    }
    if !tiene_cierre_adelante(iter) {
        return false;
    }
    consumir_umdoc(iter, lista);
    true
}

fn consumir_umdoc(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    while let Some(&c) = iter.peek() {
        if c == ' ' || c == '\t' || c == '\r' {
            iter.next();
            continue;
        }
        break;
    }
    iter.next();
    let primera = leer_primera_linea(iter);
    if let Some((antes, _)) = cierre_en_linea(&primera) {
        lista.push(Token::UmDoc(format!("{}\n", antes)));
        return;
    }
    let mut contenido = String::new();
    if !primera.trim().is_empty() {
        contenido.push_str(&primera);
        contenido.push('\n');
    }
    contenido.push_str(&leer_bloque(iter));
    lista.push(Token::UmDoc(contenido));
}

fn leer_primera_linea(iter: &mut Peekable<Chars>) -> String {
    let mut primera = String::new();
    while let Some(&c) = iter.peek() {
        if c == '\n' {
            break;
        }
        primera.push(c);
        iter.next();
    }
    if iter.peek() == Some(&'\n') {
        iter.next();
    }
    primera
}

fn consumir_comentario(iter: &mut Peekable<Chars>) {
    while let Some(n) = iter.next() {
        if n == '\n' {
            break;
        }
    }
}

pub fn analizar(texto: &str) -> Vec<Token> {
    let mut lista = Vec::new();
    let mut iter = texto.chars().peekable();
    while let Some(ch) = iter.next() {
        if manejar_exclamacion(ch, &mut iter, &mut lista) {
            continue;
        }
        if manejar_triple(ch, &mut iter, &mut lista) {
            continue;
        }
        if manejar_cadena_simple(ch, &mut iter, &mut lista) {
            continue;
        }
        if manejar_cadena_doble(ch, &mut iter, &mut lista) {
            continue;
        }
        if manejar_numero(ch, &mut iter, &mut lista) {
            continue;
        }
        if manejar_palabra(ch, &mut iter, &mut lista) {
            continue;
        }
        manejar_simbolo(ch, &mut iter, &mut lista);
    }
    lista
}

fn manejar_exclamacion(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if ch != '!' {
        return false;
    }
    if iter.peek().copied() != Some('!') {
        return false;
    }
    iter.next();
    if intentar_umdoc(iter, lista) {
        return true;
    }
    consumir_comentario(iter);
    true
}

fn manejar_triple(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if ch != '\'' {
        return false;
    }
    if iter.peek().copied() != Some('\'') {
        return false;
    }
    if iter.clone().nth(1) != Some('\'') {
        return false;
    }
    iter.next();
    iter.next();
    let val = leer_triple_simple(iter);
    lista.push(Token::CadenaMultilinea(val));
    true
}

fn manejar_cadena_simple(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if ch != '\'' {
        return false;
    }
    let val = leer_cadena_simple(iter);
    lista.push(Token::CadenaLiteral(val));
    true
}

fn manejar_cadena_doble(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if ch != '"' {
        return false;
    }
    let val = leer_cadena_doble(iter);
    lista.push(Token::Cadena(val));
    true
}

fn manejar_numero(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if !ch.is_ascii_digit() {
        return false;
    }
    let numero = leer_numero(iter, ch);
    lista.push(Token::Numero(numero));
    true
}

fn manejar_palabra(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if !ch.is_ascii_alphabetic() && ch != '_' {
        return false;
    }
    let palabra = leer_palabra(iter, ch);
    if manejar_palabra_con_dos_puntos(palabra.clone(), iter, lista) {
        return true;
    }
    manejar_palabra_suelta(palabra, iter, lista);
    true
}

fn manejar_palabra_con_dos_puntos(palabra: String, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) -> bool {
    if iter.peek().copied() != Some(':') {
        return false;
    }
    iter.next();
    if palabra_es_reservada(&palabra, lista) {
        return true;
    }
    lista.push(Token::Identificador(palabra));
    lista.push(Token::DosPuntos);
    true
}

fn palabra_es_reservada(palabra: &str, lista: &mut Vec<Token>) -> bool {
    let token = match palabra {
        "v" => Some(Token::DeclararVariable),
        "c" => Some(Token::DeclararConstante),
        "f" => Some(Token::DeclararFuncion),
        "r" => Some(Token::Return),
        "tprint" => Some(Token::TPrint),
        "tr" => Some(Token::Try),
        "ct" => Some(Token::Catch),
        "fy" => Some(Token::Finally),
        "tw" => Some(Token::Throw),
        "i" => Some(Token::If),
        "ie" => Some(Token::ElseIf),
        "e" => Some(Token::Else),
        "sw" => Some(Token::Switch),
        "ca" => Some(Token::Case),
        "def" => Some(Token::Default),
        "fo" => Some(Token::For),
        "fe" => Some(Token::ForEach),
        "wh" => Some(Token::While),
        "dw" => Some(Token::DoWhile),
        "n" => Some(Token::Instanciar),
        "pr" => Some(Token::PropPrivada),
        "pu" => Some(Token::PropPublica),
        "imp" => Some(Token::Implementacion),
        "ext" => Some(Token::Extension),
        "em" => Some(Token::DeclararEnum),
        "in" => Some(Token::DeclararInterfaz),
        "cs" => Some(Token::DeclararClase),
        "asy" => Some(Token::Asy),
        "awa" => Some(Token::Awa),
        _ => None,
    };
    if let Some(t) = token {
        lista.push(t);
        return true;
    }
    false
}

fn manejar_palabra_suelta(palabra: String, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if manejar_palabra_especial(&palabra, lista) {
        return;
    }
    lista.push(Token::Identificador(palabra.clone()));
    intentar_tipo(iter, lista);
}

fn manejar_palabra_especial(palabra: &str, lista: &mut Vec<Token>) -> bool {
    let entrada = buscar_token_especial(palabra);
    if entrada.is_none() {
        return false;
    }
    lista.push(entrada.unwrap());
    true
}

fn buscar_token_especial(palabra: &str) -> Option<Token> {
    let tabla: [(&str, Token); 11] = [
        ("equip", Token::Equip),
        ("origin", Token::Origin),
        ("as", Token::As),
        ("out", Token::Out),
        ("true", Token::Verdadero),
        ("false", Token::Falso),
        ("null", Token::Nulo),
        ("pr", Token::PropPrivada),
        ("pu", Token::PropPublica),
        ("th", Token::This),
        ("asy", Token::Asy),
    ];
    tabla
        .into_iter()
        .find(|(k, _)| *k == palabra)
        .map(|(_, v)| v)
}

fn intentar_tipo(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() != Some('-') {
        return;
    }
    if iter.clone().nth(1) != Some('>') {
        return;
    }
    iter.next();
    iter.next();
    lista.push(Token::OperadorTipo);
    let prefijo = recolectar_prefijo(iter);
    let base = recolectar_base(iter);
    lista.push(Token::Tipo(format!("{}{}", prefijo, base)));
}

fn recolectar_prefijo(iter: &mut Peekable<Chars>) -> String {
    let mut prefijo = String::new();
    while iter.peek().copied() == Some('[') && iter.clone().nth(1) == Some(']') {
        iter.next();
        iter.next();
        prefijo.push_str("[]");
    }
    prefijo
}

fn recolectar_base(iter: &mut Peekable<Chars>) -> String {
    let mut base = String::new();
    while let Some(&c) = iter.peek() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            break;
        }
        base.push(iter.next().unwrap());
    }
    base
}

fn manejar_simbolo(ch: char, iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    match ch {
        '-' => manejar_guion(iter, lista),
        '+' => manejar_mas(iter, lista),
        '=' => manejar_igual(iter, lista),
        '!' => manejar_excl(iter, lista),
        '<' => manejar_menor(iter, lista),
        '>' => manejar_mayor(iter, lista),
        '&' => manejar_amp(iter, lista),
        '|' => manejar_barra(iter, lista),
        '.' => manejar_punto(iter, lista),
        ':' => lista.push(Token::DosPuntos),
        ',' => lista.push(Token::Coma),
        '(' => lista.push(Token::ParentesisIzq),
        ')' => lista.push(Token::ParentesisDer),
        '{' => lista.push(Token::LlaveIzq),
        '}' => lista.push(Token::LlaveDer),
        '[' => lista.push(Token::CorcheteIzq),
        ']' => lista.push(Token::CorcheteDer),
        '*' => lista.push(Token::Multiplicacion),
        '/' => lista.push(Token::Division),
        '%' => lista.push(Token::Modulo),
        ';' => lista.push(Token::PuntoYComa),
        c if c.is_whitespace() => {}
        _ => lista.push(Token::Desconocido(ch)),
    }
}

fn manejar_guion(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('>') {
        iter.next();
        lista.push(Token::OperadorTipo);
        return;
    }
    if iter.peek().copied() == Some('-') {
        iter.next();
        lista.push(Token::Decremento);
        return;
    }
    lista.push(Token::Resta);
}

fn manejar_mas(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('+') {
        iter.next();
        lista.push(Token::Incremento);
        return;
    }
    lista.push(Token::Suma);
}

fn manejar_igual(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('=') {
        iter.next();
        lista.push(Token::IgualIgual);
        return;
    }
    if iter.peek().copied() == Some('>') {
        iter.next();
        lista.push(Token::FlechaDoble);
        return;
    }
    lista.push(Token::Asignacion);
}

fn manejar_excl(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('=') {
        iter.next();
        lista.push(Token::Diferente);
        return;
    }
    lista.push(Token::Not);
}

fn manejar_menor(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('=') {
        iter.next();
        lista.push(Token::MenorIgual);
        return;
    }
    lista.push(Token::Menor);
}

fn manejar_mayor(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('=') {
        iter.next();
        lista.push(Token::MayorIgual);
        return;
    }
    lista.push(Token::Mayor);
}

fn manejar_amp(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('&') {
        iter.next();
        lista.push(Token::And);
        return;
    }
    lista.push(Token::Spread);
}

fn manejar_barra(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() == Some('|') {
        iter.next();
        lista.push(Token::Or);
    }
}

fn manejar_punto(iter: &mut Peekable<Chars>, lista: &mut Vec<Token>) {
    if iter.peek().copied() != Some('.') {
        lista.push(Token::Punto);
        return;
    }
    iter.next();
    if iter.peek().copied() == Some('=') {
        iter.next();
        lista.push(Token::RangoIncluyente);
        return;
    }
    lista.push(Token::Rango);
}

pub fn analizar_con_posiciones(texto: &str) -> Vec<TokenConPosicion> {
    let tokens = analizar(texto);
    let chars: Vec<char> = texto.chars().collect();
    let posiciones = calcular_posiciones(&tokens, &chars);
    tokens
        .into_iter()
        .zip(posiciones)
        .map(|(token, posicion)| TokenConPosicion { token, posicion })
        .collect()
}

fn calcular_posiciones(tokens: &[Token], chars: &[char]) -> Vec<usize> {
    let mut idx = 0;
    let mut out = Vec::new();
    for tok in tokens {
        idx = saltar_espacios_y_comentarios(chars, idx, tok);
        out.push(idx);
        idx = avanzar_token(chars, idx, tok);
    }
    out
}

fn saltar_espacios_y_comentarios(chars: &[char], mut idx: usize, siguiente: &Token) -> usize {
    loop {
        if idx >= chars.len() {
            break;
        }
        if chars[idx].is_whitespace() {
            idx += 1;
            continue;
        }
        if es_comentario(chars, idx) && !es_umdoc_inicio(chars, idx, siguiente) {
            idx = saltar_linea(chars, idx);
            continue;
        }
        break;
    }
    idx
}

fn es_comentario(chars: &[char], idx: usize) -> bool {
    idx + 1 < chars.len() && chars[idx] == '!' && chars[idx + 1] == '!'
}

fn es_umdoc_inicio(chars: &[char], idx: usize, siguiente: &Token) -> bool {
    if !matches!(siguiente, Token::UmDoc(_)) {
        return false;
    }
    es_apertura(&chars[idx..])
}

fn saltar_linea(chars: &[char], mut idx: usize) -> usize {
    while idx < chars.len() && chars[idx] != '\n' {
        idx += 1;
    }
    idx
}

fn avanzar_token(chars: &[char], idx: usize, token: &Token) -> usize {
    if matches!(token, Token::UmDoc(_)) {
        return avanzar_umdoc(chars, idx);
    }
    idx + longitud_token(token, &chars[idx..])
}

fn es_apertura(resto: &[char]) -> bool {
    let mut i = 2;
    while i < resto.len() {
        let c = resto[i];
        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
            continue;
        }
        return c == '$';
    }
    false
}

fn avanzar_umdoc(chars: &[char], inicio: usize) -> usize {
    let mut i = inicio;
    let inicio_apertura = i;
    while i < chars.len() && chars[i] != '\n' {
        i += 1;
    }
    let apertura: String = chars[inicio_apertura..i.min(chars.len())].iter().collect();
    if cierre_en_linea(&apertura).is_some() {
        if i < chars.len() {
            i += 1;
        }
        return i;
    }
    if i < chars.len() {
        i += 1;
    }
    buscar_avance_cierre(chars, i)
}

fn buscar_avance_cierre(chars: &[char], mut i: usize) -> usize {
    loop {
        if i >= chars.len() {
            break;
        }
        let inicio_linea = i;
        while i < chars.len() && chars[i] != '\n' {
            i += 1;
        }
        let linea: String = chars[inicio_linea..i].iter().collect();
        let recorte = linea.trim();
        let es_cierre = cierre_en_linea(&linea).is_some()
            || (recorte.starts_with('$') && recorte[1..].trim_start().starts_with("!!"));
        if i < chars.len() {
            i += 1;
        }
        if es_cierre {
            break;
        }
    }
    i
}

fn longitud_token(token: &Token, _resto: &[char]) -> usize {
    use Token::*;
    match token {
        Numero(s) | Identificador(s) | Tipo(s) => s.len(),
        Cadena(s) => s.len() + 2,
        CadenaLiteral(s) => s.len() + 2,
        CadenaMultilinea(s) => s.len() + 6,
        _ => longitud_reservada(token),
    }
}

fn longitud_reservada(token: &Token) -> usize {
    use Token::*;
    match token {
        DeclararVariable | DeclararConstante | DeclararFuncion => 2,
        Instanciar => 2,
        DeclararClase => 3,
        DeclararInterfaz => 3,
        DeclararEnum => 3,
        PropPrivada => 3,
        PropPublica => 3,
        Equip => 5,
        Origin => 6,
        As => 2,
        Out => 3,
        Implementacion => 4,
        Extension => 4,
        If => 2,
        ElseIf => 3,
        Else => 2,
        Switch => 3,
        Case => 3,
        Default => 4,
        For => 3,
        ForEach => 3,
        While => 3,
        DoWhile => 3,
        Return => 2,
        This => 2,
        TPrint => 6,
        Try => 2,
        Catch => 2,
        Finally => 2,
        Throw => 2,
        Asy => 3,
        Awa => 3,
        FlechaDoble => 2,
        Asignacion => 1,
        IgualIgual => 2,
        Diferente => 2,
        MenorIgual => 2,
        MayorIgual => 2,
        And => 2,
        Or => 2,
        Incremento => 2,
        Decremento => 2,
        OperadorTipo => 2,
        Rango => 2,
        RangoIncluyente => 3,
        Spread => 1,
        Verdadero => 4,
        Falso => 5,
        Nulo => 4,
        ParentesisIzq | ParentesisDer | LlaveIzq | LlaveDer | CorcheteIzq | CorcheteDer
        | PuntoYComa | Coma | DosPuntos | Punto | Flecha | Suma | Resta | Multiplicacion
        | Division | Modulo | Menor | Mayor | Not | Asterisco => 1,
        Interpolacion => 2,
        UmDoc(_) => 5,
        Desconocido(_) => 1,
        _ => 1,
    }
}
