#[derive(Debug, Clone)]
pub struct ParseError {
    pub mensaje: String,
    pub posicion: usize,
}

impl ParseError {
    pub fn nuevo(mensaje: impl Into<String>, posicion: usize) -> Self {
        ParseError {
            mensaje: mensaje.into(),
            posicion,
        }
    }
}
