#[derive(Debug)]
pub enum Token {
    DeclararVariable,
    OperadorTipo,
    Asignacion,
    IgualIgual,
    Diferente,
    MenorIgual,
    MayorIgual,
    And,
    Or,
    Numero(String),
    Cadena(String),
    CadenaMultilinea(String),
    Identificador(String),
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
    Desconocido(char),
}

pub fn analizar(texto: &str) -> Vec<Token> {
    let mut lista = Vec::new();
    let mut iterador = texto.chars().peekable();

    while let Some(c) = iterador.next() {
        let doble_signo = iterador.peek().copied();

        let es_comentario = c == '!' && doble_signo == Some('!');
        if es_comentario {
            iterador.next();
            while let Some(n) = iterador.next() {
                if n == '\n' {
                    break;
                }
            }
            continue;
        }

        let inicio_multi = c == '\'' && doble_signo == Some('\'');
        if inicio_multi {
            iterador.next();
            if iterador.peek().copied() == Some('\'') {
                iterador.next();
                let mut valor = String::new();
                loop {
                    let sig = iterador.next();
                    if sig.is_none() {
                        break;
                    }
                    let ch = sig.unwrap();
                    let fin_1 = ch == '\'';
                    if fin_1 {
                        let fin_2 = iterador.peek().copied() == Some('\'');
                        if fin_2 {
                            iterador.next();
                            let fin_3 = iterador.peek().copied() == Some('\'');
                            if fin_3 {
                                iterador.next();
                                break;
                            }
                            valor.push(ch);
                            continue;
                        }
                        valor.push(ch);
                        continue;
                    }
                    valor.push(ch);
                }
                lista.push(Token::CadenaMultilinea(valor));
                continue;
            }
        }

        let inicio_cadena = c == '"';
        if inicio_cadena {
            let mut cadena = String::new();
            while let Some(n) = iterador.next() {
                if n == '"' {
                    break;
                }
                cadena.push(n);
            }
            lista.push(Token::Cadena(cadena));
            continue;
        }

        let es_decimal = c.is_ascii_digit();
        if es_decimal {
            let mut numero = c.to_string();
            loop {
                let sig = iterador.peek().copied();
                let es_digito = sig.map(|v| v.is_ascii_digit()).unwrap_or(false);
                let es_punto = sig == Some('.');
                if !es_digito && !es_punto {
                    break;
                }
                numero.push(sig.unwrap());
                iterador.next();
            }
            lista.push(Token::Numero(numero));
            continue;
        }

        let es_palabra = c.is_ascii_alphabetic() || c == '_';
        if es_palabra {
            let mut palabra = c.to_string();
            while let Some(sig) = iterador.peek().copied() {
                let es_valido = sig.is_ascii_alphanumeric() || sig == '_';
                if !es_valido {
                    break;
                }
                palabra.push(sig);
                iterador.next();
            }
            lista.push(Token::Identificador(palabra));
            continue;
        }

        let es_v_dos_puntos = c == 'v' && doble_signo == Some(':');
        if es_v_dos_puntos {
            iterador.next();
            lista.push(Token::DeclararVariable);
            continue;
        }

        let es_flecha = c == '-' && doble_signo == Some('>');
        if es_flecha {
            iterador.next();
            lista.push(Token::OperadorTipo);
            continue;
        }

        let es_igual_igual = c == '=' && doble_signo == Some('=');
        if es_igual_igual {
            iterador.next();
            lista.push(Token::IgualIgual);
            continue;
        }

        let es_diferente = c == '!' && doble_signo == Some('=');
        if es_diferente {
            iterador.next();
            lista.push(Token::Diferente);
            continue;
        }

        let es_menor_igual = c == '<' && doble_signo == Some('=');
        if es_menor_igual {
            iterador.next();
            lista.push(Token::MenorIgual);
            continue;
        }

        let es_mayor_igual = c == '>' && doble_signo == Some('=');
        if es_mayor_igual {
            iterador.next();
            lista.push(Token::MayorIgual);
            continue;
        }

        let es_and = c == '&' && doble_signo == Some('&');
        if es_and {
            iterador.next();
            lista.push(Token::And);
            continue;
        }

        let es_or = c == '|' && doble_signo == Some('|');
        if es_or {
            iterador.next();
            lista.push(Token::Or);
            continue;
        }

        if c == '=' {
            lista.push(Token::Asignacion);
            continue;
        }

        if c == ':' {
            lista.push(Token::DosPuntos);
            continue;
        }

        if c == ',' {
            lista.push(Token::Coma);
            continue;
        }

        if c == '(' {
            lista.push(Token::ParentesisIzq);
            continue;
        }

        if c == ')' {
            lista.push(Token::ParentesisDer);
            continue;
        }

        if c == '{' {
            lista.push(Token::LlaveIzq);
            continue;
        }

        if c == '}' {
            lista.push(Token::LlaveDer);
            continue;
        }

        if c == '[' {
            lista.push(Token::CorcheteIzq);
            continue;
        }

        if c == ']' {
            lista.push(Token::CorcheteDer);
            continue;
        }

        if c == '+' {
            lista.push(Token::Suma);
            continue;
        }

        if c == '-' {
            lista.push(Token::Resta);
            continue;
        }

        if c == ';' {
            lista.push(Token::PuntoYComa);
            continue;
        }

        if c.is_whitespace() {
            continue;
        }

        lista.push(Token::Desconocido(c));
    }

    lista
}
