# Script de desinstalación de Umbral para Windows
# Ejecutar con: PowerShell -ExecutionPolicy Bypass -File uninstall.ps1

Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   Desinstalador de Umbral              ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Desinstalar
Write-Host "🗑️  Desinstalando Umbral..." -ForegroundColor Yellow

cargo uninstall umbral 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  umbral no estaba instalado" -ForegroundColor Gray
}

cargo uninstall umbral-repl 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  umbral-repl no estaba instalado" -ForegroundColor Gray
}

Write-Host ""
Write-Host "✓ Umbral desinstalado correctamente" -ForegroundColor Green
Write-Host ""
