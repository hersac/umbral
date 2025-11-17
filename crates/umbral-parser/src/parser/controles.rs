use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_if(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' después de if", p.posicion));
    }
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }
    let bloque_entonces = parsear_bloque(p)?;

    let mut else_ifs = Vec::new();
    while p.coincidir(|t| matches!(t, LexToken::ElseIf)) {
        if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
            return Err(ParseError::nuevo("Se esperaba '(' después de else if", p.posicion));
        }
        let cond = crate::parser::expresiones::parsear_expresion_principal(p)?;
        if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
            return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
        }
        if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
            return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
        }
        let bloque = parsear_bloque(p)?;
        else_ifs.push(ElseIf { condicion: cond, bloque });
    }

    let bloque_else = if p.coincidir(|t| matches!(t, LexToken::Else)) {
        if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
            return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
        }
        Some(parsear_bloque(p)?)
    } else {
        None
    };

    Ok(Sentencia::If(If {
        condicion,
        bloque_entonces,
        else_ifs,
        bloque_else,
    }))
}

pub fn parsear_switch(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' después de switch", p.posicion));
    }
    let expresion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }

    let mut casos = Vec::new();
    let mut default = None;

    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        if p.coincidir(|t| matches!(t, LexToken::Case)) {
            let valor = crate::parser::expresiones::parsear_expresion_principal(p)?;
            if !p.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
                return Err(ParseError::nuevo("Se esperaba '=>'", p.posicion));
            }
            let mut bloque = Vec::new();
            bloque.push(p.parsear_sentencia()?);
            casos.push(Case { valor, bloque });
        } else if p.coincidir(|t| matches!(t, LexToken::Default)) {
            if !p.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
                return Err(ParseError::nuevo("Se esperaba '=>'", p.posicion));
            }
            let mut bloque = Vec::new();
            bloque.push(p.parsear_sentencia()?);
            default = Some(bloque);
        } else {
            return Err(ParseError::nuevo("Se esperaba 'case' o 'default'", p.posicion));
        }
    }

    Ok(Sentencia::Switch(Switch {
        expresion,
        casos,
        default,
    }))
}

pub fn parsear_for(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' después de for", p.posicion));
    }

    let inicializacion = Box::new(p.parsear_sentencia()?);
    
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::PuntoYComa)) {
        return Err(ParseError::nuevo("Se esperaba ';'", p.posicion));
    }

    let incremento = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }
    let bloque = parsear_bloque(p)?;

    Ok(Sentencia::For(For {
        inicializacion,
        condicion,
        incremento,
        bloque,
    }))
}

pub fn parsear_foreach(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' después de foreach", p.posicion));
    }

    p.coincidir(|t| matches!(t, LexToken::DeclararVariable));

    let variable = p.parsear_identificador_consumir()?;
    
    let tipo = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        p.parsear_tipo()?
    } else {
        None
    };

    if !p.coincidir(|t| matches!(t, LexToken::MenorIgual)) {
        return Err(ParseError::nuevo("Se esperaba '<=' en foreach", p.posicion));
    }

    let iterable = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }
    let bloque = parsear_bloque(p)?;

    Ok(Sentencia::ForEach(ForEach {
        variable,
        tipo,
        iterable,
        bloque,
    }))
}

pub fn parsear_while(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' después de while", p.posicion));
    }
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }
    let bloque = parsear_bloque(p)?;

    Ok(Sentencia::While(While { condicion, bloque }))
}

pub fn parsear_dowhile(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(ParseError::nuevo("Se esperaba '{'", p.posicion));
    }
    let bloque = parsear_bloque(p)?;

    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(ParseError::nuevo("Se esperaba '(' después del bloque", p.posicion));
    }
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(ParseError::nuevo("Se esperaba ')'", p.posicion));
    }

    Ok(Sentencia::DoWhile(DoWhile { bloque, condicion }))
}

fn parsear_bloque(p: &mut Parser) -> Result<Vec<Sentencia>, ParseError> {
    let mut sentencias = Vec::new();
    while !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        sentencias.push(p.parsear_sentencia()?);
    }
    Ok(sentencias)
}