# Ejemplos de Umbral

Esta carpeta contiene ejemplos completos y organizados que demuestran todas las características del lenguaje Umbral.

## 📚 Índice de Ejemplos

### 1️⃣ Variables y Constantes
**Archivo:** `01_variables_y_constantes.um`

Aprende sobre:
- Variables mutables (`v:`)
- Constantes inmutables (`c:`)
- Tipos de datos: Int, Flo, Str, Bool
- Arrays unidimensionales
- Matrices (arrays bidimensionales)
- Acceso a elementos

```bash
umbral 01_variables_y_constantes.um
```

---

### 2️⃣ Funciones
**Archivo:** `02_funciones.um`

Aprende sobre:
- Funciones con retorno de valores
- Funciones con múltiples parámetros
- Funciones con diferentes tipos de retorno (Int, Flo, Str, Bool, Void)
- Funciones con lógica condicional
- Funciones con bucles
- Funciones que retornan arrays
- Funciones anidadas
- Recursividad

```bash
umbral 02_funciones.um
```

---

### 3️⃣ Condicionales
**Archivo:** `03_condicionales.um`

Aprende sobre:
- Condicional simple (`si`)
- Condicional con alternativa (`si-sino`)
- Condicionales anidados
- Operadores de comparación (==, !=, <, >, <=, >=)
- Operadores lógicos (&&, ||, !)
- Expresiones complejas
- Condicionales en funciones
- Validaciones

```bash
umbral 03_condicionales.um
```

---

### 4️⃣ Bucles
**Archivo:** `04_bucles.um`

Aprende sobre:
- Bucle `mientras` (while)
- Suma acumulativa
- Iteración sobre arrays
- Bucles anidados
- Condiciones complejas
- Generación de secuencias
- Búsqueda en arrays
- Algoritmos: factorial, Fibonacci
- Construcción de arrays dinámicamente
- Conteo de elementos

```bash
umbral 04_bucles.um
```

---

### 5️⃣ Clases (POO)
**Archivo:** `05_clases.um`

Aprende sobre:
- Definición de clases (`cs:`)
- Propiedades (`pr:`)
- Constructores (`pu f:`)
- Métodos públicos (`pu f:`)
- Uso de `th` (this)
- Clases con validaciones
- Clases con arrays
- Funciones que retornan clases
- Arrays de clases (`[]Clase`)
- Matrices de clases (`[][]Clase`)

```bash
umbral 05_clases.um
```

---

### 6️⃣ Importaciones y Exportaciones
**Archivo:** `06_importaciones_exportaciones.um`

Aprende sobre:
- Sistema de módulos
- Exportaciones con `out`
- Elementos públicos vs privados
- 7 sintaxis de importación:
  1. Importación simple
  2. Importación con alias
  3. Importación de lista
  4. Lista con alias
  5. Importar todo (*)
  6. Importar con namespace
  7. Orden invertido (origin/equip)
- Validación de exportaciones
- Mejores prácticas

```bash
umbral 06_importaciones_exportaciones.um
```

---

### 7️⃣ Tipos Avanzados
**Archivo:** `07_tipos_avanzados.um`

Aprende sobre:
- Tipos básicos: Int, Flo, Str, Bool, Void
- Arrays tipados: []Int, []Flo, []Str, []Bool
- Matrices tipadas: [][]Int, [][]Str, etc.
- Clases como tipos
- Funciones con diferentes retornos
- Arrays de clases: []Clase
- Matrices de clases: [][]Clase
- Parámetros con tipos explícitos
- Anotaciones de tipo en variables

```bash
umbral 07_tipos_avanzados.um
```

---

### 8️⃣ Ejemplo Completo
**Archivo:** `08_ejemplo_completo.um`

Sistema de Gestión Académica que integra:
- Variables y constantes
- Múltiples clases (Estudiante, Curso)
- Funciones auxiliares
- Condicionales complejas
- Bucles para procesamiento
- Arrays de objetos
- Validaciones
- Cálculos estadísticos

Este ejemplo demuestra cómo construir una aplicación completa en Umbral.

```bash
umbral 08_ejemplo_completo.um
```

---

### 9️⃣ Uso de Importaciones
**Archivo:** `09_uso_importaciones.um`

