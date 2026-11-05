#!/usr/bin/env bash
# Gemeinsame Vorbereitung für start.sh und server_start.sh (Linux und macOS):
# Rust installieren, falls es fehlt, und unter Linux die Bibliotheken fürs Spiel.

set -euo pipefail

say() { printf '\033[1;33m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31mFehler:\033[0m %s\n' "$*" >&2; exit 1; }

# Ja/Nein-Frage, Standard ja. Ohne Terminal (z. B. Doppelklick ohne Konsole): ja.
ask() {
    local answer
    if [ ! -t 0 ]; then return 0; fi
    read -r -p "$1 [J/n] " answer || return 0
    case "$answer" in [nN]*) return 1 ;; *) return 0 ;; esac
}

ensure_rust() {
    # rustup legt cargo nach ~/.cargo/bin, das steht nicht immer im PATH
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
    fi
    if command -v cargo >/dev/null 2>&1; then return; fi
    say "Rust (cargo) ist nicht installiert."
    ask "Jetzt über rustup.rs installieren (ca. 1–2 GB, nur einmal)?" || fail "Ohne Rust geht es nicht: https://rustup.rs"
    command -v curl >/dev/null 2>&1 || fail "curl fehlt. Bitte zuerst curl installieren."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
}

ensure_macos_tools() {
    if xcode-select -p >/dev/null 2>&1; then return; fi
    say "Die Xcode-Kommandozeilen-Tools fehlen (Compiler und Linker)."
    xcode-select --install || true
    fail "Bitte die Installation im aufgehenden Fenster abschließen und das Skript danach nochmal starten."
}

# Bibliotheken fürs Spiel (Ton und Fenster). Der Server braucht sie nicht.
ensure_linux_game_deps() {
    local missing=() pkgs=() install=()
    if command -v apt-get >/dev/null 2>&1; then
        pkgs=(libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev pkg-config)
        for p in "${pkgs[@]}"; do
            dpkg -s "$p" >/dev/null 2>&1 || missing+=("$p")
        done
        install=(sudo apt-get install -y)
    elif command -v dnf >/dev/null 2>&1; then
        pkgs=(alsa-lib-devel libX11-devel libXi-devel mesa-libGL-devel pkgconf-pkg-config)
        for p in "${pkgs[@]}"; do
            rpm -q "$p" >/dev/null 2>&1 || missing+=("$p")
        done
        install=(sudo dnf install -y)
    elif command -v pacman >/dev/null 2>&1; then
        pkgs=(alsa-lib libx11 libxi mesa pkgconf)
        for p in "${pkgs[@]}"; do
            pacman -Q "$p" >/dev/null 2>&1 || missing+=("$p")
        done
        install=(sudo pacman -S --needed --noconfirm)
    elif command -v zypper >/dev/null 2>&1; then
        pkgs=(alsa-devel libX11-devel libXi-devel Mesa-libGL-devel pkg-config)
        for p in "${pkgs[@]}"; do
            rpm -q "$p" >/dev/null 2>&1 || missing+=("$p")
        done
        install=(sudo zypper install -y)
    else
        say "Unbekannte Distribution: falls der Build scheitert, ALSA-, X11-, Xi- und OpenGL-Entwicklerpakete installieren."
        return
    fi
    # cc (Linker) braucht Rust auch
    if ! command -v cc >/dev/null 2>&1; then
        if command -v apt-get >/dev/null 2>&1; then missing+=(build-essential); else missing+=(gcc); fi
    fi
    if [ ${#missing[@]} -eq 0 ]; then return; fi
    say "Es fehlen Pakete fürs Spiel: ${missing[*]}"
    ask "Jetzt installieren (braucht sudo)?" || fail "Ohne diese Pakete lässt sich das Spiel nicht bauen."
    "${install[@]}" "${missing[@]}"
}

ensure_linux_cc() {
    if command -v cc >/dev/null 2>&1; then return; fi
    say "Es fehlt ein C-Compiler/Linker (cc), den Rust zum Bauen braucht."
    if command -v apt-get >/dev/null 2>&1; then
        ask "build-essential installieren (braucht sudo)?" && sudo apt-get install -y build-essential && return
    elif command -v dnf >/dev/null 2>&1; then
        ask "gcc installieren (braucht sudo)?" && sudo dnf install -y gcc && return
    elif command -v pacman >/dev/null 2>&1; then
        ask "base-devel installieren (braucht sudo)?" && sudo pacman -S --needed --noconfirm base-devel && return
    fi
    fail "Bitte gcc (bzw. build-essential) installieren."
}

# Aufruf: prepare game|server
prepare() {
    case "$(uname -s)" in
        Darwin) ensure_macos_tools ;;
        Linux)
            if [ "$1" = game ]; then ensure_linux_game_deps; else ensure_linux_cc; fi
            ;;
    esac
    ensure_rust
}
