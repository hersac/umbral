#!/bin/bash

# Script de prueba automatizada para el REPL
# Este script alimenta comandos al REPL y captura la salida

echo "Testing Umbral REPL..."

# Test 1: Variables simples
{
    echo "v: x = 10;"
    echo "v: y = 20;"
    echo "tprint(x + y);"
    echo ":exit"
} | ./target/debug/umbral-repl | grep -q "30" && echo "✓ Test 1: Variables simples - PASSED" || echo "✗ Test 1: FAILED"

# Test 2: Constantes
{
    echo "c: PI = 3.14;"
    echo "tprint(PI);"
    echo ":exit"
} | ./target/debug/umbral-repl | grep -q "3.14" && echo "✓ Test 2: Constantes - PASSED" || echo "✗ Test 2: FAILED"

# Test 3: Comando :clear
{
    echo "v: x = 10;"
    echo ":clear"
    echo ":exit"
} | ./target/debug/umbral-repl | grep -q "Estado del intérprete reiniciado" && echo "✓ Test 3: Comando :clear - PASSED" || echo "✗ Test 3: FAILED"

# Test 4: Strings con interpolación
{
    echo "v: nombre = 'Umbral';"
    echo "tprint(\"Lenguaje: &nombre\");"
    echo ":exit"
} | ./target/debug/umbral-repl | grep -q "Lenguaje: Umbral" && echo "✓ Test 4: Interpolación - PASSED" || echo "✗ Test 4: FAILED"

echo ""
echo "Tests completados!"
