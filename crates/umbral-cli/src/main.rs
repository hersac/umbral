use std::fs;
use umbral_lexer::analizar;
use umbral_parser::Parser;

fn main() {
    let ruta_archivo = "codigo-ejemplo/main.um";
    let fuente = match fs::read_to_string(ruta_archivo) {
        Ok(contenido) => contenido,
        Err(e) => {
            println!("Error al leer el archivo {}: {}", ruta_archivo, e);
            return;
        }
    };

    let tokens = analizar(&fuente);

    println!("Tokens:");
    for t in &tokens {
        println!("  {:?}", t);
    }

    let mut parser = Parser::nuevo(tokens);

    match parser.parsear_programa() {
        Ok(ast) => {
            println!("\nAST:");
            println!("{:#?}", ast);
        }
        Err(e) => {
            println!("Error al parsear: {:?}", e);
        }
    }
}
