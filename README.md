# Umbral

**Versión 1.1.5**

Lenguaje de programación de propósito general con sintaxis expresiva y moderna. Diseñado para ser fácil de aprender y productivo de usar.

---

## 📋 Tabla de contenidos

- [Características](#-características)
- [Instalación rápida](#-instalación-rápida)
- [Inicio rápido](#-inicio-rápido)
- [Sintaxis](#-sintaxis)
- [Arquitectura](#-arquitectura)
- [Uso](#-uso)
- [Desarrollo](#-desarrollo)
- [Documentación de módulos](#-documentación-de-módulos)
- [Tests](#-tests)
- [Roadmap](#-roadmap)
- [Contribuir](#-contribuir)
- [Licencia](#-licencia)

---

## ✨ Características

- 🎯 **Sintaxis clara y concisa** - Fácil de leer y escribir
- 🔄 **Tipado dinámico con anotaciones opcionales** - Flexibilidad sin sacrificar claridad
- 🏗️ **Programación orientada a objetos** - Clases, herencia e interfaces
- 🔧 **Funciones de primera clase** - Funciones como valores, closures
- 🎨 **Interpolación de strings** - Sintaxis natural con `&variable`
- 🔄 **Control de flujo completo** - if/else, switch, for, foreach, while, do-while
- 🚀 **REPL interactivo** - Experimenta en tiempo real
- 📦 **CLI potente** - Ejecuta archivos con `umbral archivo.um`

---

## 🚀 Instalación rápida

### Requisitos

- [Rust](https://rustup.rs/) 1.70 o superior
- Git

### Linux / macOS

```bash
git clone https://github.com/hersac/umbral.git
cd umbral
./install.sh
```

### Windows (PowerShell como Administrador)

```powershell
git clone https://github.com/hersac/umbral.git
cd umbral
PowerShell -ExecutionPolicy Bypass -File install.ps1
```

**Importante**: Cierra y vuelve a abrir tu terminal después de instalar.

> [!WARNING]
> **Conflicto de versiones**: Si previamente instalaste Umbral manualmente con `cargo install`, es posible que esa versión tenga prioridad. Ejecuta `cargo uninstall umbral` antes de usar el instalador para evitar conflictos.

### Verificar instalación

```bash
umbral --version
umbral-repl
```

📖 [Guía de instalación completa](./INSTALL.md)

---

## 🎓 Inicio rápido

### Hola Mundo

Crea un archivo `hola.um`:

```umbral
v: mensaje = "Hola, Mundo!";
tprint(mensaje);
```

Ejecuta:

```bash
umbral hola.um
```

### REPL Interactivo

```bash
umbral-repl
```

```
umbral> v: x = 10;
umbral> v: y = 20;
umbral> tprint(x + y);
30
```

---

## 📚 Sintaxis

### Variables y constantes

```umbral
!! Variables (mutables)
v: nombre = "Heriberto";
v: edad->Int = 30;
v: precio->Flo = 99.99;
v: activo->Bool = true;

!! Constantes (inmutables)
c: PI = 3.14159;
c: MAX_INTENTOS = 3;
```

### Tipos de datos

Umbral soporta **inferencia de tipos** (tipado dinámico) y **anotaciones de tipo opcionales** (tipado fuerte).

```umbral
!! Inferencia de tipos (el tipo se deduce automáticamente)
v: entero = 42;              !! Umbral infiere que es Int
v: flotante = 3.14;          !! Umbral infiere que es Flo
v: texto = 'Hola';           !! Umbral infiere que es Str
v: booleano = true;          !! Umbral infiere que es Bool
v: nulo = null;              !! Umbral infiere que es null

!! Tipado fuerte (especificas el tipo explícitamente)
v: edad->Int = 30;           !! Declaración con tipo Int
v: precio->Flo = 99.99;      !! Declaración con tipo Flo
v: nombre->Str = "María";    !! Declaración con tipo Str
v: activo->Bool = true;      !! Declaración con tipo Bool

!! Tipos disponibles:
!! - Int   (enteros: 42, -10, 0)
!! - Flo   (flotantes: 3.14, -0.5, 2.0)
!! - Str   (strings: 'hola', "mundo")
!! - Bool  (booleanos: true, false)

!! Arrays
v: numeros = {1, 2, 3, 4, 5};                    !! Inferido como array
v: nombres->[]Str = {'Ana', 'Luis', 'María'};    !! Array tipado de strings

!! Operador Spread (&) para expandir arrays
c: arr1 = {1, 2, 3};
c: arr2 = {4, 5, 6};
c: combinado = { &arr1, &arr2 };                 !! Resultado: {1, 2, 3, 4, 5, 6}
c: mixto = { 0, &arr1, 99, &arr2 };              !! Resultado: {0, 1, 2, 3, 99, 4, 5, 6}

!! Objetos/Diccionarios
v: persona = [                                   !! Inferido como objeto
    "nombre" => "Juan",
    "edad" => 25,
    "ciudad" => "Madrid"
];

v: config->Objeto = [                           !! Objeto tipado explícito
    "host" => "localhost",
    "port" => 8080
];
```

### Operadores

```umbral
!! Aritméticos
v: suma = 10 + 5;          !! 15
v: resta = 10 - 5;         !! 5
v: multiplicacion = 10 * 5; !! 50
v: division = 10 / 5;      !! 2
v: modulo = 10 % 3;        !! 1

!! Comparación
v: igual = 10 == 10;       !! true
v: diferente = 10 != 5;    !! true
v: menor = 5 < 10;         !! true
v: mayor = 10 > 5;         !! true

!! Lógicos
v: y = true && false;      !! false
v: o = true || false;      !! true
v: no = !true;             !! false

!! Incremento/Decremento
v: contador = 0;
contador++;                 !! 1
contador--;                 !! 0

!! Spread (expansión de arrays)
v: a = {1, 2};
v: b = {3, 4};
v: c = { &a, &b };         !! {1, 2, 3, 4} - Expande ambos arrays
v: d = a + b;              !! {1, 2, 3, 4} - Concatenación equivalente
```

### Control de flujo

```umbral
!! If / Else if / Else
i: (edad < 18) {
    tprint('Menor de edad');
} ie: (edad < 65) {
    tprint('Adulto');
} e: {
    tprint('Adulto mayor');
}

!! Switch / Case
sw: (opcion) {
    ca: 1 =>
        tprint('Opción 1');
    ca: 2 =>
        tprint('Opción 2');
    def: =>
        tprint('Opción inválida');
}
```

### Bucles

```umbral
!! For
fo: (v: i = 0; i < 10; i++) {
    tprint(i);
}

!! ForEach
v: frutas = {'manzana', 'pera', 'naranja'};
fe: (v: fruta <= frutas) {
    tprint(fruta);
}

!! While
v: contador = 0;
wh: (contador < 5) {
    tprint(contador);
    contador++;
}

!! Do-While
v: numero = 0;
dw: {
    tprint(numero);
    numero++;
} (numero < 5)
```

### Funciones

```umbral
!! Función simple
f: saludar(nombre->Str) {
    tprint("Hola, &nombre!");
}

!! Función con retorno
f: sumar(a->Int, b->Int)->Int {
    r: (a + b);
}

!! Función recursiva
f: factorial(n->Int)->Int {
    i: (n <= 1) {
        r: (1);
    } e: {
        r: (n * factorial(n - 1));
    }
}

!! Uso
saludar("María");
v: resultado = sumar(10, 20);
tprint(factorial(5));
```

### Clases y POO

```umbral
!! Definición de clase
cs: Persona {
    pr: nombre->Str;
    pr: edad->Int;
    
    !! Constructor
    pu f: Persona(nombre->Str, edad->Int) {
        th.nombre = nombre;
        th.edad = edad;
    }
    
    !! Método público
    pu f: presentarse()->Void {
        tprint("Hola, soy &th.nombre y tengo &th.edad años");
    }
    
    !! Método con retorno
    pu f: esMayorDeEdad()->Bool {
        r: (th.edad >= 18);
    }
}

!! Instanciación
c: persona1 = n: Persona("Juan", 25);
persona1.presentarse();

!! Interfaces (in:)
!! Define contratos que las clases deben cumplir
in: Dibujable {
    f: dibujar()->Void;
    f: obtenerColor()->Str;
}

!! Implementar una interfaz (imp:)
cs: Circulo imp: Dibujable {
    pr: radio->Flo;
    pr: color->Str;
    
    pu f: Circulo(radio->Flo, color->Str) {
        th.radio = radio;
        th.color = color;
    }
    
    !! Implementación de métodos de la interfaz
    pu f: dibujar()->Void {
        tprint("Dibujando círculo de radio &th.radio");
    }
    
    pu f: obtenerColor()->Str {
        r: (th.color);
    }
}

!! Herencia (ext:)
!! Una clase puede extender de otra clase base
cs: Empleado ext: Persona {
    pr: salario->Flo;
    pr: puesto->Str;
    
    pu f: Empleado(nombre->Str, edad->Int, puesto->Str, salario->Flo) {
        !! Llamar al constructor de la clase base
        th.nombre = nombre;
        th.edad = edad;
        th.puesto = puesto;
        th.salario = salario;
    }
    
    !! Sobrescribir método heredado
    pu f: presentarse()->Void {
        tprint("Hola, soy &th.nombre, &th.puesto de la empresa");
    }
    
    !! Nuevo método específico
    pu f: obtenerSalario()->Flo {
        r: (th.salario);
    }
}

!! Uso de herencia
c: empleado1 = n: Empleado("Ana", 28, "Desarrolladora", 50000.0);
empleado1.presentarse();                    !! "Hola, soy Ana, Desarrolladora de la empresa"
tprint(empleado1.obtenerSalario());         !! 50000.0
tprint(empleado1.esMayorDeEdad());          !! true (heredado de Persona)

!! Combinación: Herencia + Interfaz
cs: Rectangulo ext: Forma imp: Dibujable {
    pr: ancho->Flo;
    pr: alto->Flo;
    
    !! Implementa métodos de la clase base Forma
    !! Implementa métodos de la interfaz Dibujable
}
```

### Importaciones y Exportaciones

```umbral
!! Archivo: modulos/matematicas.um
!! Solo los elementos con 'out' pueden ser importados

out f: sumar(a->Int, b->Int)->Int {
    r: (a + b);
}

out c: PI = 3.14159;

!! Función privada (sin 'out')
f: funcionInterna()->Int {
    r: (42);
}

!! Archivo: main.um
!! Sintaxis de importación

!! 1. Importación simple
equip sumar origin 'modulos/matematicas.um';

!! 2. Importación con alias
equip sumar as suma origin 'modulos/matematicas.um';

!! 3. Importación de lista
equip { sumar, PI } origin 'modulos/matematicas.um';

!! 4. Importación con namespace
equip * as mat origin 'modulos/matematicas.um';
c: resultado = mat_sumar(10, 5);

!! 5. Orden invertido
origin 'modulos/matematicas.um' equip sumar;
```

### Gestor de Paquetes UMP

Umbral tiene soporte integrado para el gestor de paquetes **UMP** (Umbral Package Manager), que permite instalar y gestionar librerías de forma sencilla.

#### Instalación de librerías

```bash
# Instalar una librería desde el registry
ump add http

# La librería se descarga en modules_ump/
```

#### Importación simplificada

Una vez instalada una librería con UMP, puedes importarla usando solo su nombre:

```umbral
!! Importación simplificada de módulo UMP
equip { fetch } origin 'http';

!! Uso de la librería
c: url = "https://api.ejemplo.com/datos";
c: respuesta = fetch(url);
tprint("Respuesta: &respuesta");
```

**Ventajas:**
- ✅ No necesitas conocer la estructura interna de `modules_ump`
- ✅ Sintaxis consistente con `ump add <nombre>`
- ✅ Búsqueda automática en la jerarquía de directorios
- ✅ Compatible con rutas relativas tradicionales

**Más información:** [UMP Package Manager](https://github.com/hersac/ump)


### Strings e interpolación

```umbral
!! String simple
v: texto1 = 'Hola mundo';

!! String con interpolación
v: nombre = "María";
v: edad = 25;
v: mensaje = "Hola &nombre, tienes &edad años";
tprint(mensaje);

!! String multilínea
v: parrafo = '''
    Este es un texto
    que ocupa múltiples
    líneas.
''';
```

### Acceso a datos

```umbral
!! Arrays
v: numeros = {10, 20, 30};
tprint(numeros[0]);        !! 10
tprint(numeros.length);    !! 3

!! Métodos de arrays
v: lista = {1, 2, 3};
v: nuevaLista = lista.push(4);      !! {1, 2, 3, 4}
v: sinUltimo = lista.pop();         !! {1, 2}
v: longitud = lista.len();          !! 3

!! Encadenamiento de métodos
v: resultado = {1, 2}.push(3).push(4).pop();  !! {1, 2, 3}

!! Objetos
v: config = [
    "host" => "localhost",
    "port" => 8080
];
tprint(config.host);       !! localhost

!! Propiedades de objetos
c: persona = n: Persona("Ana", 30);
tprint(persona.nombre);    !! Ana
persona.setEdad(31);
```

### Comentarios

```umbral
!! Este es un comentario de línea

v: x = 10; !! Comentario al final de línea

!!
!! Bloque de comentarios
!! múltiples líneas
!!
```

---

## 📚 Biblioteca Estándar (StdLib)

Umbral incluye una biblioteca estándar mínima con funciones esenciales accesibles mediante el objeto global `Std`.

### Manipulación de Strings

```umbral
v: texto = "  Hola  ";
v: limpio = Std.trim(texto);               !! "Hola"
v: partes = Std.split("a,b,c", ",");       !! ["a", "b", "c"]
v: nuevo = Std.replace("hola", "o", "0");  !! "h0la"
v: mayus = Std.to_upper("umbral");         !! "UMBRAL"
v: minus = Std.to_lower("UMBRAL");         !! "umbral"
```

### Operaciones Numéricas

```umbral
v: numero = Std.parse_int("42");           !! 42
v: decimal = Std.parse_float("3.14");      !! 3.14
v: texto = Std.to_string(123);             !! "123"
v: aleatorio = Std.random();               !! 0.0 - 1.0
v: absoluto = Std.abs(-10);                !! 10
```

### Sistema de Archivos

```umbral
v: ok = Std.write_file("datos.txt", "contenido");
v: existe = Std.exists("datos.txt");       !! true
v: contenido = Std.read_file("datos.txt");
```

### Colecciones

```umbral
!! Métodos de Std (también disponibles directamente en arrays)
v: lista = {1, 2, 3};
v: longitud = Std.len(lista);              !! 3
v: nueva = Std.push(lista, 4);             !! {1, 2, 3, 4}
v: sin_ultimo = Std.pop(lista);            !! {1, 2}

!! Equivalente usando métodos directos
v: lista2 = {1, 2, 3};
v: longitud2 = lista2.len();               !! 3
v: nueva2 = lista2.push(4);                !! {1, 2, 3, 4}
v: sin_ultimo2 = lista2.pop();             !! {1, 2}

!! Diccionarios
v: dict = ["a" => 1, "b" => 2];
v: claves = Std.keys(dict);                !! ["a", "b"]
```

---

---

## 📜 Especificación Formal (v1.1.5)

### Sistema de Tipos

#### Tipos Primitivos
*   **Int**: Entero con signo de 64 bits (`i64`).
*   **Flo**: Punto flotante de doble precisión (`f64`).
*   **Bool**: Booleano (`true`, `false`).
*   **Str**: Cadena de caracteres UTF-8.
*   **Null**: Representa la ausencia de valor (`null`).
*   **Void**: Tipo de retorno para funciones sin valor.

#### Tipos Compuestos
*   **List**: Secuencia ordenada de valores (`[]`).
*   **Dict**: Colección de pares clave-valor (`HashMap`).
*   **Obj**: Instancia de una clase.
*   **Func**: Referencia a una función.

#### Reglas de Tipado
*   **Inferencia**: El tipo se infiere en la asignación si no se especifica.
*   **Anotación**: `v: nombre->Tipo = valor;` fuerza la validación del tipo.
*   **Coerción**: No existe coerción implícita entre tipos incompatibles (ej. `Int` + `Str` es error, excepto en interpolación).

### Gramática (EBNF Simplificado)

```ebnf
programa ::= sentencia*

sentencia ::= declaracion_var
            | declaracion_const
            | asignacion
            | definicion_func
            | definicion_clase
            | control_flujo
            | expresion ';'

declaracion_var ::= 'v:' identificador ('->' tipo)? '=' expresion ';'
declaracion_const ::= 'c:' identificador ('->' tipo)? '=' expresion ';'

asignacion ::= identificador '=' expresion ';'

definicion_func ::= 'f:' identificador '(' parametros? ')' ('->' tipo)? bloque

definicion_clase ::= 'cs:' identificador '{' miembro_clase* '}'

control_flujo ::= if_stmt | while_stmt | for_stmt | switch_stmt

bloque ::= '{' sentencia* '}'
```

### Reglas Semánticas

#### Igualdad
*   **Igualdad Numérica**: `Int` y `Flo` son comparables. `10 == 10.0` es `true`.
*   **Igualdad Estricta por Tipo**: Tipos diferentes (salvo numéricos) nunca son iguales. `10 == "10"` es `false`.
*   **Identidad**: Objetos y Listas se comparan por referencia.

#### Alcance y Variables
*   **Declaración Explícita**: Toda variable debe ser declarada con `v:` o `c:` antes de usarse.
*   **Asignación**: Asignar a una variable no declarada es un **ERROR**. No existe declaración implícita.
*   **Shadowing**: Declarar una variable con el mismo nombre que una existente en un ámbito superior generará una **ADVERTENCIA** (Warning).
*   **Ámbito**: Léxico (bloques `{}`).

#### Truthiness
Los siguientes valores se evalúan como `false`: `false`, `null`, `0`, `0.0`, `""`, `[]`. Todo lo demás es `true`.

### Sistema de Módulos
*   **Importación**: `equip [item] origin 'ruta';`
*   **Exportación**: `out` prefijo en declaraciones.
*   **Rutas**: Relativas al archivo actual o nombre de módulo UMP.
*   **Módulos UMP**: Si el origen no contiene `/`, `./` o `../`, se busca automáticamente en `modules_ump/`.
*   **Búsqueda**: El intérprete busca `modules_ump` subiendo en la jerarquía de directorios hasta encontrarlo.

---

## 🏗️ Arquitectura

Umbral está construido como un intérprete modular en Rust:

```
┌─────────────────────────────────────────┐
│         Código fuente (.um)             │
└─────────────────┬───────────────────────┘
                  │
                  ▼
         ┌────────────────┐
         │ umbral-lexer   │ ──→ Tokens
         └────────┬───────┘
                  │
                  ▼
         ┌────────────────┐
         │ umbral-parser  │ ──→ AST
         └────────┬───────┘
                  │
                  ▼
         ┌────────────────┐
         │ umbral-runtime │ ──→ Ejecución
         └────────┬───────┘
                  │
                  ▼
         ┌────────────────────┐
         │ umbral-interpreter │ ──→ Coordinador
         └──────┬────────┬────┘
                │        │
       ┌────────┘        └────────┐
       ▼                           ▼
┌──────────────┐          ┌──────────────┐
│ umbral-cli   │          │ umbral-repl  │
└──────────────┘          └──────────────┘
   Archivos                 Interactivo
```

### Módulos

| Módulo | Propósito | README |
|--------|-----------|--------|
| `umbral-lexer` | Análisis léxico (tokenización) | [README](./crates/umbral-lexer/README.md) |
| `umbral-parser` | Análisis sintáctico (AST) | [README](./crates/umbral-parser/README.md) |
| `umbral-runtime` | Motor de ejecución | [README](./crates/umbral-runtime/README.md) |
| `umbral-interpreter` | Coordinador de fases | [README](./crates/umbral-interpreter/README.md) |
| `umbral-cli` | Ejecutor de archivos | [README](./crates/umbral-cli/README.md) |
| `umbral-repl` | REPL interactivo | [README](./crates/umbral-repl/README.md) |

---

## 💻 Uso

### CLI - Ejecutar archivos

```bash
# Sintaxis básica
umbral archivo.um

# Ejemplos
umbral main.um
umbral codigo-ejemplo/main.um
umbral /ruta/completa/programa.um
```

### REPL - Modo interactivo

```bash
umbral-repl
```

**Comandos del REPL:**

| Comando | Descripción |
|---------|-------------|
| `:help` | Muestra ayuda |
| `:clear` | Reinicia el intérprete |
| `:exit` / `:quit` | Sale del REPL |
| `Ctrl+C` | Cancela entrada actual |
| `Ctrl+D` | Sale del REPL |

**Ejemplo de sesión:**

```
umbral> v: x = 10;
umbral> v: y = 20;
umbral> f: sumar(a, b) {
     ->     r: (a + b);
     -> }
umbral> tprint(sumar(x, y));
30
```

---

## 🛠️ Desarrollo

### Clonar el repositorio

```bash
git clone https://github.com/hersac/umbral.git
cd umbral
```

### Compilar

```bash
# Modo debug
cargo build

# Modo release (optimizado)
cargo build --release
```

### Ejecutar sin instalar

```bash
# CLI
cargo run --bin umbral -- archivo.um

# REPL
cargo run --bin umbral-repl
```

### Estructura del proyecto

```
umbral/
├── Cargo.toml              # Workspace principal
├── install.sh              # Instalador Linux/macOS
├── install.ps1             # Instalador Windows
├── uninstall.sh            # Desinstalador Linux/macOS
├── uninstall.ps1           # Desinstalador Windows
├── test_repl.sh            # Tests del REPL
├── INSTALL.md              # Guía de instalación
├── LICENSE                 # Licencia del proyecto
├── ejemplos/               # Ejemplos de código organizados
│   ├── 01_variables_y_constantes.um
│   ├── 02_funciones.um
│   ├── 03_condicionales.um
│   ├── 04_bucles.um
│   ├── 05_clases.um
│   ├── 06_importaciones_exportaciones.um
│   ├── 07_tipos_avanzados.um
│   ├── 08_ejemplo_completo.um
│   ├── 09_uso_importaciones.um
│   └── modulos/
│       └── matematicas.um
└── crates/
    ├── umbral-lexer/       # Tokenizador
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    ├── umbral-parser/      # Parser (AST)
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    ├── umbral-runtime/     # Motor de ejecución
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    ├── umbral-interpreter/ # Coordinador
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    ├── umbral-cli/         # CLI
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    └── umbral-repl/        # REPL
        ├── Cargo.toml
        ├── README.md
        └── src/
```

---

## 📖 Documentación de módulos

Cada módulo tiene su propia documentación detallada:

- **[umbral-lexer](./crates/umbral-lexer/README.md)** - Tokenización y análisis léxico
- **[umbral-parser](./crates/umbral-parser/README.md)** - Parser y construcción del AST
- **[umbral-runtime](./crates/umbral-runtime/README.md)** - Motor de ejecución y runtime
- **[umbral-interpreter](./crates/umbral-interpreter/README.md)** - API unificada
- **[umbral-cli](./crates/umbral-cli/README.md)** - Ejecutor de archivos
- **[umbral-repl](./crates/umbral-repl/README.md)** - REPL interactivo

---

## 🧪 Tests

### Ejecutar todos los tests

```bash
cargo test
```

### Tests por módulo

```bash
cargo test -p umbral-lexer
cargo test -p umbral-parser
cargo test -p umbral-runtime
cargo test -p umbral-interpreter
```

### Test del REPL

```bash
./test_repl.sh
```

### Ejemplos de código

Explora los ejemplos organizados por tema:

```bash
# Variables y constantes
umbral ejemplos/01_variables_y_constantes.um

# Funciones
umbral ejemplos/02_funciones.um

# Condicionales
umbral ejemplos/03_condicionales.um

# Bucles
umbral ejemplos/04_bucles.um

# Clases (POO)
umbral ejemplos/05_clases.um

# Importaciones y exportaciones
umbral ejemplos/06_importaciones_exportaciones.um

# Tipos avanzados
umbral ejemplos/07_tipos_avanzados.um

# Ejemplo completo (Sistema de gestión)
umbral ejemplos/08_ejemplo_completo.um

# Uso de importaciones
umbral ejemplos/09_uso_importaciones.um
```

---

## 🗺️ Roadmap

### ✅ Versión 1.1.5 (Actual)

- ✅ Lexer completo
- ✅ Parser con AST
- ✅ Runtime funcional
- ✅ Variables y constantes con tipado (Int, Flo, Str, Bool, Void)
- ✅ Funciones con retorno de tipos avanzados
- ✅ Clases y POO básico
- ✅ Condicionales (si/sino)
- ✅ Bucles (mientras)
- ✅ Operadores aritméticos, lógicos y de comparación
- ✅ Arrays y matrices ([]Tipo, [][]Tipo)
- ✅ Arrays/matrices de clases ([]Clase, [][]Clase)
- ✅ Interpolación de strings
- ✅ Sistema de módulos con importaciones/exportaciones
- ✅ Control de acceso público/privado con `out`
- ✅ 7 sintaxis de importación (equip/origin)
- ✅ CLI (`umbral`)
- ✅ REPL interactivo (`umbral-repl`)
- ✅ Instaladores para Linux/macOS/Windows
- ✅ 9 ejemplos completos organizados por tema

### 🔄 Versión 1.1.0 (Próxima)

- [ ] Soporte completo para `th` (this) en constructores
- [ ] Validación de interfaces
- [ ] Enums funcionales
- [ ] Manejo de errores con try/catch
- [ ] Bucles adicionales (for, foreach, do-while)
- [ ] Switch/case
- [ ] Librería estándar básica
- [ ] Sistema de paquetes

### 🚀 Versión 2.0.0 (Futuro)

- [ ] Sistema de tipos estático opcional
- [ ] Compilador a bytecode
- [ ] Optimización de performance
- [ ] Debugger integrado
- [ ] Language Server Protocol (LSP)
- [ ] Gestión de paquetes
- [ ] Documentación generada automáticamente

---

## 🤝 Contribuir

¡Las contribuciones son bienvenidas!

### Proceso

1. **Fork** el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/amazing-feature`)
3. Haz commit de tus cambios (`git commit -m 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing-feature`)
5. Abre un **Pull Request**

### Guías

- Sigue las convenciones de código existentes
- Escribe tests para nuevas funcionalidades
- Actualiza la documentación cuando sea necesario
- Asegúrate de que `cargo test` pase antes de enviar PR

### Reportar bugs

Abre un issue con:
- Descripción clara del problema
- Pasos para reproducir
- Comportamiento esperado vs actual
- Versión de Umbral y sistema operativo

---

## 📄 Licencia

Este proyecto está bajo la licencia especificada en [LICENSE](./LICENSE).

---

## 👥 Autores

- **Heriberto Sánchez** - Creador y mantenedor principal

---

## 📞 Contacto

- GitHub: [@hersac](https://github.com/hersac)
- Repositorio: [github.com/hersac/umbral](https://github.com/hersac/umbral)

---

## 🙏 Agradecimientos

Gracias a la comunidad de Rust por las excelentes herramientas y librerías que hicieron posible este proyecto.

---

**¡Disfruta programando en Umbral! 🎉**
