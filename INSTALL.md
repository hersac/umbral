# Guía de Instalación de Umbral

Esta guía te ayudará a instalar Umbral v1.1.5 en tu sistema operativo.

---

## 📋 Requisitos previos

### Todos los sistemas

- **Rust**: Versión 1.70 o superior
  - Descargar desde: https://rustup.rs/
- **Git**: Para clonar el repositorio
  - Descargar desde: https://git-scm.com/

### Verificar requisitos

```bash
# Verificar Rust
rustc --version
cargo --version

# Verificar Git
git --version
```

---

## 🐧 Instalación en Linux

### Paso 1: Clonar el repositorio

```bash
git clone https://github.com/hersac/umbral.git
cd umbral
```

### Paso 2: Ejecutar el instalador

```bash
chmod +x install.sh
./install.sh
```

El script hará lo siguiente:
1. ✅ Verificará que Rust esté instalado
2. 📦 Compilará Umbral en modo release
3. 🚀 Instalará los binarios `umbral` y `umbral-repl` en `~/.cargo/bin`
4. ℹ️ Mostrará instrucciones para configurar el PATH (si es necesario)

### Paso 3: Configurar PATH (si es necesario)

Si el comando `umbral` no se encuentra, agrega esto a tu `~/.bashrc`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Luego recarga tu shell:

```bash
source ~/.bashrc
```

### Paso 4: Verificar la instalación

```bash
umbral --version
umbral-repl
```

---

## 🍎 Instalación en macOS

### Paso 1: Instalar Homebrew (si no lo tienes)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Paso 2: Instalar Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Paso 3: Clonar e instalar Umbral

```bash
git clone https://github.com/hersac/umbral.git
cd umbral
chmod +x install.sh
./install.sh
```

### Paso 4: Configurar PATH (si es necesario)

Para **bash** (`~/.bash_profile`):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bash_profile
```

Para **zsh** (`~/.zshrc`):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.zshrc
```

### Paso 5: Verificar la instalación

```bash
umbral --version
umbral-repl
```

---

## 🪟 Instalación en Windows

### Opción A: Instalador gráfico (.exe) — recomendada

Si se dispone de un instalador compilado (`umbral-setup-<versión>.exe`), es la forma más sencilla:

1. Ejecuta `umbral-setup-<versión>.exe`
2. En el asistente, selecciona el directorio donde quieres instalar Umbral
3. El instalador copia `umbral.exe` y `umbral-repl.exe` y agrega automáticamente el directorio al `PATH` del usuario
4. Al finalizar puedes iniciar el REPL directamente

Para **desinstalar**, usa "Agregar o quitar programas" de Windows o ejecuta `uninstall.exe` en la carpeta de instalación. El desinstalador también elimina la entrada del `PATH`.

### Opción B: Compilar e instalar desde el código fuente

