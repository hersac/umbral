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
        let prompt = if buffer_multilinea.is_empty() {
            PROMPT
        } else {
            PROMPT_MULTILINE
        };
        
        match leer_linea(&mut editor, prompt) {
            Ok(linea) => {
                if manejar_comando_especial(&linea, &mut interprete, &mut buffer_multilinea) {
                    continue;
                }
                
                buffer_multilinea.push_str(&linea);
                buffer_multilinea.push('\n');
                
                if expresion_completa(&buffer_multilinea) {
                    ejecutar_codigo(&mut interprete, &buffer_multilinea);
                    buffer_multilinea.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                buffer_multilinea.clear();
            }
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
    let linea_trimmed = linea.trim();
    
    match linea_trimmed {
        ":help" => {
            mostrar_ayuda();
            true
        }
        ":clear" => {
            interprete.reiniciar();
            buffer.clear();
            println!("✓ Estado del intérprete reiniciado");
            true
        }
        ":exit" | ":quit" => {
            println!("Adiós!");
            std::process::exit(0);
        }
        _ if linea_trimmed.starts_with(':') => {
            println!("Comando desconocido: {}", linea_trimmed);
            println!("Usa :help para ver comandos disponibles");
            true
        }
        _ => false,
    }
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
    let codigo_trimmed = codigo.trim();
    
    if codigo_trimmed.is_empty() {
        return false;
    }
    
    // Verificar comandos simples que terminan con ;
    if codigo_trimmed.ends_with(';') {
        return cuenta_balanceada(codigo_trimmed);
    }
    
    // Verificar bloques que terminan con }
    if codigo_trimmed.ends_with('}') {
        return cuenta_balanceada(codigo_trimmed);
    }
    
    false
}

fn cuenta_balanceada(codigo: &str) -> bool {
    let mut llaves = 0;
    let mut parentesis = 0;
    let mut corchetes = 0;
    let mut en_string = false;
    let mut en_string_triple = false;
    let mut chars = codigo.chars().peekable();
    
    while let Some(c) = chars.next() {
        // Manejar strings triples
        if c == '\'' && chars.peek() == Some(&'\'') {
            chars.next();
            if chars.peek() == Some(&'\'') {
                chars.next();
                en_string_triple = !en_string_triple;
            }
            continue;
        }
        
        if en_string_triple {
            continue;
        }
        
        // Manejar strings simples
        if c == '"' || c == '\'' {
            en_string = !en_string;
            continue;
        }
        
        if en_string {
            continue;
        }
        
        // Contar delimitadores
        match c {
            '{' => llaves += 1,
            '}' => llaves -= 1,
            '(' => parentesis += 1,
            ')' => parentesis -= 1,
            '[' => corchetes += 1,
            ']' => corchetes -= 1,
            _ => {}
        }
    }
    
    llaves == 0 && parentesis == 0 && corchetes == 0
}

fn ejecutar_codigo(interprete: &mut Interpreter, codigo: &str) {
    match interprete.ejecutar(codigo) {
        Ok(()) => {},
        Err(e) => eprintln!("✗ {}", e),
    }
}
