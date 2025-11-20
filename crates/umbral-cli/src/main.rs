use std::env;
use std::fs;
use std::process;
use umbral_interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let ruta_archivo = obtener_ruta_archivo(&args);
    let codigo = leer_archivo(ruta_archivo);
    
    ejecutar_codigo(&codigo);
}

fn obtener_ruta_archivo(args: &[String]) -> &str {
    if args.len() > 1 {
        &args[1]
    } else {
        "./codigo-ejemplo/main.um"
    }
}

fn leer_archivo(ruta: &str) -> String {
    fs::read_to_string(ruta).unwrap_or_else(|e| {
        eprintln!("Error al leer el archivo {}: {}", ruta, e);
        process::exit(1);
    })
}

fn ejecutar_codigo(codigo: &str) {
    let mut interprete = Interpreter::nuevo();
    
    if let Err(e) = interprete.ejecutar(codigo) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
