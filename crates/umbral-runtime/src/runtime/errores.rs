#[derive(Debug)]
pub enum RuntimeError {
    VariableNoEncontrada(String),
    TipoInvalido(String),
    Otro(String),
}
