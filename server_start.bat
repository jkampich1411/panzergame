@echo off
rem Dedizierten Panzer-Server starten (Windows), ohne Grafik.
rem
rem   server_start.bat                                  Port 5555, keine Bots
rem   server_start.bat --bots 3 --admin-pass geheim     freie Plaetze mit Bots, Admin per /login
rem
rem Konsolenbefehle (ohne /): players, kick, ban, unban, bans, mute, unmute, bots, say, help
rem Freigeben bzw. weiterleiten: UDP-Port 5555. Die Windows-Firewall fragt beim ersten Start.
cd /d "%~dp0"
call scripts\setup.bat
if errorlevel 1 goto fail
echo ==^> Baue den Server ...
cargo build --release -p panzer-server
if errorlevel 1 goto fail
"%~dp0target\release\panzer-server.exe" %*
pause
exit /b 0

:fail
echo.
echo Etwas ist schiefgegangen (siehe oben).
pause
exit /b 1
