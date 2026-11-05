@echo off
rem Gemeinsame Vorbereitung fuer start.bat und server_start.bat:
rem Rust installieren, falls es fehlt, dazu die C++-Build-Tools von Microsoft (Linker).
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where cargo >nul 2>nul
if not errorlevel 1 goto have_cargo
echo ==^> Rust ist nicht installiert.
where winget >nul 2>nul
if errorlevel 1 goto no_winget
choice /C JN /M "Jetzt mit winget installieren (ca. 1-2 GB, nur einmal)"
if errorlevel 2 goto no_rust
winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul
if errorlevel 1 goto restart_needed

:have_cargo
rem Ohne Standard-Toolchain meldet cargo nur Fehler
rustup default >nul 2>nul
if errorlevel 1 rustup default stable

rem Der MSVC-Linker kommt mit den Visual-Studio-Build-Tools
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VSPATH="
if not exist "%VSWHERE%" goto need_vs
for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSPATH=%%i"
if defined VSPATH exit /b 0

:need_vs
echo ==^> Es fehlen die C++-Build-Tools von Microsoft, die Rust zum Bauen braucht.
where winget >nul 2>nul
if errorlevel 1 goto no_winget_vs
choice /C JN /M "Jetzt mit winget installieren (ca. 2-4 GB, nur einmal)"
if errorlevel 2 goto no_vs
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-source-agreements --accept-package-agreements --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
exit /b 0

:no_winget
echo Bitte Rust von https://rustup.rs installieren: rustup-init.exe starten, Standardoptionen nehmen.
echo Danach dieses Skript nochmal starten.
start "" https://rustup.rs
exit /b 1

:no_winget_vs
echo Bitte die "Build Tools fuer Visual Studio" mit "Desktopentwicklung mit C++" installieren:
echo https://visualstudio.microsoft.com/de/visual-cpp-build-tools/
start "" https://visualstudio.microsoft.com/de/visual-cpp-build-tools/
exit /b 1

:restart_needed
echo Rust ist installiert. Bitte dieses Fenster schliessen und das Skript nochmal starten.
exit /b 1

:no_rust
echo Ohne Rust geht es nicht: https://rustup.rs
exit /b 1

:no_vs
echo Ohne die Build-Tools laesst sich das Spiel nicht bauen.
exit /b 1
