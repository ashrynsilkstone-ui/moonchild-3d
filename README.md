# MOONCHILD 3D · Version 0.1

Ein kostenloser, selbst gebauter begehbarer Techno-Club in Rust und Bevy 0.20.1. Die Clubgeometrie und Materialbilder sind für Moonchild erstellt; es werden keine kostenpflichtigen Modelle, Texturen oder Dienste benötigt.

## Start auf CachyOS / Arch Linux

```bash
sudo pacman -S --needed rustup base-devel clang lld pkgconf alsa-lib libx11 libxcursor libxi libxrandr wayland libxkbcommon vulkan-radeon vulkan-tools
rustup default stable
git clone https://github.com/ashrynsilkstone-ui/moonchild-3d.git
cd moonchild-3d
cargo run --release
```

Die Paketnamen und Treiber bitte bei Bedarf an die eigene Installation anpassen. `systemd-libs` stellt auf Arch normalerweise libudev bereit. Das erste Bauen lädt und kompiliert Bevy. Vulkan und ein Grafiktreiber sind erforderlich. Aus dem Projektverzeichnis starten, damit Bevy `assets/` findet.

## Steuerung

| Taste | Funktion |
|---|---|
| W A S D | Gehen, links hinten über die Treppe zur Galerie |
| Shift | Schneller gehen |
| Rechte Maustaste gedrückt + Maus | Umsehen |
| 1 / 2 / 3 | Neon / Rave / Chill |
| 4 | Strobe (nur manuell aktiv) |
| L | Show-Licht an/aus, Grundlicht bleibt an |
| + / - | Tempo und BPM ändern |
| Escape | Beenden |

## Club und Materialien

Der erste ausgebaute Bereich umfasst eine geschwungene Galerie mit Rohrgeländer, eine Treppe, zwei gepolsterte Lounges, eine halbrunde Bar mit Flaschenregal, ein abgerundetes DJ-Pult sowie einen beleuchteten Deckenring. Der bestehende Raum, die Bühne, LED-Wand und Lichtprogramme bleiben erhalten. Die Tanzflächenstreifen liegen über dem Bodenaufsatz; die senkrechten Deckenspots verwenden eine geeignete Aufwärtsrichtung. Zwei warme Grundlichter werfen Schatten, die übrigen bleiben aus Leistungsgründen ohne Schatten. HDR und natürliches Bloom bleiben aktiv.

Fünf eigene 256×256-PNG-Materialsets für Beton, Stahl, Boden, Leder und geprägtes Metallgitter liegen unter `assets/textures/moonchild/`. Sie enthalten jeweils Base Color, eine aus einem periodischen Höhenfeld abgeleitete Tangent-Space-Normalmap und eine kombinierte ORM-Datei (R: AO konstant weiß, G: Roughness, B: Metallic). Das Gitter ist eine geprägte Oberfläche ohne echte Löcher. Der Generator `tools/generate_textures.py` erstellt die Dateien deterministisch; dafür werden Python, NumPy und Pillow benötigt, zum Spielen nicht. Die Gitternetz-, Boden- und Materialskalen sind ein erster Ansatz und brauchen Sichtprüfung im laufenden Spiel.

Die Geometrie, das Generatorskript und die erzeugten Texturen sind eigenständig für Moonchild erstellt und stehen unter der MIT-Lizenz dieses Repositories. Es wurden keine fremden Bild- oder Modell-Assets verwendet. Bevy und seine Abhängigkeiten unterliegen ihren jeweiligen eigenen Open-Source-Lizenzen.

## Grenzen und weitere Arbeit

Die Bewegung prüft die Raumgrenzen, Bühne, Galerie, Treppe und vereinfachte Hindernisbereiche. Es ist keine vollständige Physik und keine präzise Mesh-Kollision; Geländer und Möbel sind nur näherungsweise berücksichtigt. Keine echte Audiosynchronisierung, keine Sound-Dateien, keine volumetrischen Laser, keine NPCs und kein Multiplayer. Eine Umgebungskarte, Screen-Space-Reflexionen oder Raytracing sind nicht eingerichtet; punktuelle PBR-Glanzlichter sind keine vollständige Spiegelung. Visuelle Qualität, Schatten, UV-Nähte und Bildrate müssen auf dem Zielsystem geprüft werden.

## Prüfungen

```bash
cargo check
cargo test
cargo run --release
```

Der unveränderte Ausgangsstand `06a53ca` hat `cargo check` auf einem Linux-CI-Runner bestanden. Ergebnisse für spätere Änderungen siehe Commit- und CI-Status. Eine erfolgreiche CI-Prüfung ersetzt keine Sicht- und Steuerungsprüfung auf CachyOS mit der RX 9070 XT.

## Lizenz

MIT – siehe LICENSE.
