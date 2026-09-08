use std::env;
use std::fs;
use std::process;
use umbral_interpreter::Interpreter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        mostrar_ayuda();
        return;
    }
    match args[1].as_str() {
        "--help" | "-h" => mostrar_ayuda(),
        "--version" | "-v" => mostrar_version(),
        "--doc" => manejar_doc(&args),
        ruta => manejar_ruta(ruta, &args).await,
    }
}

fn manejar_doc(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Uso: umbral --doc <archivo.um>");
        process::exit(1);
    }
    let codigo = leer_archivo(&args[2]);
    mostrar_docs(&codigo);
}

async fn manejar_ruta(ruta: &str, args: &[String]) {
    let tiene_doc = args.iter().any(|a| a == "--doc");
    if tiene_doc {
        let destino = args.iter().find(|a| a.ends_with(".um")).unwrap_or(&args[1]);
        let codigo = leer_archivo(destino);
        mostrar_docs(&codigo);
        return;
    }
    let codigo = leer_archivo(ruta);
    ejecutar_codigo(&codigo, ruta).await;
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
    println!("    --doc <archivo.um>  Muestra la documentación umdocs sin ejecutar");
    println!();
    println!("ARGUMENTOS:");
    println!("    <archivo.um>     Ruta al archivo .um a ejecutar");
    println!();
    println!("EJEMPLOS:");
    println!("    umbral programa.um");
    println!("    umbral /ruta/completa/script.um");
    println!("    umbral ejemplos/01_variables_y_constantes.um");
    println!("    umbral --doc programa.um");
    println!();
    println!("Para usar el REPL interactivo, ejecuta:");
    println!("    umbral-repl");
    println!();
    println!("Documentación: https://github.com/hersac/umbral");
}

fn mostrar_docs(codigo: &str) {
    use umbral_parser::ast::Sentencia;
    let tokens = umbral_lexer::analizar(codigo);
    let programa = umbral_parser::parsear_programa(tokens).unwrap_or_else(|e| {
        eprintln!("Error al parsear: {}", e);
        process::exit(1);
    });
    let impresos = imprimir_documentos(&programa);
    if impresos == 0 {
        println!("(sin documentación umdocs en este archivo)");
        println!("Usa bloques `!!$ ... $!!` antes de `f:` / métodos / `cs:`.");
    }
}

fn imprimir_documentos(programa: &umbral_parser::ast::Programa) -> usize {
    programa
        .sentencias
        .iter()
        .map(imprimir_sentencia)
        .sum()
}

fn imprimir_sentencia(sent: &umbral_parser::ast::Sentencia) -> usize {
    match sent {
        umbral_parser::ast::Sentencia::Funcion(f) => imprimir_funcion(f),
        umbral_parser::ast::Sentencia::Clase(c) => imprimir_clase(c),
        _ => 0,
    }
}

fn imprimir_funcion(func: &umbral_parser::ast::DeclaracionFuncion) -> usize {
    let doc = func.doc.as_ref();
    if doc.is_none() {
        return 0;
    }
    println!("f: {}", firma_funcion(func));
    println!("{}", doc.unwrap().formatear(&func.nombre, ""));
    println!();
    1
}

fn firma_funcion(func: &umbral_parser::ast::DeclaracionFuncion) -> String {
    let partes: Vec<String> = func
        .parametros
        .iter()
        .map(firma_param)
        .collect();
    format!("{}({})", func.nombre, partes.join(", "))
}

fn firma_param(param: &umbral_parser::ast::Parametro) -> String {
    match &param.tipo {
        Some(t) => format!("{}->{}", param.nombre, t.nombre),
        None => param.nombre.clone(),
    }
}

fn imprimir_clase(clase: &umbral_parser::ast::DeclaracionClase) -> usize {
    let mut total = 0;
    if let Some(d) = &clase.doc {
        println!("cs: {}", clase.nombre);
        println!("{}", d.formatear(&clase.nombre, ""));
        println!();
        total += 1;
    }
    total + clase.metodos.iter().map(|m| imprimir_metodo(clase, m)).sum::<usize>()
}

fn imprimir_metodo(clase: &umbral_parser::ast::DeclaracionClase, metodo: &umbral_parser::ast::Metodo) -> usize {
    let doc = metodo.doc.as_ref();
    if doc.is_none() {
        return 0;
    }
    let param_nombres: Vec<String> = metodo.parametros.iter().map(|p| p.nombre.clone()).collect();
    let firma = format!("({})", param_nombres.join(", "));
    let nombre = format!("{}.{}", clase.nombre, metodo.nombre);
    println!("  f: {}.{}", clase.nombre, metodo.nombre);
    println!("{}", doc.unwrap().formatear(&nombre, &firma));
    println!();
    1
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

async fn ejecutar_codigo(codigo: &str, ruta_archivo: &str) {
    let mut interprete = Interpreter::nuevo();
    if let Ok(absoluta) = fs::canonicalize(ruta_archivo) {
        if let Some(dir) = absoluta.parent() {
            interprete.establecer_directorio_base(dir.to_path_buf());
        }
    }
    if let Err(e) = interprete.ejecutar(codigo).await {
        eprintln!("Error de ejecución:");
        eprintln!("{}", e);
        process::exit(1);
    }
}
