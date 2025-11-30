use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::expresiones;
use umbral_lexer::Token as LexToken;

pub fn parsear_declaracion_constante(p: &mut Parser, exportado: bool) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;

    let tipo = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        Some(p.parsear_tipo()?.ok_or_else(|| {
            p.crear_error("Tipo esperado despues de '->' en constante")
        })?)
    } else {
        None
    };

    if !p.coincidir(|t| matches!(t, LexToken::Asignacion)) {
        return Err(p.crear_error("Se esperaba '=' en declaracion constante"));
    }

    let valor = expresiones::parsear_expresion_principal(p)?;
    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));

    let tipo = tipo.or_else(|| Some(inferir_tipo(&valor)));

    Ok(Sentencia::DeclaracionConstante(DeclaracionConstante {
        nombre,
        tipo,
        valor,
        exportado,
    }))
}

fn inferir_tipo(valor: &Expresion) -> Tipo {
    match valor {
        Expresion::LiteralEntero(_) => Tipo {
            nombre: "Int".to_string(),
        },
        Expresion::LiteralFloat(_) => Tipo {
            nombre: "Flo".to_string(),
        },
        Expresion::LiteralCadena(_) | Expresion::LiteralCadenaLiteral(_) => Tipo {
            nombre: "Str".to_string(),
        },
        Expresion::LiteralBool(_) => Tipo {
            nombre: "Bool".to_string(),
        },
        Expresion::Objeto(_) => Tipo {
            nombre: "Obj".to_string(),
        },
        Expresion::Array(_) => Tipo {
            nombre: "Array".to_string(),
        },
        Expresion::Binaria {
            izquierda,
            derecha,
            operador,
        } => {
            let t_izq = inferir_tipo(izquierda);
            let t_der = inferir_tipo(derecha);
            match operador.as_str() {
                "+" | "-" | "*" | "/" | "%" => {
                    if t_izq.nombre == "Flo" || t_der.nombre == "Flo" {
                        Tipo {
                            nombre: "Flo".to_string(),
                        }
                    } else {
                        Tipo {
                            nombre: "Int".to_string(),
                        }
                    }
                }
                "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" => Tipo {
                    nombre: "Bool".to_string(),
                },
                _ => Tipo {
                    nombre: "Any".to_string(),
                },
            }
        }
        Expresion::Unaria {
            operador,
            expresion,
        } => match operador.as_str() {
            "-" => inferir_tipo(expresion),
            "!" => Tipo {
                nombre: "Bool".to_string(),
            },
            _ => Tipo {
                nombre: "Any".to_string(),
            },
        },
        Expresion::Agrupada(e) => inferir_tipo(e),
        _ => Tipo {
            nombre: "Any".to_string(),
        },
    }
}
