# umbral-interpreter

Capa de coordinación que integra lexer, parser y runtime en una API unificada.

## Propósito

Proporciona una interfaz simple para ejecutar código Umbral, ocultando la complejidad de las fases de compilación.

## Uso básico

```rust
use umbral_interpreter::Interpreter;

let mut interprete = Interpreter::nuevo();

match interprete.ejecutar("v: x = 10; tprint(x);") {
    Ok(()) => println!("Ejecutado correctamente"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Arquitectura

```
Interpreter
├── tokenizar()  → Vec<Token>        (Lexer)
├── parsear()    → Programa          (Parser)
└── evaluar()    → ()                (Runtime)
```

## API pública

### `Interpreter::nuevo()`

Crea una nueva instancia con runtime limpio.

```rust
let mut interprete = Interpreter::nuevo();
```

### `ejecutar(&mut self, codigo: &str)`

Ejecuta código fuente completo.

```rust
interprete.ejecutar("v: x = 42;")?;
```

### `ejecutar_con_resultado(&mut self, codigo: &str)`

Ejecuta código y retorna resultado (útil para REPL).

```rust
let resultado = interprete.ejecutar_con_resultado("v: suma = 5 + 10;")?;
```

### `reiniciar(&mut self)`

Limpia todo el estado del intérprete.

```rust
interprete.reiniciar();
```

## Sistema de errores

```rust
pub enum InterpreterError {
    LexerError(String),      // Error de tokenización
    ParserError(String),     // Error sintáctico
    RuntimeError(String),    // Error de ejecución
    IoError(String),         // Error de I/O
}
```

Ejemplo de manejo:

```rust
use umbral_interpreter::{Interpreter, InterpreterError};

let mut interprete = Interpreter::nuevo();

match interprete.ejecutar("codigo invalido") {
    Ok(()) => println!("Éxito"),
    Err(InterpreterError::LexerError(msg)) => eprintln!("Lexer: {}", msg),
    Err(InterpreterError::ParserError(msg)) => eprintln!("Parser: {}", msg),
    Err(InterpreterError::RuntimeError(msg)) => eprintln!("Runtime: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Ejemplo completo

```rust
use umbral_interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interprete = Interpreter::nuevo();
    
    // Ejecutar código
    interprete.ejecutar(r#"
        v: nombre = "Umbral";
        v: version = 1.0;
        tprint("Lenguaje: &nombre v&version");
        
        f: factorial(n->Int)->Int {
            i: (n <= 1) {
                r: (1);
            } e: {
                r: (n * factorial(n - 1));
            }
        }
        
        tprint(factorial(5));
    "#)?;
    
    Ok(())
}
```

## Uso en CLI

```rust
use umbral_interpreter::Interpreter;
use std::fs;

fn ejecutar_archivo(ruta: &str) -> Result<(), Box<dyn std::error::Error>> {
    let codigo = fs::read_to_string(ruta)?;
    let mut interprete = Interpreter::nuevo();
    interprete.ejecutar(&codigo)?;
    Ok(())
}
```

## Uso en REPL

```rust
use umbral_interpreter::Interpreter;

fn repl() {
    let mut interprete = Interpreter::nuevo();
    
    loop {
        let linea = leer_entrada();
        
        match linea.as_str() {
            ":clear" => {
                interprete.reiniciar();
                println!("Estado reiniciado");
            }
            ":exit" => break,
            _ => {
                if let Err(e) = interprete.ejecutar(&linea) {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}
```

## Características

- ✅ API simple y consistente
- ✅ Manejo de errores tipado
- ✅ Estado persistente entre ejecuciones
- ✅ Capacidad de reinicio
- ✅ Reutilizable entre CLI y REPL
- ✅ Tests unitarios incluidos

## Testing

```bash
cargo test -p umbral-interpreter
```

Tests incluidos:
- Variables y constantes
- Operaciones aritméticas
- Definición de funciones
- Código vacío/inválido
- Reinicio de estado

## Dependencias

- `umbral-lexer` - Tokenización
- `umbral-parser` - Análisis sintáctico  
- `umbral-runtime` - Ejecución
