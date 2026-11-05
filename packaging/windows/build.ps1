# Baut alle Windows-Downloads nach dist\ (braucht Rust mit MSVC und Inno Setup):
#   Panzer-Setup-windows-x86_64.exe     Installer des Launchers
#   panzer-game-windows-x86_64.zip      Spielpaket, das der Launcher lädt
#   panzer-server-windows-x86_64.zip    dedizierter Server
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\..")

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"$' | Select-Object -First 1).Matches.Groups[1].Value
if (-not $version) { throw "Version nicht in Cargo.toml gefunden" }

cargo build --release --locked -p panzer -p panzer-launcher -p panzer-server
if ($LASTEXITCODE -ne 0) { throw "cargo build fehlgeschlagen" }

New-Item -ItemType Directory -Force dist | Out-Null
Compress-Archive -Force -Path target\release\panzer.exe -DestinationPath dist\panzer-game-windows-x86_64.zip
Compress-Archive -Force -Path target\release\panzer-server.exe -DestinationPath dist\panzer-server-windows-x86_64.zip

iscc "/DAppVersion=$version" packaging\windows\panzer.iss
if ($LASTEXITCODE -ne 0) { throw "Inno Setup fehlgeschlagen" }

Get-ChildItem dist
