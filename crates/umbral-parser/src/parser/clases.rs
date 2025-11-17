use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;

pub fn parsear_declaracion_clase(p: &mut Parser) -> Result<Sentencia, ParseError> {
    let nombre = p.parsear_identificador_consumir()?;
    let miembros = p.parsear_bloque()?;
    let mut propiedades = Vec::new();
    let mut metodos = Vec::new();
    for s in miembros {
        match s {
            Sentencia::DeclaracionVariable(dv) => propiedades.push(Propiedad {
                nombre: dv.nombre,
                tipo: dv.tipo,
                publico: false,
                valor_inicial: Some(dv.valor),
            }),
            Sentencia::Funcion(f) => metodos.push(Metodo {
                nombre: f.nombre,
                parametros: f.parametros,
                tipo_retorno: f.tipo_retorno,
                cuerpo: f.cuerpo,
                publico: true,
            }),
            _ => {}
        }
    }
    Ok(Sentencia::Clase(DeclaracionClase {
        nombre,
        propiedades,
        metodos,
    }))
}
