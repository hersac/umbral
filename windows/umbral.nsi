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
  !define VERSION "1.5.4"
!endif
; VI_VERSION debe ser estrictamente X.X.X.X numérico para VIProductVersion.
; Se pasa desde CI con -DVI_VERSION=... (sanitizado desde el tag).
; El fallback solo sirve para compilación local.
!ifndef VI_VERSION
  !define VI_VERSION "1.5.4.0"
!endif
!define APPVERSION "${VERSION}"
!define EXE_MAIN "umbral.exe"
!define EXE_REPL "umbral-repl.exe"

Name "${APPNAME} ${APPVERSION}"
; NOTA: usar ${__FILEDIR__} para que las rutas no dependan del CWD ni de
; si makensis se invoca con ruta relativa o absoluta (evita "windows/windows").
OutFile "${__FILEDIR__}\umbral-setup-${APPVERSION}.exe"
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
!define MUI_ICON "${__FILEDIR__}\..\images\umbral-pet-sin-fondo.ico"
!define MUI_UNICON "${__FILEDIR__}\..\images\umbral-pet-sin-fondo.ico"
; Imagen personalizada (mascota Umbral sin fondo -> BMP 24-bit para NSIS)
; Fuente: images/umbral-pet-sin-fondo.png convertida a BMP (NSIS no acepta PNG directo)
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${__FILEDIR__}\umbral-header.bmp"
!define MUI_HEADERIMAGE_BITMAP_NOSTRETCH
!define MUI_WELCOMEFINISHPAGE_BITMAP "${__FILEDIR__}\umbral-wizard.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${__FILEDIR__}\umbral-wizard.bmp"
BrandingText "Hersac"
VIProductVersion "${VI_VERSION}"
VIAddVersionKey "ProductName" "${APPNAME}"
VIAddVersionKey "CompanyName" "Hersac"
VIAddVersionKey "FileDescription" "${APPNAME} - Interprete de lenguaje de programacion"
VIAddVersionKey "LegalCopyright" "Hersac"
VIAddVersionKey "FileVersion" "${APPVERSION}"
VIAddVersionKey "ProductVersion" "${APPVERSION}"
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

  File "${__FILEDIR__}\..\target\release\${EXE_MAIN}"
  File "${__FILEDIR__}\..\target\release\${EXE_REPL}"

  ; Guardar ruta para el desinstalador
  WriteRegStr HKCU "Software\Umbral" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Umbral" "BinDir" "$BinDir"

  ; Agregar la carpeta bin al PATH del usuario
  Push "$BinDir"
  Call AddToPath

  ; Entrada en "Agregar o quitar programas"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${APPVERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "Hersac"
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
;  Funciones de manipulación del PATH (solo tocan HKCU Environment\Path)
;  - AddToPath: agrega $0 como entrada exacta (sin duplicar, sin tocar
;    el resto de entradas ni otras variables de entorno).
;  - un.RemoveFromPath: elimina solo $0 como entrada exacta, conserva
;    el resto del PATH y no toca otras variables. Si no hay cambios,
;    no escribe ni notifica.
; ------------------------------------------------------------
;  AddToPath - Agrega el directorio $0 al PATH del usuario
; ------------------------------------------------------------
Function AddToPath
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4

  ; no agregar si el directorio no existe
  IfFileExists "$0\*.*" "" AddToPath_done
  ; no agregar si la ruta está vacía
  StrCmp $0 "" AddToPath_done

  ReadRegStr $1 HKCU "Environment" "Path"
  ; PATH vacío -> escribir directo
  StrCmp $1 "" AddToPath_write_new

  ; Búsqueda exacta insensible a mayúsculas: ";PATH;" contiene ";DIR;"
  ; (los ';' evitan falsos positivos con subcadenas, ej. bin vs bin2)
  StrCpy $2 ";$1;"
  StrCpy $3 ";$0;"
  Push "$2"
  Push "$3"
  Call StrStr
  Pop $2
  ; Si StrStr devolvió algo distinto de vacío, la entrada ya existe
  StrCmp $2 "" AddToPath_write_append AddToPath_done

AddToPath_write_append:
  ; Releer por si cambió y anexar al final (preserva el orden existente)
  ClearErrors
  ReadRegStr $2 HKCU "Environment" "Path"
  StrCmp $2 "" AddToPath_write_new
    StrCpy $3 "$2;$0"
    Goto AddToPath_write
AddToPath_write_new:
  StrCpy $3 "$0"
AddToPath_write:
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
;  RemoveFromPath - Quita solo el directorio $0 del PATH del usuario
;  Conserva todas las demás entradas y variables. No escribe si no
;  hay cambios (evita borrar el PATH por error).
; ------------------------------------------------------------
Function un.RemoveFromPath
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7

  StrCmp $0 "" un.RemoveFromPath_done

  ReadRegStr $1 HKCU "Environment" "Path"
  ; Si no hay PATH, no hay nada que hacer (no escribir, no notificar)
  StrCmp $1 "" un.RemoveFromPath_done

  ; Trabajar con ';' alrededor para coincidencias exactas
  StrCpy $2 ";$1;"
  StrCpy $3 ";$0;"

un.RemoveFromPath_loop:
  Push "$2"
  Push "$3"
  Call un.StrStr
  Pop $4
  ; $4 vacío = ya no hay ocurrencias
  StrCmp $4 "" un.RemoveFromPath_finish
  ; $4 = cola desde la coincidencia (";DIR;...resto")
  ; prefixLen = Len($2) - Len($4)
  StrLen $5 "$2"
  StrLen $6 "$4"
  IntOp $5 $5 - $6
  ; Len($3) para calcular el resto (conservando un ';')
  StrLen $6 "$3"
  ; prefijo = primeros $5 caracteres de $2
  StrCpy $7 "$2" $5
  ; resto = $4 sin los primeros Len($3)-1 caracteres
  IntOp $6 $6 - 1
  StrCpy $4 "$4" "" $6
  ; nuevo valor de trabajo = prefijo + resto
  StrCpy $2 "$7$4"
  Goto un.RemoveFromPath_loop

un.RemoveFromPath_finish:
  ; Quitar los ';' auxiliares del inicio y fin
  StrCmp $2 ";" un.RemoveFromPath_empty
  StrLen $5 "$2"
  IntOp $5 $5 - 2
  IntCmp $5 0 un.RemoveFromPath_empty un.RemoveFromPath_empty 0
  StrCpy $4 "$2" $5 1
  StrCpy $2 $4
  Goto un.RemoveFromPath_compare

un.RemoveFromPath_empty:
  StrCpy $2 ""

un.RemoveFromPath_compare:
  ; Solo escribir si realmente cambió (protege el resto del PATH)
  StrCmp $1 $2 un.RemoveFromPath_done
  WriteRegExpandStr HKCU "Environment" "Path" $2
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

un.RemoveFromPath_done:
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; ------------------------------------------------------------
;  StrStr - Busca una subcadena dentro de otra (insensible a mayúsculas,
;  necesario porque Windows compara rutas del PATH sin distinguir caso)
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
  ; StrCmp ya es insensible a mayusculas/minusculas (no lleva flag /ignorecase).
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

; ------------------------------------------------------------
;  un.StrStr - Versión para el desinstalador (las funciones del
;  instalador no son visibles desde la sección Uninstall)
; ------------------------------------------------------------
Function un.StrStr
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

un.StrStr_loop:
  StrCpy $4 $1 1 $5
  StrCmp $4 "" un.StrStr_NotFound
  StrCpy $4 $1 $2 $5
  ; StrCmp ya es insensible a mayusculas/minusculas (no lleva flag /ignorecase).
  StrCmp $4 $0 un.StrStr_Found
  IntOp $5 $5 + 1
  Goto un.StrStr_loop

un.StrStr_Found:
  StrCpy $3 $1 "" $5
  Goto un.StrStr_End

un.StrStr_NotFound:
  StrCpy $3 ""

un.StrStr_End:
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  StrCpy $1 $3
  Exch $1
  Exch
  Pop $0
FunctionEnd