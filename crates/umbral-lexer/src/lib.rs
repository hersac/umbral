use std::iter::Peekable;
use std::str::Chars;

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

fn leer_cadena_simple(iter: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = iter.next() {
        if c == '\'' {
            break;
        }
        s.push(c);
    }
    s
}

fn leer_cadena_doble(iter: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = iter.next() {
        if c == '"' {
            break;
        }
        s.push(c);
    }
    s
}

fn leer_triple_comilla_simple(iter: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = iter.next() {
        if c == '\'' && iter.peek().copied() == Some('\'') && iter.clone().nth(1) == Some('\'') {
            iter.next();
            iter.next();
            break;
        }
        s.push(c);
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

        if ch == '\'' && doble == Some('\'') && iterador.clone().nth(1) == Some('\'') {
            iterador.next();
            iterador.next();
            let val = leer_triple_comilla_simple(&mut iterador);
            lista.push(Token::CadenaMultilinea(val));
            continue;
        }

        if ch == '\'' {
            let val = leer_cadena_simple(&mut iterador);
            lista.push(Token::Cadena(val));
            continue;
        }

        if ch == '"' {
            let val = leer_cadena_doble(&mut iterador);
            lista.push(Token::Cadena(val));
            continue;
        }

        if ch.is_ascii_digit() {
            let numero = leer_numero(&mut iterador, ch);
            lista.push(Token::Numero(numero));
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
                    "r" => {
                        lista.push(Token::Return);
                        continue;
                    }
                    "tprint" => {
                        lista.push(Token::TPrint);
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
                    "n" => {
                        lista.push(Token::Instanciar);
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
                    "imp" => {
                        lista.push(Token::Implementacion);
                        continue;
                    }
                    "em" => {
                        lista.push(Token::DeclararEnum);
                        continue;
                    }
                    "in" => {
                        lista.push(Token::DeclararInterfaz);
                        continue;
                    }
                    "cs" => {
                        lista.push(Token::DeclararClase);
                        continue;
                    }
                    _ => {
                        // Si no coincide con ninguna palabra clave, retroceder y tratar como identificador
                        lista.push(Token::Identificador(palabra.clone()));
                        lista.push(Token::DosPuntos);
                        continue;
                    }
                }
            }

            if palabra == "true" {
                lista.push(Token::Verdadero);
                continue;
            }

            if palabra == "false" {
                lista.push(Token::Falso);
                continue;
            }

            if palabra == "null" {
                lista.push(Token::Nulo);
                continue;
            }

            if palabra == "pr" {
                lista.push(Token::PropPrivada);
                continue;
            }

            if palabra == "pu" {
                lista.push(Token::PropPublica);
                continue;
            }

            if palabra == "imp" {
                lista.push(Token::Implementacion);
                continue;
            }

            lista.push(Token::Identificador(palabra.clone()));

            if iterador.peek().copied() == Some('-') && iterador.clone().nth(1) == Some('>') {
                iterador.next();
                iterador.next();
                lista.push(Token::OperadorTipo);

                let mut prefijo = String::new();
                while iterador.peek().copied() == Some('[') && iterador.clone().nth(1) == Some(']')
                {
                    iterador.next();
                    iterador.next();
                    prefijo.push_str("[]");
                }

                let mut tipo_base = String::new();
                while let Some(&c) = iterador.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        tipo_base.push(iterador.next().unwrap());
                    } else {
                        break;
                    }
                }

                lista.push(Token::Tipo(format!("{}{}", prefijo, tipo_base)));
            }

            continue;
        }

        match ch {
            '-' => {
                if doble == Some('>') {
                    iterador.next();
                    lista.push(Token::OperadorTipo);
                    continue;
                }
                if doble == Some('-') {
                    iterador.next();
                    lista.push(Token::Decremento);
                    continue;
                }
                lista.push(Token::Resta);
                continue;
            }
            '+' => {
                if doble == Some('+') {
                    iterador.next();
                    lista.push(Token::Incremento);
                    continue;
                }
                lista.push(Token::Suma);
                continue;
            }
            '=' => {
                if doble == Some('=') {
                    iterador.next();
                    lista.push(Token::IgualIgual);
                    continue;
                }
                if doble == Some('>') {
                    iterador.next();
                    lista.push(Token::FlechaDoble);
                    continue;
                }
                lista.push(Token::Asignacion);
                continue;
            }
            '!' => {
                if doble == Some('=') {
                    iterador.next();
                    lista.push(Token::Diferente);
                    continue;
                }
                lista.push(Token::Not);
                continue;
            }
            '<' => {
                if doble == Some('=') {
                    iterador.next();
                    lista.push(Token::MenorIgual);
                    continue;
                }
                lista.push(Token::Menor);
                continue;
            }
            '>' => {
                if doble == Some('=') {
                    iterador.next();
                    lista.push(Token::MayorIgual);
                    continue;
                }
                lista.push(Token::Mayor);
                continue;
            }
            '&' => {
                if doble == Some('&') {
                    iterador.next();
                    lista.push(Token::And);
                    continue;
                }
                continue;
            }
            '|' => {
                if doble == Some('|') {
                    iterador.next();
                    lista.push(Token::Or);
                    continue;
                }
                continue;
            }
            '.' => {
                if iterador.peek().copied() == Some('.') {
                    iterador.next();
                    if iterador.peek().copied() == Some('=') {
                        iterador.next();
                        lista.push(Token::RangoIncluyente);
                        continue;
                    }
                    lista.push(Token::Rango);
                    continue;
                }
                lista.push(Token::Punto);
                continue;
            }
            ':' => {
                lista.push(Token::DosPuntos);
                continue;
            }
            ',' => {
                lista.push(Token::Coma);
                continue;
            }
            '(' => {
                lista.push(Token::ParentesisIzq);
                continue;
            }
            ')' => {
                lista.push(Token::ParentesisDer);
                continue;
            }
            '{' => {
                lista.push(Token::LlaveIzq);
                continue;
            }
            '}' => {
                lista.push(Token::LlaveDer);
                continue;
            }
            '[' => {
                lista.push(Token::CorcheteIzq);
                continue;
            }
            ']' => {
                lista.push(Token::CorcheteDer);
                continue;
            }
            '*' => {
                lista.push(Token::Multiplicacion);
                continue;
            }
            '/' => {
                lista.push(Token::Division);
                continue;
            }
            '%' => {
                lista.push(Token::Modulo);
                continue;
            }
            ';' => {
                lista.push(Token::PuntoYComa);
                continue;
            }
            ch if ch.is_whitespace() => continue,
            _ => lista.push(Token::Desconocido(ch)),
        }
    }

    lista
}
