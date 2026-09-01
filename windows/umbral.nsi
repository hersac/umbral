; ============================================================
;  Instalador de Umbral para Windows (NSIS)
;  Compilar:  makensis umbral.nsi
;  Requiere:  makensis  (https://nsis.sourceforge.net/)
;  Fuente:    windows/umbral.nsi
; ============================================================

Unicode true

!include "MUI2.nsh"
!include "WinMessages.nsh"

; ------------------------------------------------------------
;  Definiciones generales
; ------------------------------------------------------------
!define APPNAME "Umbral"
!ifndef VERSION
  !define VERSION "1.3.8"
!endif
!define APPVERSION "${VERSION}"
!define EXE_MAIN "umbral.exe"
!define EXE_REPL "umbral-repl.exe"

Name "${APPNAME} ${APPVERSION}"
OutFile "umbral-setup-${APPVERSION}.exe"
; Instalación por usuario (sin privilegios de administrador)
InstallDir "$LOCALAPPDATA\Umbral"
InstallDirRegKey HKCU "Software\Umbral" "InstallDir"
RequestExecutionLevel user
ShowInstDetails show
ShowUninstDetails show

; ------------------------------------------------------------
;  Interfaz MUI
; ------------------------------------------------------------
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\bin\${EXE_REPL}"
!define MUI_FINISHPAGE_RUN_TEXT "Iniciar el REPL de Umbral"
!define MUI_FINISHPAGE_RUN_CHECKED

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "English"

; Subdirectorio donde se instalan los binarios (se agrega al PATH)
Var /GLOBAL BinDir

; ------------------------------------------------------------
;  Sección de instalación
; ------------------------------------------------------------
Section "Instalar Umbral" SecInstall
  StrCpy $BinDir "$INSTDIR\bin"
  SetOutPath "$BinDir"

  File "..\target\release\${EXE_MAIN}"
  File "..\target\release\${EXE_REPL}"

  ; Guardar ruta para el desinstalador
  WriteRegStr HKCU "Software\Umbral" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Umbral" "BinDir" "$BinDir"

  ; Agregar la carpeta bin al PATH del usuario
  Push "$BinDir"
  Call AddToPath

  ; Entrada en "Agregar o quitar programas"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${APPVERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "Heriberto Sánchez"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$BinDir\${EXE_MAIN}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoRepair" 1

  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; ------------------------------------------------------------
;  Sección de desinstalación
; ------------------------------------------------------------
Section "Uninstall"
  ; Reconstruir la ruta de binarios
  StrCpy $BinDir "$INSTDIR\bin"

  ; Quitar la carpeta bin del PATH del usuario
  Push "$BinDir"
  Call un.RemoveFromPath

  ; Eliminar archivos
  Delete "$BinDir\${EXE_MAIN}"
  Delete "$BinDir\${EXE_REPL}"
  RMDir "$BinDir"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  ; Limpiar registro
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
  DeleteRegKey HKCU "Software\Umbral"

  ; Notificar a la shell que cambió el entorno
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

; ------------------------------------------------------------
;  Funciones de manipulación del PATH (canónicas del wiki de NSIS)
;  http://nsis.sourceforge.net/Path_Manipulation
; ------------------------------------------------------------
;  AddToPath - Agrega el directorio $0 al PATH del usuario
;  (evita duplicados y maneja PATH vacío)
; ------------------------------------------------------------
Function AddToPath
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4

  ; no agregar si el directorio no existe
  IfFileExists "$0\*.*" "" AddToPath_done

  ReadRegStr $1 HKCU "Environment" "Path"
  StrCpy $2 $1 1 -1
  StrCmp $2 ";" 0 +3
    StrCpy $1 $1 -1 ; quitar el ';' final
  IntCmp $1 "" AddToPath_Get
    ; ya hay contenido: comprobar si $0 ya está presente
    StrCpy $2 $1
    StrCpy $3 ""
    Push "$2"
    Push "$0"
    Call StrStr
    Pop $2
    StrCmp $2 "" AddToPath_Get
    ; ya está en el PATH
    Goto AddToPath_done

AddToPath_Get:
  ClearErrors
  ReadRegStr $2 HKCU "Environment" "Path"
  StrCmp $2 "" AddToPath_Add
    StrCpy $3 "$0;$2"
    Goto AddToPath_Write
AddToPath_Add:
  StrCpy $3 "$0"
AddToPath_Write:
  WriteRegExpandStr HKCU "Environment" "Path" $3
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

AddToPath_done:
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; ------------------------------------------------------------
;  RemoveFromPath - Quita el directorio $0 del PATH del usuario
; ------------------------------------------------------------
Function un.RemoveFromPath
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6

  ReadRegStr $1 HKCU "Environment" "Path"
  StrCpy $5 $1 1 -1
  StrCmp $5 ";" +2
    StrCpy $1 "$1;" ; asegurar que termine en ';'

  Push $1
  Push $0
  Call un.TrimPath
  Pop $1

  StrCpy $3 $1 1 -1
  StrCmp $3 ";" +2
    StrCpy $1 "$1;" ; asegurar que termine en ';'

  Push $1
  Push $0
  Call un.TrimPath
  Pop $1

  ClearErrors
  ReadRegStr $2 HKCU "Environment" "Path"
  WriteRegExpandStr HKCU "Environment" "Path" $1
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

un.RemoveFromPath_done:
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; ------------------------------------------------------------
;  TrimPath - Quita la ruta $0 de $1
;  Entrada: tope de pila = ruta a quitar, segundo = PATH
;  Salida:  tope de pila = PATH sin la ruta
; ------------------------------------------------------------
Function un.TrimPath
  Exch $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6

  StrCpy $5 $1 1 -1
  StrCmp $5 ";" +2
    StrCpy $1 "$1;"
  StrCpy $6 $1

  ; si $0 está vacío o no es un directorio, no hay nada que hacer
  StrCpy $3 $0 1 0
  StrCmp $3 ";" un.TrimPath_NoPath
  StrCpy $4 ""
  StrCpy $2 -1
  IntOp $2 $2 + 1
  StrCpy $3 $6 $2
  StrCmp $3 "" un.TrimPath_NoPath
  StrCpy $4 $0
  StrCmp $3 $4 un.TrimPath_Found
  Goto -5

un.TrimPath_NoPath:
  StrCpy $1 ""
  Goto un.TrimPath_Done

un.TrimPath_Found:
  StrCpy $5 $6 "" $2
  IntOp $5 $5 + 1
  StrCpy $3 $6 $5
  StrCpy $5 $3
  StrCpy $4 ""
  StrCpy $2 -1
  IntOp $2 $2 + 1
  StrCpy $3 $5 $2
  StrCmp $3 "" un.TrimPath_NoPath2
  StrCpy $4 ";"
  StrCmp $3 $4 un.TrimPath_Done
  Goto -6

un.TrimPath_NoPath2:
  StrCpy $1 ""

un.TrimPath_Done:
  IntOp $5 $2 + 1
  StrCpy $5 $6 $5 -1
  StrCpy $1 $5
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Exch $1
FunctionEnd

; ------------------------------------------------------------
;  StrStr - Busca una subcadena dentro de otra
;  Entrada:  tope de pila = subcadena, segundo = cadena
;  Salida:   tope de pila = resto desde la coincidencia o vacío
; ------------------------------------------------------------
Function StrStr
  Exch $0
  Exch
  Exch $1
  Push $2
  Push $3
  Push $4
  Push $5

  StrLen $2 $0
  StrCpy $3 ""
  StrCpy $4 ""
  StrCpy $5 0

StrStr_loop:
  StrCpy $4 $1 1 $5
  StrCmp $4 "" StrStr_NotFound
  StrCpy $4 $1 $2 $5
  StrCmp $4 $0 StrStr_Found
  IntOp $5 $5 + 1
  Goto StrStr_loop

StrStr_Found:
  StrCpy $3 $1 "" $5
  Goto StrStr_End

StrStr_NotFound:
  StrCpy $3 ""

StrStr_End:
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  StrCpy $1 $3
  Exch $1
  Exch
  Pop $0
FunctionEnd