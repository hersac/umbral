use std::env;
use std::fs;
use std::process;
use umbral_interpreter::Interpreter;

const VERSION: &str = "1.0.0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        mostrar_ayuda();
        return;
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            mostrar_ayuda();
        }
        "--version" | "-v" => {
            mostrar_version();
        }
        ruta_archivo => {
            let codigo = leer_archivo(ruta_archivo);
            ejecutar_codigo(&codigo, ruta_archivo);
        }
    }
}

fn mostrar_ayuda() {
    println!("╔════════════════════════════════════════╗");
    println!("║     Umbral CLI - v{}                ║", VERSION);
    println!("╚════════════════════════════════════════╝");
    println!();
    println!("Lenguaje de programación de propósito general");
    println!();
    println!("USO:");
    println!("    umbral [OPCIONES] <archivo.um>");
    println!();
    println!("OPCIONES:");
    println!("    -h, --help       Muestra esta ayuda");
    println!("    -v, --version    Muestra la versión del intérprete");
    println!();
    println!("ARGUMENTOS:");
    println!("    <archivo.um>     Ruta al archivo .um a ejecutar");
    println!();
    println!("EJEMPLOS:");
    println!("    umbral programa.um");
    println!("    umbral /ruta/completa/script.um");
    println!("    umbral ejemplos/01_variables_y_constantes.um");
    println!();
    println!("Para usar el REPL interactivo, ejecuta:");
    println!("    umbral-repl");
    println!();
    println!("Documentación: https://github.com/hersac/umbral");
}

fn mostrar_version() {
    println!("Umbral v{}", VERSION);
}

fn leer_archivo(ruta: &str) -> String {
    fs::read_to_string(ruta).unwrap_or_else(|e| {
        eprintln!("Error al leer el archivo '{}': {}", ruta, e);
        eprintln!();
        eprintln!("Uso: umbral <archivo.um>");
        eprintln!("Ayuda: umbral --help");
        process::exit(1);
    })
}

fn ejecutar_codigo(codigo: &str, ruta_archivo: &str) {
    let mut interprete = Interpreter::nuevo();

    if let Ok(ruta_abs) = fs::canonicalize(ruta_archivo) {
        if let Some(parent) = ruta_abs.parent() {
            interprete.establecer_directorio_base(parent.to_path_buf());
        }
    }

    if let Err(e) = interprete.ejecutar(codigo) {
        eprintln!("Error de ejecución:");
        eprintln!("{}", e);
        process::exit(1);
    }
}
