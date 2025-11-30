use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use umbral_lexer::Token as LexToken;

pub fn parsear_importacion(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let mut tiene_equip = false;
    let mut tiene_origin = false;
    let mut items: Vec<ItemImportacion> = Vec::new();
    let mut ruta = String::new();

    loop {
        if p.coincidir(|t| matches!(t, LexToken::Equip)) {
            if tiene_equip {
                return Err(p.crear_error("Palabra clave 'equip' duplicada en importación"));
            }
            tiene_equip = true;
            items = parsear_items_importacion(p)?;
            continue;
        }
        
        if p.coincidir(|t| matches!(t, LexToken::Origin)) {
            if tiene_origin {
                return Err(p.crear_error("Palabra clave 'origin' duplicada en importación"));
            }
            tiene_origin = true;
            ruta = parsear_ruta(p)?;
            continue;
        }
        
        break;
    }

    if !tiene_equip || !tiene_origin {
        return Err(p.crear_error("Se requieren las palabras clave 'equip' y 'origin' en la importación"));
    }

    p.coincidir(|t| matches!(t, LexToken::PuntoYComa));

    Ok(Sentencia::Importacion(Importacion { items, ruta }))
}

fn parsear_items_importacion(p: &mut Parser) -> Result<Vec<ItemImportacion>, ParseError> {
    if p.coincidir(|t| matches!(t, LexToken::Multiplicacion)) {
        let alias = parsear_alias_opcional(p)?;
        return Ok(vec![ItemImportacion::Todo(alias)]);
    }

    if p.coincidir(|t| matches!(t, LexToken::LlaveIzq)) {
        return parsear_lista_items(p);
    }

    let nombre = parsear_identificador(p)?;
    let alias = parsear_alias_opcional(p)?;

    Ok(vec![ItemImportacion::Nombre(nombre, alias)])
}

fn parsear_alias_opcional(p: &mut Parser) -> Result<Option<String>, ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::As)) {
        return Ok(None);
    }
    
    Ok(Some(parsear_identificador(p)?))
}

fn parsear_lista_items(p: &mut Parser) -> Result<Vec<ItemImportacion>, ParseError> {
    let mut items = Vec::new();
    
    if p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        return Ok(vec![ItemImportacion::ListaNombres(items)]);
    }

    loop {
        if p.coincidir(|t| matches!(t, LexToken::Multiplicacion)) {
            parsear_item_asterisco(p, &mut items)?;
        }
        
        if !p.coincidir(|t| matches!(t, LexToken::Multiplicacion)) {
            parsear_item_normal(p, &mut items)?;
        }

        if !p.coincidir(|t| matches!(t, LexToken::Coma)) {
            break;
        }
    }

    if !p.coincidir(|t| matches!(t, LexToken::LlaveDer)) {
        return Err(p.crear_error("Se esperaba '}' al final de la lista de importación"));
    }

    Ok(vec![ItemImportacion::ListaNombres(items)])
}

fn parsear_item_asterisco(p: &mut Parser, items: &mut Vec<ItemImportacion>) -> Result<(), ParseError> {
    if !p.coincidir(|t| matches!(t, LexToken::As)) {
        return Err(p.crear_error("Se esperaba 'as' después de '*' en importación"));
    }
    let alias = parsear_identificador(p)?;
    items.push(ItemImportacion::Todo(Some(alias)));
    Ok(())
}

fn parsear_item_normal(p: &mut Parser, items: &mut Vec<ItemImportacion>) -> Result<(), ParseError> {
    let nombre = parsear_identificador(p)?;
    let alias = parsear_alias_opcional(p)?;
    items.push(ItemImportacion::Nombre(nombre, alias));
    Ok(())
}

fn parsear_ruta(p: &mut Parser) -> Result<String, ParseError> {
    let Some(LexToken::Cadena(ruta) | LexToken::CadenaLiteral(ruta)) = p.peekear() else {
        return Err(p.crear_error("Se esperaba una ruta de archivo como cadena después de 'origin'"));
    };
    
    let ruta = ruta.clone();
    p.avanzar();
    Ok(ruta)
}

fn parsear_identificador(p: &mut Parser) -> Result<String, ParseError> {
    let Some(LexToken::Identificador(nombre)) = p.peekear() else {
        return Err(p.crear_error("Se esperaba un identificador"));
    };
    
    let nombre = nombre.clone();
    p.avanzar();
    Ok(nombre)
}
