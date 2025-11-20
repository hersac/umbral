# umbral-lexer

Analizador léxico (tokenizador) para el lenguaje Umbral.

## Propósito

Convierte código fuente de Umbral en una secuencia de tokens que el parser puede entender.

## Uso

```rust
use umbral_lexer::analizar;

let codigo = "v: x = 10;";
let tokens = analizar(codigo);

for token in tokens {
    println!("{:?}", token);
}
```

## Tokens soportados

### Palabras clave
- `v:` - Variable
- `c:` - Constante
- `f:` - Función
- `cs:` - Clase
- `pr:` - Propiedad privada
- `pu:` - Público
- `in:` - Interfaz
- `imp:` - Implementa
- `em:` - Enum
- `i:` - If
- `ie:` - Else if
- `e:` - Else
- `sw:` - Switch
- `ca:` - Case
- `def:` - Default
- `fo:` - For
- `fe:` - ForEach
- `wh:` - While
- `dw:` - Do-While
- `r:` - Return
- `th` - This
- `n:` - New
- `tprint` - Print

### Tipos de datos
- `Int`, `Str`, `Flo`, `Bool`

### Operadores
- Aritméticos: `+`, `-`, `*`, `/`, `%`
- Comparación: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Lógicos: `&&`, `||`, `!`
- Asignación: `=`
- Incremento/Decremento: `++`, `--`
- Acceso: `.`, `=>`

### Delimitadores
- `{`, `}` - Bloques
- `(`, `)` - Expresiones
- `[`, `]` - Arrays/Objetos
- `;` - Fin de sentencia
- `,` - Separador
- `:` - Tipo/Declaración

### Literales
- Enteros: `123`, `-45`
- Flotantes: `3.14`, `-0.5`
- Strings: `'texto'`, `"texto con &interpolacion"`, `'''multilinea'''`
- Booleanos: `true`, `false`
- Nulo: `null`

### Comentarios
- `!!` - Comentario de línea

## Ejemplo completo

```rust
use umbral_lexer::analizar;

fn main() {
    let codigo = r#"
        v: nombre = 'Umbral';
        v: version = 1.0;
        
        f: saludar(msg->Str) {
            tprint("Hola &msg");
        }
    "#;
    
    let tokens = analizar(codigo);
    println!("Tokens generados: {}", tokens.len());
}
```

## Estructura del Token

```rust
pub struct Token {
    pub tipo: TipoToken,
    pub lexema: String,
    pub linea: usize,
}
```

## Algoritmo

1. Recorre el código fuente carácter por carácter
2. Identifica patrones (palabras clave, operadores, literales)
3. Genera tokens con tipo, lexema y posición
4. Ignora whitespace y comentarios
