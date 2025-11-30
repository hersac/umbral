use umbral_lexer::{analizar_con_posiciones, TokenConPosicion};
use umbral_parser::Parser;
use umbral_runtime::Runtime;
use std::path::PathBuf;

pub mod error;
pub use error::{InterpreterError, InterpreterResult};

pub struct Interpreter {
    runtime: Runtime,
}

impl Interpreter {
    pub fn nuevo() -> Self {
        Self {
            runtime: Runtime::nuevo(),
        }
    }

    pub fn establecer_directorio_base(&mut self, ruta: PathBuf) {
        self.runtime.establecer_directorio_base(ruta);
    }

    pub fn ejecutar(&mut self, codigo: &str) -> InterpreterResult<()> {
        let tokens = self.tokenizar(codigo)?;
        let ast = self.parsear(tokens, codigo)?;
        self.evaluar(ast)?;
        Ok(())
    }

    pub fn ejecutar_con_resultado(&mut self, codigo: &str) -> InterpreterResult<String> {
        self.ejecutar(codigo)?;
        Ok(String::new())
    }

    pub fn reiniciar(&mut self) {
        self.runtime = Runtime::nuevo();
    }

    fn tokenizar(&self, codigo: &str) -> InterpreterResult<Vec<TokenConPosicion>> {
        let tokens = analizar_con_posiciones(codigo);
        
        if tokens.is_empty() {
            return Err(InterpreterError::LexerError(
                "No se generaron tokens del código fuente".to_string()
            ));
        }
        
        Ok(tokens)
    }

    fn parsear(&self, tokens: Vec<TokenConPosicion>, codigo: &str) -> InterpreterResult<umbral_parser::ast::Programa> {
        let mut parser = Parser::nuevo_con_posiciones(tokens, codigo.to_string());
        
        parser.parsear_programa().map_err(|e| {
            InterpreterError::ParserError(e.formatear_error())
        })
    }

    fn evaluar(&mut self, programa: umbral_parser::ast::Programa) -> InterpreterResult<()> {
        self.runtime.ejecutar(programa);
        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::nuevo()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declaracion_variable_simple() {
        let mut interprete = Interpreter::nuevo();
        let resultado = interprete.ejecutar("v: x = 10;");
        assert!(resultado.is_ok());
    }

    #[test]
    fn test_declaracion_constante() {
        let mut interprete = Interpreter::nuevo();
        let resultado = interprete.ejecutar("c: PI = 3.14;");
        assert!(resultado.is_ok());
    }

    #[test]
    fn test_operacion_aritmetica() {
        let mut interprete = Interpreter::nuevo();
        let resultado = interprete.ejecutar("v: suma = 5 + 10;");
        assert!(resultado.is_ok());
    }

    #[test]
    fn test_funcion_simple() {
        let mut interprete = Interpreter::nuevo();
        let codigo = r#"
            f: sumar(a->Int, b->Int)->Int {
                r: (a + b);
            }
        "#;
        let resultado = interprete.ejecutar(codigo);
        assert!(resultado.is_ok());
    }

    #[test]
    fn test_codigo_vacio() {
        let mut interprete = Interpreter::nuevo();
        let resultado = interprete.ejecutar("");
        assert!(resultado.is_err());
    }

    #[test]
    fn test_reiniciar_interprete() {
        let mut interprete = Interpreter::nuevo();
        interprete.ejecutar("v: x = 10;").unwrap();
        interprete.reiniciar();
        // Después de reiniciar, el estado debería estar limpio
        assert!(interprete.ejecutar("v: y = 20;").is_ok());
    }
}
