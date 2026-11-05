#!/usr/bin/env bash
# macOS: Doppelklick im Finder startet das Spiel im Terminal
cd "$(dirname "$0")" || exit 1
./start.sh "$@" || { echo; read -r -p "Etwas ist schiefgegangen (siehe oben). Enter zum Schließen …" _; }
