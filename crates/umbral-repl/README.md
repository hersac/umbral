# umbral-repl

REPL interactivo (Read-Eval-Print Loop) para ejecutar código Umbral línea por línea.

## Propósito

Proporciona un entorno interactivo para experimentar con Umbral, similar a `python`, `node`, o `irb`.

## Instalación

```bash
cargo install --path .
```

## Uso

```bash
umbral-repl
```

## Características

### ✅ Entrada multilínea inteligente

El REPL detecta automáticamente expresiones incompletas:

```
umbral> f: sumar(a, b) {
     ->     r: (a + b);
     -> }
```

### ✅ Historial de comandos

- `↑` / `↓` - Navegar por comandos anteriores
- Rustyline maneja el historial automáticamente

### ✅ Estado persistente

Las variables persisten entre comandos:

```
umbral> v: contador = 0;
umbral> contador++;
umbral> tprint(contador);
1
```

### ✅ Comandos especiales

| Comando | Descripción |
|---------|-------------|
| `:help` | Muestra ayuda |
| `:clear` | Reinicia el intérprete |
| `:exit` / `:quit` | Sale del REPL |

### ✅ Manejo de interrupciones

- `Ctrl+C` - Cancela entrada actual
- `Ctrl+D` - Sale del REPL

## Ejemplo de sesión

```
╔════════════════════════════════════════╗
║     Umbral REPL v1.1.5                 ║
║     Lenguaje de Programación Umbral   ║
╚════════════════════════════════════════╝

umbral> v: x = 10;
umbral> v: y = 20;
umbral> tprint(x + y);
30
umbral> f: factorial(n->Int)->Int {
     ->     i: (n <= 1) {
     ->         r: (1);
     ->     } e: {
     ->         r: (n * factorial(n - 1));
     ->     }
     -> }
umbral> tprint(factorial(5));
120
umbral> :exit
Adiós!
```

## Detección de completitud

El REPL usa un algoritmo inteligente para detectar si una expresión está completa:

1. Verifica terminadores (`;`, `}`)
2. Cuenta balance de delimitadores (`{}`, `()`, `[]`)
3. Considera strings y comentarios

```rust
fn expresion_completa(codigo: &str) -> bool {
    // Termina con ; o }
    // Y todos los delimitadores están balanceados
}
```

## Arquitectura

```
REPL
├── Rustyline (editor de línea)
├── Buffer multilínea
├── Detector de completitud
└── umbral-interpreter
```

## Casos de uso

### 1. Aprendizaje

Experimenta con sintaxis sin crear archivos:

```
umbral> v: lista = {1, 2, 3};
umbral> tprint(lista[0]);
1
```

### 2. Debugging

Prueba funciones rápidamente:

```
umbral> f: esPar(n) { r: (n % 2 == 0); }
umbral> tprint(esPar(4));
true
```

### 3. Calculadora

Usa como calculadora con estado:

```
umbral> v: precio = 100;
umbral> v: iva = precio * 0.16;
umbral> tprint(precio + iva);
116
```

### 4. Prototipado

Desarrolla lógica antes del código final:

```
umbral> v: datos = {10, 20, 30};
umbral> v: suma = 0;
umbral> fe: (v: n <= datos) { suma = suma + n; }
umbral> tprint(suma);
60
```

## Comandos especiales detallados

### `:help`

Muestra ayuda completa con ejemplos de sintaxis.

### `:clear`

Reinicia el intérprete, limpiando todas las variables y funciones:

```
umbral> v: x = 100;
umbral> :clear
✓ Estado del intérprete reiniciado
umbral> tprint(x);
✗ Variable 'x' no encontrada
```

### `:exit` / `:quit`

Sale del REPL limpiamente.

## Desarrollo

```bash
# Compilar
cargo build -p umbral-repl

# Ejecutar en desarrollo
cargo run --bin umbral-repl

# Compilar release
cargo build --release -p umbral-repl
```

## Dependencias

- **rustyline** (v14.0): Editor de línea con historial
- **umbral-interpreter**: Motor de ejecución

## Testing

Tests automatizados en `test_repl.sh`:

```bash
./test_repl.sh
```

Prueba:
- Variables simples
- Constantes
- Comando :clear
- Interpolación de strings
