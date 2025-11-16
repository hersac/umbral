use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
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
    DeclararEnum,

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

    TPrint,

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

    Verdadero,
    Falso,
    Nulo,

    Desconocido(char),
}

fn leer_cadena(iter: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = iter.next() {
        if c == '"' {
            break;
        }
        s.push(c);
    }
    s
}

fn leer_multilinea(iter: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    loop {
        let a = iter.next();
        if a.is_none() {
            break;
        }
        let ch = a.unwrap();
        if ch == '\'' && iter.peek().copied() == Some('\'') && iter.clone().nth(1) == Some('\'') {
            iter.next();
            iter.next();
            break;
        }
        s.push(ch);
    }
    s
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

pub fn analizar(texto: &str) -> Vec<Token> {
    let mut lista = Vec::new();
    let mut iterador = texto.chars().peekable();

    while let Some(ch) = iterador.next() {
        let doble = iterador.peek().copied();

        if ch == '!' && doble == Some('!') {
            iterador.next();
            while let Some(n) = iterador.next() {
                if n == '\n' {
                    break;
                }
            }
            continue;
        }

        if ch == '\'' && doble == Some('\'') {
            iterador.next();
            if iterador.peek().copied() == Some('\'') {
                iterador.next();
                let val = leer_multilinea(&mut iterador);
                lista.push(Token::CadenaMultilinea(val));
                continue;
            }
        }

        if ch == '"' {
            let val = leer_cadena(&mut iterador);
            lista.push(Token::Cadena(val));
            continue;
        }

        if ch.is_ascii_digit() {
            let num = leer_numero(&mut iterador, ch);
            lista.push(Token::Numero(num));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let palabra = leer_palabra(&mut iterador, ch);
            let prox = iterador.peek().copied();

            if prox == Some(':') {
                iterador.next();
                match palabra.as_str() {
                    "v" => {
                        lista.push(Token::DeclararVariable);
                        continue;
                    }
                    "c" => {
                        lista.push(Token::DeclararConstante);
                        continue;
                    }
                    "f" => {
                        lista.push(Token::DeclararFuncion);
                        continue;
                    }
                    "n" => {
                        lista.push(Token::Instanciar);
                        continue;
                    }
                    "cs" => {
                        lista.push(Token::DeclararClase);
                        continue;
                    }
                    "pr" => {
                        lista.push(Token::PropPrivada);
                        continue;
                    }
                    "pu" => {
                        lista.push(Token::PropPublica);
                        continue;
                    }
                    "in" => {
                        lista.push(Token::DeclararInterfaz);
                        continue;
                    }
                    "imp" => {
                        lista.push(Token::Implementacion);
                        continue;
                    }
                    "em" => {
                        lista.push(Token::DeclararEnum);
                        continue;
                    }
                    "i" => {
                        lista.push(Token::If);
                        continue;
                    }
                    "ie" => {
                        lista.push(Token::ElseIf);
                        continue;
                    }
                    "e" => {
                        lista.push(Token::Else);
                        continue;
                    }
                    "sw" => {
                        lista.push(Token::Switch);
                        continue;
                    }
                    "ca" => {
                        lista.push(Token::Case);
                        continue;
                    }
                    "def" => {
                        lista.push(Token::Default);
                        continue;
                    }
                    "fo" => {
                        lista.push(Token::For);
                        continue;
                    }
                    "fe" => {
                        lista.push(Token::ForEach);
                        continue;
                    }
                    "wh" => {
                        lista.push(Token::While);
                        continue;
                    }
                    "dw" => {
                        lista.push(Token::DoWhile);
                        continue;
                    }
                    "r" => {
                        lista.push(Token::Return);
                        continue;
                    }
                    _ => {
                        lista.push(Token::Identificador(palabra));
                        continue;
                    }
                }
            }

            match palabra.as_str() {
                "tprint" => {
                    lista.push(Token::TPrint);
                    continue;
                }
                "true" => {
                    lista.push(Token::Verdadero);
                    continue;
                }
                "false" => {
                    lista.push(Token::Falso);
                    continue;
                }
                "null" => {
                    lista.push(Token::Nulo);
                    continue;
                }
                _ => {}
            }

            let primera = palabra.chars().next().unwrap();
            if primera.is_ascii_uppercase() {
                lista.push(Token::Tipo(palabra));
                continue;
            }

            lista.push(Token::Identificador(palabra));
            continue;
        }

        if ch == '-' && doble == Some('>') {
            iterador.next();
            lista.push(Token::OperadorTipo);
            continue;
        }

        if ch == '=' && doble == Some('=') {
            iterador.next();
            lista.push(Token::IgualIgual);
            continue;
        }

        if ch == '=' && doble == Some('>') {
            iterador.next();
            lista.push(Token::FlechaDoble);
            continue;
        }

        if ch == '!' && doble == Some('=') {
            iterador.next();
            lista.push(Token::Diferente);
            continue;
        }

        if ch == '<' && doble == Some('=') {
            iterador.next();
            lista.push(Token::MenorIgual);
            continue;
        }

        if ch == '>' && doble == Some('=') {
            iterador.next();
            lista.push(Token::MayorIgual);
            continue;
        }

        if ch == '&' && doble == Some('&') {
            iterador.next();
            lista.push(Token::And);
            continue;
        }

        if ch == '|' && doble == Some('|') {
            iterador.next();
            lista.push(Token::Or);
            continue;
        }

        if ch == '+' && doble == Some('+') {
            iterador.next();
            lista.push(Token::Incremento);
            continue;
        }

        if ch == '-' && doble == Some('-') {
            iterador.next();
            lista.push(Token::Decremento);
            continue;
        }

        if ch == '.' && iterador.peek().copied() == Some('.') {
            iterador.next();
            if iterador.peek().copied() == Some('=') {
                iterador.next();
                lista.push(Token::RangoIncluyente);
                continue;
            }
            lista.push(Token::Rango);
            continue;
        }

        if ch == '.' {
            lista.push(Token::Punto);
            continue;
        }

        if ch == '&' {
            lista.push(Token::Interpolacion);
            continue;
        }

        if ch == '=' {
            lista.push(Token::Asignacion);
            continue;
        }

        if ch == ':' {
            lista.push(Token::DosPuntos);
            continue;
        }

        if ch == ',' {
            lista.push(Token::Coma);
            continue;
        }

        if ch == '(' {
            lista.push(Token::ParentesisIzq);
            continue;
        }

        if ch == ')' {
            lista.push(Token::ParentesisDer);
            continue;
        }

        if ch == '{' {
            lista.push(Token::LlaveIzq);
            continue;
        }

        if ch == '}' {
            lista.push(Token::LlaveDer);
            continue;
        }

        if ch == '[' {
            lista.push(Token::CorcheteIzq);
            continue;
        }

        if ch == ']' {
            lista.push(Token::CorcheteDer);
            continue;
        }

        if ch == '+' {
            lista.push(Token::Suma);
            continue;
        }

        if ch == '-' {
            lista.push(Token::Resta);
            continue;
        }

        if ch == '*' {
            lista.push(Token::Multiplicacion);
            continue;
        }

        if ch == '/' {
            lista.push(Token::Division);
            continue;
        }

        if ch == '%' {
            lista.push(Token::Modulo);
            continue;
        }

        if ch == ';' {
            lista.push(Token::PuntoYComa);
            continue;
        }

        if ch.is_whitespace() {
            continue;
        }

        lista.push(Token::Desconocido(ch));
    }

    lista
}
