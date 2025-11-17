use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;

pub fn parsear_declaracion_interfaz(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let miembros = p.parsear_bloque()?;
    let mut metodos = Vec::new();
    for s in miembros {
        if let Sentencia::Funcion(f) = s {
            metodos.push(Metodo {
                nombre: f.nombre,
                parametros: f.parametros,
                tipo_retorno: f.tipo_retorno,
                cuerpo: f.cuerpo,
                publico: true,
            });
        }
    }
    Ok(Sentencia::Interfaz(DeclaracionInterfaz { nombre, metodos }))
}