Ejemplo práctico de:
- Importar funciones y constantes de módulos
- Usar elementos importados
- Importación con namespace
- Módulo: `modulos/matematicas.um`

```bash
umbral 09_uso_importaciones.um
```

---

### 🔟 Operador Spread
**Archivo:** `10_operador_spread.um`

Aprende sobre:
- Operador spread (`&`) para expandir arrays
- Combinar múltiples arrays
- Mezclar spread con valores literales
- Spread múltiple en un solo array
- Comparación con concatenación (`+`)
- Arrays vacíos y spread
- Funciones que retornan arrays expandidos
- Spread anidado
- Construcción dinámica de arrays
- Clonar y extender arrays

```bash
umbral 10_operador_spread.um
```

---

## 📦 Módulos

### modulos/matematicas.um

Módulo de utilidades matemáticas con:

**Funciones públicas (exportadas):**
- `sumar(a, b)` - Suma dos números
- `restar(a, b)` - Resta dos números
- `multiplicar(a, b)` - Multiplica dos números
- `dividir(a, b)` - Divide dos números (con validación)
- `potencia(base, exponente)` - Calcula potencias

**Constantes públicas (exportadas):**
- `PI` - 3.14159
- `E` - 2.71828
- `PHI` - 1.61803

**Elementos privados:**
- `funcionInterna()` - No se puede importar
- `CONSTANTE_INTERNA` - No se puede importar

---

## 🚀 Ejecución de Ejemplos

### Ejecutar todos los ejemplos

```bash
# Desde la carpeta ejemplos
for file in *.um; do
    echo "=== Ejecutando $file ==="
    umbral "$file"
    echo ""
done
```

### Ejecutar un ejemplo específico

```bash
umbral ejemplos/01_variables_y_constantes.um
```

### Ejecutar desde el directorio raíz

```bash
cd /ruta/a/umbral
umbral ejemplos/08_ejemplo_completo.um
```

---

## 📖 Orden Recomendado de Aprendizaje

Para aprender Umbral desde cero, sigue este orden:

1. **01_variables_y_constantes.um** - Fundamentos básicos
2. **02_funciones.um** - Crear y usar funciones
3. **03_condicionales.um** - Control de flujo
4. **04_bucles.um** - Iteración y repetición
5. **05_clases.um** - Programación orientada a objetos
6. **07_tipos_avanzados.um** - Sistema de tipos
7. **06_importaciones_exportaciones.um** - Módulos y organización
8. **09_uso_importaciones.um** - Práctica de importaciones
9. **08_ejemplo_completo.um** - Aplicación completa

---

## 💡 Consejos

- **Lee los comentarios**: Cada archivo tiene explicaciones detalladas
- **Experimenta**: Modifica los ejemplos y observa los resultados
- **Usa el REPL**: Prueba fragmentos de código interactivamente con `umbral-repl`
- **Revisa errores**: Si algo no funciona, lee el mensaje de error cuidadosamente
- **Combina conceptos**: Usa lo aprendido en varios ejemplos juntos

---

## 🔍 Características Destacadas

### Sistema de Tipos Completo
```umbral
v: numero->Int = 42;
v: decimal->Flo = 3.14;
f: obtener()->[][]Persona { ... }
```

### Importaciones Flexibles
```umbral
equip { func1, func2 } origin 'modulo.um';
origin 'modulo.um' equip * as prefijo;
```

### Clases Completas
```umbral
cs: MiClase {
    pr: propiedad->Str;
    pu f: MiClase(param->Str) {
        th.propiedad = param;
    }
}
```

---

## 📚 Recursos Adicionales

- **README Principal**: [../README.md](../README.md)
- **Guía de Instalación**: [../INSTALL.md](../INSTALL.md)
- **Documentación de Módulos**: [../crates/](../crates/)

---

## 🤝 Contribuir

¿Tienes ideas para nuevos ejemplos? ¡Contribuye!

1. Crea un nuevo archivo `.um` en esta carpeta
2. Sigue el formato de numeración: `10_nuevo_ejemplo.um`
3. Incluye comentarios explicativos
4. Actualiza este README
5. Envía un Pull Request

---

**¡Disfruta aprendiendo Umbral! 🎉**
