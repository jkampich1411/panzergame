#!/usr/bin/env bash
# macOS: Doppelklick im Finder startet den Server im Terminal
cd "$(dirname "$0")" || exit 1
./server_start.sh "$@" || { echo; read -r -p "Etwas ist schiefgegangen (siehe oben). Enter zum Schließen …" _; }
