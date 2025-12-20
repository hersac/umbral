# umbral-runtime

Motor de ejecución del lenguaje Umbral. Interpreta y ejecuta el AST generado por el parser.

## Propósito

Ejecuta el código Umbral evaluando el AST, manejando variables, funciones, clases y control de flujo.

## Uso

```rust
use umbral_lexer::analizar;
use umbral_parser::Parser;
use umbral_runtime::Runtime;

let codigo = "v: x = 10; tprint(x);";
let tokens = analizar(codigo);
let mut parser = Parser::nuevo(tokens);
let programa = parser.parsear_programa().unwrap();

let mut runtime = Runtime::nuevo();
runtime.ejecutar(programa);
```

## Arquitectura

```
Runtime
└── Interpretador
    ├── Entorno (variables y constantes)
    ├── GestorClases (definiciones de clases)
    ├── GestorFunciones (ejecución de funciones)
    └── Sistema de valores
```

## Módulos

### `valores.rs`

Define todos los tipos de valores en Umbral:

```rust
pub enum Valor {
    Entero(i64),
    Flotante(f64),
    Texto(String),
    Booleano(bool),
    Lista(Vec<Valor>),
    Diccionario(HashMap<String, Valor>),
    Objeto(Instancia),
    Funcion(Funcion),
    Nulo,
}
```

### `entorno.rs`

Gestiona el scope de variables y constantes:

```rust
pub struct Entorno {
    variables: HashMap<String, Valor>,
    constantes: HashSet<String>,
    padre: Option<Box<Entorno>>,
}
```

Características:
- Scope anidado con cadena de padres
- Protección de constantes
- Búsqueda en cascada

### `clases.rs`

Maneja definiciones de clases e instancias:

```rust
pub struct Clase {
    pub nombre: String,
    pub propiedades: HashMap<String, Valor>,
    pub metodos: HashMap<String, Metodo>,
    pub constructor: Option<Metodo>,
}
```

### `funciones.rs`

Ejecuta funciones con binding de parámetros:

```rust
pub struct GestorFunciones;

impl GestorFunciones {
    pub fn ejecutar_funcion(
        func: &Funcion,
        argumentos: Vec<Valor>,
        interprete: &mut Interpretador,
    ) -> Valor;
}
```

### `interpretador.rs`

Núcleo de ejecución del runtime:

**Métodos principales:**
- `ejecutar_sentencia()` - Dispatcher de sentencias
- `evaluar_expresion()` - Evaluador de expresiones
- `ejecutar_bloque()` - Ejecución de bloques con scope

**Operaciones:**
- Aritméticas: `+`, `-`, `*`, `/`, `%`
- Comparación: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Lógicas: `&&`, `||`, `!`
- Incremento/Decremento: `++`, `--`

**Control de flujo:**
- `ejecutar_if()` - If/else if/else
- `ejecutar_switch()` - Switch/case/default
- `ejecutar_for()` - Bucle for
- `ejecutar_foreach()` - Bucle foreach
- `ejecutar_while()` - Bucle while
- `ejecutar_do_while()` - Bucle do-while
- `ejecutar_try_catch()` - Manejo de excepciones
- `ejecutar_throw()` - Lanzamiento de errores

**Funcionalidades especiales:**
- Interpolación de strings con `&variable`
- Acceso a propiedades con `.`
- Acceso a índices con `[]`
- Instanciación de clases
- Llamadas a métodos

## Ejemplo completo

```rust
use umbral_lexer::analizar;
use umbral_parser::Parser;
use umbral_runtime::Runtime;

fn main() {
    let codigo = r#"
        v: contador = 0;
        
        wh: (contador < 5) {
            tprint(contador);
            contador++;
        }
        
        f: factorial(n->Int)->Int {
            i: (n <= 1) {
                r: (1);
            } e: {
                r: (n * factorial(n - 1));
            }
        }
        
        tprint(factorial(5));
    "#;
    
    let tokens = analizar(codigo);
    let mut parser = Parser::nuevo(tokens);
    let programa = parser.parsear_programa().unwrap();
    
    let mut runtime = Runtime::nuevo();
    runtime.ejecutar(programa);
}
```

## Características implementadas

### Variables y constantes
- ✅ Declaración con tipos opcionales
- ✅ Asignación y reasignación
- ✅ Protección de constantes
- ✅ Scope léxico

### Funciones
- ✅ Declaración con parámetros tipados
- ✅ Retorno de valores
- ✅ Recursión
- ✅ Closure sobre scope

### Clases
- ✅ Declaración con propiedades
- ✅ Constructor
- ✅ Métodos
- ✅ Instanciación con `n:`
- ✅ Acceso a propiedades con `.`

### Operadores
- ✅ Aritméticos con promoción de tipos
- ✅ Comparación con coerción
- ✅ Lógicos con evaluación cortocircuito
- ✅ Incremento/Decremento

### Control de flujo
- ✅ If/else if/else
- ✅ Switch/case/default
- ✅ For con inicialización
- ✅ ForEach sobre iterables
- ✅ While
- ✅ Do-While
- ✅ Try-Catch-Finally
- ✅ Throw

### Manejo de errores
- ✅ Captura de excepciones
- ✅ Propagación de errores
- ✅ Clase Error nativa

### Estructuras de datos
- ✅ Arrays con acceso por índice
- ✅ Objetos/Diccionarios
- ✅ Propiedad `.length` en arrays
- ✅ Operador spread (`&`) para expandir arrays

### Strings
- ✅ Strings simples (`'texto'`)
- ✅ Strings con interpolación (`"Hola &nombre"`)
- ✅ Strings multilínea (`'''texto'''`)

### Funciones built-in
- ✅ `tprint()` - Impresión en consola

## Clean Code

El runtime está implementado siguiendo principios de código limpio:
- ❌ Sin if-else anidados
- ✅ Uso de early returns
- ✅ Funciones pequeñas con responsabilidad única
- ✅ Nombres descriptivos
- ✅ Extracción de métodos
- ✅ Uso de operadores funcionales (map, and_then)

## Performance

- Interpretación directa del AST
- Gestión eficiente de scope con HashMap
- Clonación mínima de valores
