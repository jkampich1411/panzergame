#!/usr/bin/env bash
# Panzer starten (Linux und macOS). Beim ersten Mal wird alles Nötige installiert und
# gebaut, danach startet das Spiel sofort.
#
#   ./start.sh                     Menü
#   ./start.sh solo 3              direkt gegen 3 Bots
#   ./start.sh join 100.64.0.5     direkt beitreten
#   ./start.sh host 5555 2         direkt hosten mit 2 Bots

set -euo pipefail
cd "$(dirname "$0")"
# shellcheck source=scripts/setup.sh
. scripts/setup.sh

prepare game
say "Baue das Spiel (beim ersten Mal ein paar Minuten, danach Sekunden) …"
cargo build --release -p panzer
exec ./target/release/panzer "$@"
