# umbral-cli

Ejecutor de archivos `.um` desde línea de comandos.

## Propósito

Permite ejecutar programas escritos en Umbral desde la terminal, similar a `python`, `php`, o `node`.

## Instalación

```bash
cargo install --path .
```

Después de instalar, el comando `umbral` estará disponible globalmente.

## Uso

```bash
umbral archivo.um
```

### Ejemplos

```bash
# Ejecutar un archivo
umbral main.um

# Ejecutar con ruta relativa
umbral codigo-ejemplo/main.um

# Ejecutar con ruta absoluta
umbral /ruta/completa/al/archivo.um
```

## Arquitectura

```rust
CLI
├── Leer argumentos
├── Leer archivo
└── umbral-interpreter
    └── Lexer → Parser → Runtime
```

## Código fuente

```rust
use umbral_interpreter::Interpreter;
use std::fs;

fn main() {
    let ruta = obtener_ruta_archivo();
    let codigo = leer_archivo(ruta);
    ejecutar_codigo(&codigo);
}
```

## Manejo de errores

- **Archivo no encontrado**: Mensaje claro con la ruta
- **Error de lectura**: Detalles del error de I/O
- **Error de ejecución**: Mensaje del interpreter con tipo de error

## Salida

El CLI imprime directamente a stdout/stderr:

```bash
umbral test.um
# Output directo en la terminal
```

## Exit codes

- `0` - Ejecución exitosa
- `1` - Error (archivo no encontrado, error de parsing, error de runtime)

## Desarrollo

```bash
# Compilar
cargo build -p umbral-cli

# Ejecutar en desarrollo
cargo run --bin umbral -- archivo.um

# Compilar release
cargo build --release -p umbral-cli

# El binario estará en: target/release/umbral
```

## Integración con el sistema

Después de `cargo install`:
- Linux/macOS: `~/.cargo/bin/umbral`
- Windows: `%USERPROFILE%\.cargo\bin\umbral.exe`

Asegúrate de que `.cargo/bin` esté en tu PATH.
