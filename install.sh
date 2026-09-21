#!/bin/bash

# Script de instalación de Umbral
# Este script instala Umbral globalmente en el sistema

set -e

echo "╔════════════════════════════════════════╗"
echo "║   Instalador de Umbral v1.5.6          ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Verificar que Rust esté instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo no está instalado"
    echo "Por favor instala Rust desde: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust encontrado: $(rustc --version)"
echo ""

# Compilar en modo release
echo "📦 Compilando Umbral (esto puede tomar unos minutos)..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Error al compilar"
    exit 1
fi

echo "✓ Compilación exitosa"
echo ""

# Instalar globalmente
echo "🚀 Instalando Umbral globalmente..."
cargo install --path . --force

if [ $? -ne 0 ]; then
    echo "❌ Error al instalar"
    exit 1
fi

echo ""
echo "╔════════════════════════════════════════╗"
echo "║   ✓ Umbral instalado correctamente     ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Configurar PATH automáticamente
echo "🔧 Configurando PATH en ~/.bashrc..."

# Verificar si ~/.bashrc existe, si no, crearlo
if [ ! -f "$HOME/.bashrc" ]; then
    echo "📝 Creando ~/.bashrc..."
    touch "$HOME/.bashrc"
fi

# Verificar si la ruta de cargo ya está en .bashrc
if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$HOME/.bashrc"; then
    echo "" >> "$HOME/.bashrc"
    echo "# Agregado por el instalador de Umbral" >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.bashrc"
    echo "✓ PATH agregado a ~/.bashrc"
else
    echo "✓ PATH ya configurado en ~/.bashrc"
fi

# Aplicar cambios en la sesión actual
export PATH="$HOME/.cargo/bin:$PATH"
echo "✓ PATH actualizado en la sesión actual"

echo ""
echo "Comandos disponibles:"
echo "  umbral <archivo.um>    - Ejecutar un archivo"
echo "  umbral-repl            - Iniciar REPL interactivo"
echo ""
echo "Ejemplo:"
echo "  umbral ejemplos/01_variables_y_constantes.um"
echo ""
echo "Nota: Para nuevas terminales, el PATH se cargará automáticamente."
echo "Para la terminal actual, ejecuta: source ~/.bashrc"
echo ""
echo "¡Disfruta programando en Umbral! 🎉"
