use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_if(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    let condicion = parsear_condicion_entre_parentesis(parseador, "if")?;
    let bloque_entonces = parsear_bloque_con_llaves(parseador)?;
    let else_ifs = parsear_else_ifs(parseador)?;
    let bloque_else = parsear_bloque_else_opcional(parseador)?;

    Ok(Sentencia::If(If {
        condicion,
        bloque_entonces,
        else_ifs,
        bloque_else,
    }))
}

fn parsear_condicion_entre_parentesis(parseador: &mut Parser, contexto: &str) -> Result<Expresion, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(parseador.crear_error(&format!("Se esperaba '(' después de {}", contexto)));
    }
    
    let condicion = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(parseador.crear_error("Se esperaba ')'"));
    }
    
    Ok(condicion)
}

fn parsear_bloque_con_llaves(parseador: &mut Parser) -> Result<Vec<Sentencia>, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(parseador.crear_error("Se esperaba '{'"));
    }
    
    parsear_bloque(parseador)
}

fn parsear_else_ifs(parseador: &mut Parser) -> Result<Vec<ElseIf>, ParseError> {
    let mut else_ifs = Vec::new();
    
    while parseador.coincidir(|t| matches!(t, LexToken::ElseIf)) {
        let condicion = parsear_condicion_entre_parentesis(parseador, "else if")?;
        let bloque = parsear_bloque_con_llaves(parseador)?;
        else_ifs.push(ElseIf { condicion, bloque });
    }
    
    Ok(else_ifs)
}

fn parsear_bloque_else_opcional(parseador: &mut Parser) -> Result<Option<Vec<Sentencia>>, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::Else)) {
        return Ok(None);
    }
    
    let bloque = parsear_bloque_con_llaves(parseador)?;
    Ok(Some(bloque))
}

pub fn parsear_switch(parseador: &mut Parser) -> Result<Sentencia, ParseError> {
    let expresion = parsear_condicion_entre_parentesis(parseador, "switch")?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(parseador.crear_error("Se esperaba '{'"));
    }

    let (casos, default) = parsear_casos_switch(parseador)?;

    Ok(Sentencia::Switch(Switch {
        expresion,
        casos,
        default,
    }))
}

fn parsear_casos_switch(parseador: &mut Parser) -> Result<(Vec<Case>, Option<Vec<Sentencia>>), ParseError> {
    let mut casos = Vec::new();
    let mut default = None;

    while !parseador.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        if parseador.coincidir(|t| matches!(t, LexToken::Case)) {
            let caso = parsear_caso_individual(parseador)?;
            casos.push(caso);
            continue;
        }
        
        if parseador.coincidir(|t| matches!(t, LexToken::Default)) {
            default = Some(parsear_bloque_default(parseador)?);
            continue;
        }
        
        return Err(parseador.crear_error("Se esperaba 'case' o 'default'"));
    }

    Ok((casos, default))
}

fn parsear_caso_individual(parseador: &mut Parser) -> Result<Case, ParseError> {
    let valor = crate::parser::expresiones::parsear_expresion_principal(parseador)?;
    
    if !parseador.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
        return Err(parseador.crear_error("Se esperaba '=>'"));
    }
    
    let mut bloque = Vec::new();
    bloque.push(parseador.parsear_sentencia()?);
    
    Ok(Case { valor, bloque })
}

fn parsear_bloque_default(parseador: &mut Parser) -> Result<Vec<Sentencia>, ParseError> {
    if !parseador.coincidir(|t| matches!(t, LexToken::FlechaDoble)) {
        return Err(parseador.crear_error("Se esperaba '=>'"));
    }
    
    let mut bloque = Vec::new();
    bloque.push(parseador.parsear_sentencia()?);
    
    Ok(bloque)
}

pub fn parsear_for(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(p.crear_error("Se esperaba '(' después de for"));
    }

    let inicializacion = Box::new(p.parsear_sentencia()?);
    
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::PuntoYComa)) {
        return Err(p.crear_error("Se esperaba ';'"));
    }

    let incremento = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(p.crear_error("Se esperaba ')'"));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
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
        return Err(p.crear_error("Se esperaba '(' después de foreach"));
    }

    p.coincidir(|t| matches!(t, LexToken::DeclararVariable));

    let variable = p.parsear_identificador_consumir()?;
    
    let tipo = if p.coincidir(|t| matches!(t, LexToken::OperadorTipo)) {
        p.parsear_tipo()?
    } else {
        None
    };

    if !p.coincidir(|t| matches!(t, LexToken::MenorIgual)) {
        return Err(p.crear_error("Se esperaba '<=' en foreach"));
    }

    let iterable = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(p.crear_error("Se esperaba ')'"));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
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
        return Err(p.crear_error("Se esperaba '(' después de while"));
    }
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(p.crear_error("Se esperaba ')'"));
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
    }
    let bloque = parsear_bloque(p)?;

    Ok(Sentencia::While(While { condicion, bloque }))
}

pub fn parsear_dowhile(p: &mut Parser) -> Result<Sentencia, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return Err(p.crear_error("Se esperaba '{'"));
    }
    let bloque = parsear_bloque(p)?;

    if !p.coincidir(|t| matches!(t, LexToken::ParentesisIzq)) {
        return Err(p.crear_error("Se esperaba '(' después del bloque"));
    }
    let condicion = crate::parser::expresiones::parsear_expresion_principal(p)?;
    if !p.coincidir(|t| matches!(t, LexToken::ParentesisDer)) {
        return Err(p.crear_error("Se esperaba ')'"));
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