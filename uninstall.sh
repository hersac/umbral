#!/bin/bash

# Script de desinstalación de Umbral

set -e

echo "╔════════════════════════════════════════╗"
echo "║   Desinstalador de Umbral              ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Desinstalar
echo "🗑️  Desinstalando Umbral..."
cargo uninstall umbral 2>/dev/null || echo "umbral no estaba instalado"
cargo uninstall umbral-repl 2>/dev/null || echo "umbral-repl no estaba instalado"

echo ""
echo "✓ Umbral desinstalado correctamente"
echo ""
