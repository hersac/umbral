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

fn formatear_error_con_indicador(
    mensaje: &str,
    codigo: &str,
    linea: usize,
    columna: usize,
) -> String {
    let lineas: Vec<&str> = codigo.lines().collect();
    
    if linea == 0 || linea > lineas.len() {
        return format!("{} en línea {}, columna {}", mensaje, linea, columna);
    }
    
    let linea_idx = linea - 1;
    let linea_codigo = lineas[linea_idx];
    
    let espacios = " ".repeat(columna.saturating_sub(1));
    
    let indicador = "^";
    
    let mut resultado = String::new();
    resultado.push_str(&format!("Error: {}\n", mensaje));
    resultado.push_str(&format!("  --> línea {}, columna {}\n", linea, columna));
    resultado.push_str(&format!("   |\n"));
    
    if linea_idx > 0 {
        resultado.push_str(&format!("{:3} | {}\n", linea_idx, lineas[linea_idx - 1]));
    }
    
    resultado.push_str(&format!("{:3} | {}\n", linea, linea_codigo));
    resultado.push_str(&format!("   | {}{}\n", espacios, indicador));
    
    if linea_idx + 1 < lineas.len() {
        resultado.push_str(&format!("{:3} | {}\n", linea + 1, lineas[linea_idx + 1]));
    }
    
    resultado.push_str(&format!("   |\n"));
    
    resultado
}
