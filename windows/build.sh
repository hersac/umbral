#!/bin/bash
set -e

# Script para generar el instalador de Windows (setup.exe) de Umbral.
# Requiere:
#   - Rust configurado para el target x86_64-pc-windows-gnu
#     (rustup target add x86_64-pc-windows-gnu)
#   - makensis  (https://nsis.sourceforge.net/) en el PATH
#
# Uso:
#   ./windows/build.sh
#
# Salida:
#   windows/umbral-setup-<version>.exe

echo "╔══════════════════════════════════════════════╗"
echo "║  Build del instalador de Windows de Umbral   ║"
echo "╚══════════════════════════════════════════════╝"
echo ""

TARGET="x86_64-pc-windows-gnu"

# 1. Verificar herramientas
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo no está instalado"
    exit 1
fi
if ! command -v makensis &> /dev/null; then
    echo "❌ Error: makensis no está instalado"
    echo "   Instálalo desde https://nsis.sourceforge.net/ o con:"
    echo "   sudo apt install nsis"
    exit 1
fi

# 2. Asegurar el target de Windows
echo "🎯 Asegurando target ${TARGET}..."
rustup target add ${TARGET}

# 3. Compilar en modo release para Windows
echo "📦 Compilando Umbral (target: ${TARGET})..."
cargo build --release --target ${TARGET}

# Copiar los .exe a target/release para que el script NSIS los encuentre
CROSS_BIN="target/${TARGET}/release"
mkdir -p target/release
cp "${CROSS_BIN}/umbral.exe" target/release/umbral.exe
cp "${CROSS_BIN}/umbral-repl.exe" target/release/umbral-repl.exe

# 4. Generar el instalador
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*= *"\(.*\)".*/\1/')
echo "🚀 Generando instalador v${VERSION}..."
makensis -DVERSION="${VERSION}" windows/umbral.nsi

echo ""
echo "✅ Instalador generado: windows/umbral-setup-${VERSION}.exe"