// pub fn add(left: u64, right: u64) -> u64 {
// left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//    #[test]
//    fn it_works() {
//        let result = add(2, 2);
//        assert_eq!(result, 4);
//    }
// }

#[derive(Debug)]
pub enum Token {
    Numero(String),
    Identificador(String),
    ParentesisIzq,
    ParentesisDer,
    Suma,
    Resta,
    Desconocido(char),
}

pub fn analizar(texto: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for c in texto.chars() {
        if c.is_ascii_digit() {
            tokens.push(Token::Numero(c.to_string()));
            continue;
        }
        if c.is_ascii_alphabetic() {
            tokens.push(Token::Identificador(c.to_string()));
            continue;
        }
        if c == '(' {
            tokens.push(Token::ParentesisIzq);
            continue;
        }
        if c == ')' {
           tokens.push(Token::ParentesisDer);
            continue;
        }
        if c == '+' {
            tokens.push(Token::Suma);
            continue;
        }
        if c == '-' {
            tokens.push(Token::Resta);
            continue;
        }
        tokens.push(Token::Desconocido(c));
    }
    tokens
}
