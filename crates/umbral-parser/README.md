# umbral-parser

Analizador sintáctico que convierte tokens en un Árbol de Sintaxis Abstracta (AST).

## Propósito

Toma la secuencia de tokens del lexer y construye una estructura de árbol que representa la sintaxis del programa.

## Uso

```rust
use umbral_lexer::analizar;
use umbral_parser::Parser;

let codigo = "v: x = 10 + 5;";
let tokens = analizar(codigo);
let mut parser = Parser::nuevo(tokens);

match parser.parsear_programa() {
    Ok(programa) => println!("AST: {:?}", programa),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

## Estructura del AST

### Programa

```rust
pub struct Programa {
    pub sentencias: Vec<Sentencia>,
}
```

### Sentencias

- `DeclaracionVariable` - Declaración de variable
- `DeclaracionConstante` - Declaración de constante
- `Asignacion` - Asignación de valor
- `DeclaracionFuncion` - Definición de función
- `DeclaracionClase` - Definición de clase
- `LlamadoFuncion` - Llamada a función
- `LlamadoTPrint` - Llamada a tprint
- `If` - Condicional if/else
- `Switch` - Switch/case
- `For` - Bucle for
- `ForEach` - Bucle foreach
- `While` - Bucle while
- `DoWhile` - Bucle do-while
- `Return` - Retorno de función
- `Expresion` - Expresión evaluable

### Expresiones

- `LiteralEntero` - Número entero
- `LiteralFloat` - Número flotante
- `LiteralCadena` - String
- `LiteralBool` - Booleano
- `LiteralNulo` - null
- `Identificador` - Nombre de variable
- `Binaria` - Operación binaria (a + b)
- `Unaria` - Operación unaria (!a, -a)
- `Spread` - Operador spread (&array) para expandir arrays
- `Agrupada` - Expresión entre paréntesis
- `Array` - Array literal
- `Objeto` - Objeto literal
- `Instanciacion` - Creación de instancia
- `AccesoPropiedad` - Acceso a propiedad (obj.prop)
- `AccesoIndice` - Acceso por índice (arr[0])

## Ejemplo completo

```rust
use umbral_lexer::analizar;
use umbral_parser::Parser;
use umbral_parser::ast::*;

fn main() {
    let codigo = r#"
        f: sumar(a->Int, b->Int)->Int {
            r: (a + b);
        }
        
        v: resultado = sumar(5, 10);
        tprint(resultado);
    "#;
    
    let tokens = analizar(codigo);
    let mut parser = Parser::nuevo(tokens);
    
    match parser.parsear_programa() {
        Ok(programa) => {
            println!("Sentencias parseadas: {}", programa.sentencias.len());
            for sentencia in programa.sentencias {
                println!("  - {:?}", sentencia);
            }
        }
        Err(e) => eprintln!("Error de parser: {:?}", e),
    }
}
```

## Gramática soportada

### Declaraciones

```
Variable    ::= "v:" Identificador ["->Tipo"] "=" Expresion ";"
Constante   ::= "c:" Identificador ["->Tipo"] "=" Expresion ";"
Funcion     ::= "f:" Identificador "(" Parametros ")" ["->" Tipo] Bloque
Clase       ::= "cs:" Identificador Bloque
```

### Control de flujo

```
If          ::= "i:" "(" Expresion ")" Bloque ["ie:" "(" Expresion ")" Bloque]* ["e:" Bloque]
Switch      ::= "sw:" "(" Expresion ")" "{" Case* [Default] "}"
For         ::= "fo:" "(" Sentencia ";" Expresion ";" Expresion ")" Bloque
ForEach     ::= "fe:" "(" "v:" Identificador "<=" Expresion ")" Bloque
While       ::= "wh:" "(" Expresion ")" Bloque
DoWhile     ::= "dw:" Bloque "(" Expresion ")"
```

### Expresiones

```
Expresion   ::= Logica
Logica      ::= Comparacion ( ("&&" | "||") Comparacion )*
Comparacion ::= Suma ( ("==" | "!=" | "<" | ">" | "<=" | ">=") Suma )*
Suma        ::= Producto ( ("+" | "-") Producto )*
Producto    ::= Unaria ( ("*" | "/" | "%") Unaria )*
Unaria      ::= ("!" | "-" | "++" | "--") Unaria | Llamada
Llamada     ::= Primaria ( "(" Argumentos ")" | "." Identificador | "[" Expresion "]" )*
Primaria    ::= Numero | String | Boolean | Null | Identificador | "(" Expresion ")"
```

## Manejo de errores

```rust
pub enum ParserError {
    TokenInesperado { esperado: String, encontrado: String },
    FinInesperado,
    ExpresionInvalida,
}
```

## Características

- ✅ Análisis sintáctico recursivo descendente
- ✅ Manejo de precedencia de operadores
- ✅ Soporte para expresiones anidadas
- ✅ Validación de sintaxis
- ✅ Mensajes de error descriptivos
- ✅ AST completo y tipado
