@echo off
rem Panzer starten (Windows). Beim ersten Mal wird alles Noetige installiert und gebaut,
rem danach startet das Spiel sofort. Doppelklick genuegt.
rem
rem   start.bat                    Menue
rem   start.bat solo 3             direkt gegen 3 Bots
rem   start.bat join 100.64.0.5    direkt beitreten
cd /d "%~dp0"
call scripts\setup.bat
if errorlevel 1 goto fail
echo ==^> Baue das Spiel (beim ersten Mal ein paar Minuten, danach Sekunden) ...
cargo build --release -p panzer
if errorlevel 1 goto fail
start "" "%~dp0target\release\panzer.exe" %*
exit /b 0

:fail
echo.
echo Etwas ist schiefgegangen (siehe oben).
pause
exit /b 1
