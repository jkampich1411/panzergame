#!/usr/bin/env bash
# Dedizierten Panzer-Server starten (Linux und macOS), ohne Grafik.
#
#   ./server_start.sh                                   Port 5555, keine Bots
#   ./server_start.sh --bots 3 --admin-pass geheim      freie Plätze mit Bots, Admin per /login
#   ./server_start.sh --port 6000
#
# Konsolenbefehle (ohne /): players, kick, ban, unban, bans, mute, unmute, bots, say, help
# Freigeben bzw. weiterleiten: UDP-Port 5555 (oder der gewählte).

set -euo pipefail
cd "$(dirname "$0")"
# shellcheck source=scripts/setup.sh
. scripts/setup.sh

prepare server
say "Baue den Server …"
cargo build --release -p panzer-server
exec ./target/release/panzer-server "$@"