1. Instala Rust (https://rustup.rs/) y Git (https://git-scm.com/download/win)
2. Clona el repositorio:
   ```powershell
   git clone https://github.com/hersac/umbral.git
   cd umbral
   ```
3. Ejecuta el instalador:
   **IMPORTANTE**: Abre **PowerShell como Administrador** (clic derecho → "Ejecutar como administrador")
   ```powershell
   PowerShell -ExecutionPolicy Bypass -File install.ps1
   ```
   El script hará lo siguiente:
   1. ✅ Verificará que Rust esté instalado
   2. 📦 Compilará Umbral en modo release
   3. 🚀 Instalará los binarios en `%USERPROFILE%\.cargo\bin`
   4. ⚙️ Configurará automáticamente el PATH del sistema

### Paso 5: Reiniciar terminal

**IMPORTANTE**: Cierra y vuelve a abrir PowerShell/CMD para que los cambios en el PATH surtan efecto.

### Paso 6: Verificar la instalación

```powershell
umbral --version
umbral-repl
```

### Generar el instalador .exe (para distribución)

Para producir `umbral-setup-<versión>.exe` desde el código fuente:

```bash
./windows/build.sh
```

Requisitos:
- **Rust** con el target `x86_64-pc-windows-gnu` (en Windows se usa el nativo; el script agrega el target automáticamente)
- **makensis** (NSIS) — en Linux: `sudo apt install nsis`; en Windows descárgalo de https://nsis.sourceforge.net/

El script compila los binarios para Windows, los copia a `target/release/` y genera el instalador en `windows/`. El asistente permite elegir el directorio de instalación y agrega ese directorio al `PATH` del usuario (HKCU), sin requerir permisos de administrador. El desinstalador elimina los archivos y la entrada del `PATH`.

---

## ✅ Verificar que todo funciona

### Prueba rápida con archivo

#### 1. Crear un archivo de prueba

```bash
echo 'v: x = 42; tprint("El resultado es: &x");' > test.um
```

#### 2. Ejecutarlo

```bash
umbral test.um
```

**Salida esperada:**
```
El resultado es: 42
```

### Prueba del REPL

```bash
umbral-repl
```

Deberías ver el banner de bienvenida:

```
╔════════════════════════════════════════╗
║     Umbral REPL v1.1.5                 ║
║     Lenguaje de Programación Umbral   ║
╚════════════════════════════════════════╝
```

Prueba ejecutar:

```
umbral> v: saludo = "Hola Mundo";
umbral> tprint(saludo);
Hola Mundo
umbral> v: suma = 10 + 20;
umbral> tprint(suma);
30
umbral> :exit
Adiós!
```

---

## 🔧 Solución de problemas

### Error: "cargo: command not found"

**Causa**: Rust no está instalado o no está en el PATH.

**Solución**:
1. Instala Rust desde https://rustup.rs/
2. Reinicia tu terminal
3. Verifica: `cargo --version`

### Error: "umbral: command not found" (después de instalar)

#### Linux/macOS

**Causa**: `~/.cargo/bin` no está en tu PATH.

**Solución**:

1. Verifica tu PATH:
```bash
echo $PATH | grep cargo
```

2. Si no aparece, agrega a tu `~/.bashrc` o `~/.zshrc`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

3. Recarga el shell:
```bash
source ~/.bashrc  # o source ~/.zshrc
```

#### Windows

**Causa**: El PATH no se actualizó correctamente.

**Solución**:

1. Cierra y vuelve a abrir PowerShell/CMD (IMPORTANTE)

2. Verifica la variable PATH:
```powershell
$env:Path
```

3. Si no aparece `%USERPROFILE%\.cargo\bin`, ejecuta el instalador nuevamente como Administrador

4. Alternativamente, agrega manualmente al PATH:
   - Busca "Variables de entorno" en el menú de inicio
   - Edita la variable "Path" del usuario
   - Agrega: `%USERPROFILE%\.cargo\bin`

### Error de permisos en Linux/macOS

**Causa**: El script de instalación no tiene permisos de ejecución.

**Solución**:
```bash
chmod +x install.sh
./install.sh
```

### Error de permisos en Windows

**Causa**: PowerShell no tiene permisos de administrador.

**Solución**:
1. Cierra PowerShell
2. Busca "PowerShell" en el menú de inicio
3. Clic derecho → "Ejecutar como administrador"
4. Navega al directorio de Umbral
5. Ejecuta el instalador nuevamente

### Error: "execution policy" en Windows

**Causa**: La política de ejecución de PowerShell está restringida.

**Solución**:
```powershell
PowerShell -ExecutionPolicy Bypass -File install.ps1
```

O cambia la política permanentemente:
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Error de compilación

**Causa**: Falta alguna dependencia de Rust o hay un problema con el código fuente.

**Solución**:

1. Actualiza Rust:
```bash
rustup update
```

2. Limpia y recompila:
```bash
cargo clean
cargo build --release
```

3. Si persiste, reporta el issue en GitHub con el log completo.

---

## 🔄 Actualización

Para actualizar Umbral a la última versión:

```bash
cd umbral
git pull origin main
./install.sh  # o install.ps1 en Windows
```

---

## 🗑️ Desinstalación

### Linux / macOS

```bash
cd umbral
./uninstall.sh
```

O manualmente:

```bash
cargo uninstall umbral
cargo uninstall umbral-repl
```

### Windows

```powershell
cd umbral
PowerShell -ExecutionPolicy Bypass -File uninstall.ps1
```

O manualmente:

```powershell
cargo uninstall umbral
cargo uninstall umbral-repl
```

---

## 🛠️ Instalación desde código fuente (sin script)

Si prefieres instalación manual:

```bash
# 1. Clonar
git clone https://github.com/hersac/umbral.git
cd umbral

# 2. Compilar
cargo build --release

# 3. Instalar
cargo install --path .

# 4. Verificar
umbral --version
umbral-repl
```

Los binarios se instalarán en:
- Linux/macOS: `~/.cargo/bin/`
- Windows: `%USERPROFILE%\.cargo\bin\`

---

## 📦 Instalación en sistemas sin Rust

Si no puedes instalar Rust, puedes usar los binarios precompilados (cuando estén disponibles):

### Releases

Descarga el binario para tu plataforma desde:
https://github.com/hersac/umbral/releases

Extrae y mueve a una ubicación en tu PATH:

**Linux/macOS:**
```bash
tar -xzf umbral-linux-x64.tar.gz
sudo mv umbral /usr/local/bin/
sudo mv umbral-repl /usr/local/bin/
```

**Windows:**
```powershell
# Extrae el ZIP
# Mueve los .exe a C:\Program Files\Umbral\
# Agrega C:\Program Files\Umbral\ al PATH
```

---

## 🌐 Instalación en entornos especiales

### Docker

```dockerfile
FROM rust:1.70

WORKDIR /app
RUN git clone https://github.com/hersac/umbral.git
WORKDIR /app/umbral
RUN cargo install --path .

CMD ["umbral-repl"]
```

### WSL (Windows Subsystem for Linux)

Sigue las instrucciones de Linux dentro de tu distribución WSL.

---

## 📝 Siguiente paso

Una vez instalado, consulta:

- [README.md](./README.md) - Documentación principal y sintaxis
- [Ejemplos de código](./codigo-ejemplo/main.um) - Código de ejemplo
- [REPL](./crates/umbral-repl/README.md) - Guía del REPL interactivo

---

## 💬 ¿Necesitas ayuda?

- **Issues**: https://github.com/hersac/umbral/issues
- **Discussions**: https://github.com/hersac/umbral/discussions

---

**¡Disfruta programando en Umbral! 🎉**
