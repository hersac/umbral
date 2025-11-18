use std::env;
use std::fs;
use umbral_lexer::analizar;
use umbral_parser::Parser;
use umbral_runtime::Runtime;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ruta_archivo = if args.len() > 1 {
        &args[1]
    } else {
        "../codigo-ejemplo/main.um"
    };
    
    let fuente = match fs::read_to_string(ruta_archivo) {
        Ok(contenido) => contenido,
        Err(e) => {
            eprintln!("Error al leer el archivo {}: {}", ruta_archivo, e);
            return;
        }
    };

    let tokens = analizar(&fuente);

    println!("Tokens:");
    for (i, t) in tokens.iter().enumerate() {
        println!("  [{}] {:?}", i, t);
    }

    let mut parser = Parser::nuevo(tokens);

    match parser.parsear_programa() {
        Ok(ast) => {
            println!("\nAST generado correctamente.");

            let mut runtime = Runtime::nuevo();
            runtime.ejecutar(ast);

        }
        Err(e) => {
            eprintln!("Error al parsear: {:?}", e);
        }
    }
}
