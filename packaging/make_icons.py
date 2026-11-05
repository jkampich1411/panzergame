#!/usr/bin/env python3
"""Erzeugt aus assets/icon/panzer-1024.png die Symbole für alle Systeme.

    cargo test -p panzer --release export_icon -- --ignored   # panzer-1024.png neu rendern
    python3 packaging/make_icons.py                            # braucht Pillow
"""

from pathlib import Path

from PIL import Image

ICON = Path(__file__).resolve().parent.parent / "assets" / "icon"
src = Image.open(ICON / "panzer-1024.png").convert("RGBA")

for size in (16, 32, 48, 64, 128, 256, 512):
    src.resize((size, size), Image.LANCZOS).save(ICON / f"panzer-{size}.png")
src.save(ICON / "panzer.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
src.save(ICON / "panzer.icns")
print("fertig:", ", ".join(sorted(p.name for p in ICON.iterdir())))
