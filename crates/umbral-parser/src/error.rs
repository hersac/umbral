use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub mensaje: String,
    pub posicion: usize,
    pub linea: usize,
    pub columna: usize,
    pub codigo_fuente: Option<String>,
}

impl ParseError {
    pub fn nuevo(mensaje: impl Into<String>, posicion: usize) -> Self {
        ParseError {
            mensaje: mensaje.into(),
            posicion,
            linea: 0,
            columna: 0,
            codigo_fuente: None,
        }
    }

    pub fn con_contexto(
        mensaje: impl Into<String>,
        posicion: usize,
        codigo_fuente: &str,
    ) -> Self {
        let (linea, columna) = calcular_linea_columna(codigo_fuente, posicion);
        ParseError {
            mensaje: mensaje.into(),
            posicion,
            linea,
            columna,
            codigo_fuente: Some(codigo_fuente.to_string()),
        }
    }

    pub fn formatear_error(&self) -> String {
        if let Some(ref codigo) = self.codigo_fuente {
            formatear_error_con_indicador(&self.mensaje, codigo, self.linea, self.columna)
        } else {
            format!("{} en posición {}", self.mensaje, self.posicion)
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatear_error())
    }
}

fn calcular_linea_columna(codigo: &str, posicion: usize) -> (usize, usize) {
    let mut linea = 1;
    let mut columna = 1;
    
    for (i, c) in codigo.chars().enumerate() {
        if i >= posicion {
            break;
        }
        if c == '\n' {
            linea += 1;
            columna = 1;
        } else {
            columna += 1;
        }
    }
    
    (linea, columna)
}

fn validar_linea(linea: usize, total_lineas: usize) -> bool {
    linea > 0 && linea <= total_lineas
}

fn agregar_encabezado_error(resultado: &mut String, mensaje: &str, linea: usize, columna: usize) {
    resultado.push_str(&format!("Error: {}\n", mensaje));
    resultado.push_str(&format!("  --> línea {}, columna {}\n", linea, columna));
    resultado.push_str(&format!("   |\n"));
}

fn agregar_linea_contexto(resultado: &mut String, numero: usize, contenido: &str) {
    resultado.push_str(&format!("{:3} | {}\n", numero, contenido));
}

fn agregar_linea_con_indicador(resultado: &mut String, numero: usize, contenido: &str, columna: usize) {
    let espacios = " ".repeat(columna.saturating_sub(1));
    resultado.push_str(&format!("{:3} | {}\n", numero, contenido));
    resultado.push_str(&format!("   | {}^\n", espacios));
}

fn formatear_error_con_indicador(
    mensaje: &str,
    codigo: &str,
    linea: usize,
    columna: usize,
) -> String {
    let lineas: Vec<&str> = codigo.lines().collect();
    
    if !validar_linea(linea, lineas.len()) {
        return format!("{} en línea {}, columna {}", mensaje, linea, columna);
    }
    
    let indice_linea = linea - 1;
    let mut resultado = String::new();
    
    agregar_encabezado_error(&mut resultado, mensaje, linea, columna);
    
    if indice_linea > 0 {
        agregar_linea_contexto(&mut resultado, linea - 1, lineas[indice_linea - 1]);
    }
    
    agregar_linea_con_indicador(&mut resultado, linea, lineas[indice_linea], columna);
    
    if indice_linea + 1 < lineas.len() {
        agregar_linea_contexto(&mut resultado, linea + 1, lineas[indice_linea + 1]);
    }
    
    resultado.push_str(&format!("   |\n"));
    resultado
}
