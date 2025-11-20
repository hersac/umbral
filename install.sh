#!/bin/bash

# Script de instalación de Umbral
# Este script instala Umbral globalmente en el sistema

set -e

echo "╔════════════════════════════════════════╗"
echo "║   Instalador de Umbral v0.1.0          ║"
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
echo "Comandos disponibles:"
echo "  umbral <archivo.um>    - Ejecutar un archivo"
echo "  umbral-repl            - Iniciar REPL interactivo"
echo ""
echo "Ejemplo:"
echo "  umbral codigo-ejemplo/main.um"
echo ""
echo "Nota: Asegúrate de que ~/.cargo/bin está en tu PATH"
echo "Para agregar a tu PATH, ejecuta:"
echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
echo ""
echo "¡Disfruta programando en Umbral! 🎉"
