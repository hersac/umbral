use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use umbral_interpreter::Interpreter;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROMPT: &str = "umbral> ";
const PROMPT_MULTILINE: &str = "     -> ";

fn main() {
    mostrar_banner();
    
    let mut interprete = Interpreter::nuevo();
    let mut editor = crear_editor();
    let mut buffer_multilinea = String::new();
    
    loop {
        let prompt = obtener_prompt(&buffer_multilinea);
        
        match leer_linea(&mut editor, prompt) {
            Ok(linea) => procesar_entrada(&mut interprete, &mut buffer_multilinea, linea),
            Err(ReadlineError::Interrupted) => manejar_interrupcion(&mut buffer_multilinea),
            Err(ReadlineError::Eof) => {
                println!("Adiós!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn obtener_prompt(buffer: &str) -> &'static str {
    if buffer.is_empty() {
        PROMPT
    } else {
        PROMPT_MULTILINE
    }
}

fn procesar_entrada(interprete: &mut Interpreter, buffer: &mut String, linea: String) {
    if manejar_comando_especial(&linea, interprete, buffer) {
        return;
    }
    
    buffer.push_str(&linea);
    buffer.push('\n');
    
    if expresion_completa(buffer) {
        ejecutar_codigo(interprete, buffer);
        buffer.clear();
    }
}

fn manejar_interrupcion(buffer: &mut String) {
    println!("^C");
    buffer.clear();
}

fn mostrar_banner() {
    println!("╔════════════════════════════════════════╗");
    println!("║     Umbral REPL v{}                  ║", VERSION);
    println!("║     Lenguaje de Programación Umbral   ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    println!("Comandos especiales:");
    println!("  :help    - Muestra esta ayuda");
    println!("  :clear   - Limpia el estado del intérprete");
    println!("  :exit    - Sale del REPL");
    println!("  Ctrl+C   - Cancela entrada actual");
    println!("  Ctrl+D   - Sale del REPL");
    println!();
}

fn crear_editor() -> DefaultEditor {
    DefaultEditor::new().expect("No se pudo crear el editor")
}

fn leer_linea(editor: &mut DefaultEditor, prompt: &str) -> Result<String, ReadlineError> {
    let linea = editor.readline(prompt)?;
    editor.add_history_entry(linea.as_str())?;
    Ok(linea)
}

fn manejar_comando_especial(
    linea: &str,
    interprete: &mut Interpreter,
    buffer: &mut String,
) -> bool {
    let linea_limpia = linea.trim();
    
    if !linea_limpia.starts_with(':') {
        return false;
    }
    
    ejecutar_comando(linea_limpia, interprete, buffer);
    true
}

fn ejecutar_comando(comando: &str, interprete: &mut Interpreter, buffer: &mut String) {
    match comando {
        ":help" => mostrar_ayuda(),
        ":clear" => reiniciar_interprete(interprete, buffer),
        ":exit" | ":quit" => salir_repl(),
        _ => mostrar_comando_desconocido(comando),
    }
}

fn reiniciar_interprete(interprete: &mut Interpreter, buffer: &mut String) {
    interprete.reiniciar();
    buffer.clear();
    println!("✓ Estado del intérprete reiniciado");
}

fn salir_repl() -> ! {
    println!("Adiós!");
    std::process::exit(0);
}

fn mostrar_comando_desconocido(comando: &str) {
    println!("Comando desconocido: {}", comando);
    println!("Usa :help para ver comandos disponibles");
}

fn mostrar_ayuda() {
    println!();
    println!("═══════════════════════════════════════════════════");
    println!("  AYUDA - Umbral REPL");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("Comandos especiales:");
    println!("  :help           Muestra esta ayuda");
    println!("  :clear          Reinicia el intérprete (limpia variables)");
    println!("  :exit, :quit    Sale del REPL");
    println!();
    println!("Atajos de teclado:");
    println!("  Ctrl+C          Cancela la entrada actual");
    println!("  Ctrl+D          Sale del REPL");
    println!();
    println!("Sintaxis Umbral:");
    println!("  v: nombre = valor;              Variable");
    println!("  c: NOMBRE = valor;              Constante");
    println!("  f: nombre(params) {{ ... }}       Función");
    println!("  tprint(valor);                  Imprimir");
    println!();
    println!("Ejemplo:");
    println!("  umbral> v: x = 10;");
    println!("  umbral> v: y = 20;");
    println!("  umbral> tprint(x + y);");
    println!("  30");
    println!();
    println!("═══════════════════════════════════════════════════");
    println!();
}

fn expresion_completa(codigo: &str) -> bool {
    let codigo_limpio = codigo.trim();
    
    if codigo_limpio.is_empty() {
        return false;
    }
    
    if !termina_correctamente(codigo_limpio) {
        return false;
    }
    
    cuenta_balanceada(codigo_limpio)
}

fn termina_correctamente(codigo: &str) -> bool {
    codigo.ends_with(';') || codigo.ends_with('}')
}

fn procesar_string_triple(chars: &mut std::iter::Peekable<std::str::Chars>, en_triple: &mut bool) -> bool {
    if chars.peek() == Some(&'\'') {
        chars.next();
        if chars.peek() == Some(&'\'') {
            chars.next();
            *en_triple = !*en_triple;
            return true;
        }
    }
    false
}

fn actualizar_balances(caracter: char, llaves: &mut i32, parentesis: &mut i32, corchetes: &mut i32) {
    match caracter {
        '{' => *llaves += 1,
        '}' => *llaves -= 1,
        '(' => *parentesis += 1,
        ')' => *parentesis -= 1,
        '[' => *corchetes += 1,
        ']' => *corchetes -= 1,
        _ => {}
    }
}

fn cuenta_balanceada(codigo: &str) -> bool {
    let mut llaves = 0;
    let mut parentesis = 0;
    let mut corchetes = 0;
    let mut en_string = false;
    let mut en_string_triple = false;
    let mut chars = codigo.chars().peekable();
    
    while let Some(caracter) = chars.next() {
        if caracter == '\'' && procesar_string_triple(&mut chars, &mut en_string_triple) {
            continue;
        }
        
        if en_string_triple {
            continue;
        }
        
        if caracter == '"' || caracter == '\'' {
            en_string = !en_string;
            continue;
        }
        
        if en_string {
            continue;
        }
        
        actualizar_balances(caracter, &mut llaves, &mut parentesis, &mut corchetes);
    }
    
    llaves == 0 && parentesis == 0 && corchetes == 0
}

fn ejecutar_codigo(interprete: &mut Interpreter, codigo: &str) {
    match interprete.ejecutar(codigo) {
        Ok(()) => {},
        Err(e) => eprintln!("✗ {}", e),
    }
}
