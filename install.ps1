# Script de instalación de Umbral para Windows
# Ejecutar con: PowerShell -ExecutionPolicy Bypass -File install.ps1

Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   Instalador de Umbral v1.0.0          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Verificar que Rust esté instalado
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Cargo no está instalado" -ForegroundColor Red
    Write-Host "Por favor instala Rust desde: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

$rustVersion = cargo --version
Write-Host "✓ Rust encontrado: $rustVersion" -ForegroundColor Green
Write-Host ""

# Compilar en modo release
Write-Host "📦 Compilando Umbral (esto puede tomar unos minutos)..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error al compilar" -ForegroundColor Red
    exit 1
}

Write-Host "✓ Compilación exitosa" -ForegroundColor Green
Write-Host ""

# Instalar globalmente
Write-Host "🚀 Instalando Umbral globalmente..." -ForegroundColor Yellow
cargo install --path . --force

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error al instalar" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║   ✓ Umbral instalado correctamente     ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "Comandos disponibles:" -ForegroundColor Cyan
Write-Host "  umbral <archivo.um>    - Ejecutar un archivo" -ForegroundColor White
Write-Host "  umbral-repl            - Iniciar REPL interactivo" -ForegroundColor White
Write-Host ""
Write-Host "Ejemplo:" -ForegroundColor Cyan
Write-Host "  umbral codigo-ejemplo\main.um" -ForegroundColor White
Write-Host ""

# Verificar si .cargo\bin está en el PATH
$cargoPath = "$env:USERPROFILE\.cargo\bin"
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($currentPath -notlike "*$cargoPath*") {
    Write-Host "⚠️  Configurando PATH automáticamente..." -ForegroundColor Yellow
    
    # Agregar al PATH del usuario
    $newPath = "$currentPath;$cargoPath"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    
    Write-Host "✓ PATH configurado correctamente" -ForegroundColor Green
    Write-Host ""
    Write-Host "⚠️  IMPORTANTE: Cierra y vuelve a abrir PowerShell/CMD para que los cambios surtan efecto" -ForegroundColor Yellow
} else {
    Write-Host "✓ PATH ya está configurado correctamente" -ForegroundColor Green
}

Write-Host ""
Write-Host "¡Disfruta programando en Umbral! 🎉" -ForegroundColor Cyan
Write-Host ""
