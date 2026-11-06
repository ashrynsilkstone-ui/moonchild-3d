# MOONCHILD 3D · Version 0.1

Ein kostenloser, selbst gebauter begehbarer Techno-Club in Rust und Bevy 0.20.1. Die Clubgeometrie und Materialbilder sind für Moonchild erstellt; es werden keine kostenpflichtigen Modelle, Texturen oder Dienste benötigt.

## Start auf CachyOS / Arch Linux

```bash
sudo pacman -S --needed rustup base-devel clang lld pkgconf alsa-lib libx11 libxcursor libxi libxrandr wayland libxkbcommon libxkbcommon-x11 vulkan-radeon vulkan-tools
rustup default stable
git clone https://github.com/ashrynsilkstone-ui/moonchild-3d.git
cd moonchild-3d
cargo run --release
```

Die Paketnamen und Treiber bitte bei Bedarf an die eigene Installation anpassen. `systemd-libs` stellt auf Arch normalerweise libudev bereit. Das erste Bauen lädt und kompiliert Bevy. Vulkan und ein Grafiktreiber sind erforderlich. `cargo run --release` aus dem Projektverzeichnis starten. Beim direkten Start der Binärdatei muss der Asset-Pfad gesetzt werden, zum Beispiel `BEVY_ASSET_ROOT="$PWD" target/release/moonchild-3d` aus dem Projektverzeichnis.

## Steuerung

| Taste | Funktion |
|---|---|
| W A S D | Gehen, links hinten über die Treppe zur Galerie |
| Shift | Schneller gehen |
| Leertaste | Springen, beispielsweise auf die Bühne |
| Rechte Maustaste gedrückt + Maus | Umsehen |
| 1 / 2 / 3 | Neon / Rave / Chill |
| 4 | Strobe (nur manuell aktiv) |
| L | Show-Licht an/aus, Grundlicht bleibt an |
| + / - | Tempo und BPM ändern |
| Escape | Beenden |

## Club und Materialien

Die DJ-Bühne ist 0,96 m hoch und per Sprung erreichbar. Springen und Landen berücksichtigen Bühnenkante, Galerieunterseite und einfache Hindernisse. Die Controller bestehen aus zwei Media-Playern und einem Vierkanal-Mixer mit eigenen Texturdateien sowie räumlichen Jogwheel-Ringen, Reglern und Faderkappen. Displays und einzelne LED-Anzeigen besitzen zusätzlich eigene Emissionstexturen und werden von einer Pultleuchte unterstützt. Die Bedienelemente und Bildschirmgrafiken sind derzeit rein visuell und steuern keine Musik.

Der erste ausgebaute Bereich umfasst eine geschwungene Galerie mit Rohrgeländer, eine Treppe, zwei gepolsterte Lounges, eine halbrunde Bar mit Flaschenregal, ein abgerundetes DJ-Pult sowie einen beleuchteten Deckenring. Der bestehende Raum, die Bühne, LED-Wand und Lichtprogramme bleiben erhalten. Die Tanzflächenstreifen liegen über dem Bodenaufsatz; die senkrechten Deckenspots verwenden eine geeignete Aufwärtsrichtung. Zwei warme Grundlichter werfen Schatten, die übrigen bleiben aus Leistungsgründen ohne Schatten. HDR und natürliches Bloom bleiben aktiv.

Fünf eigene 256×256-PNG-Materialsets für Beton, Stahl, Boden, Leder und geprägtes Metallgitter liegen unter `assets/textures/moonchild/`. Sie enthalten jeweils Base Color, eine aus einem periodischen Höhenfeld abgeleitete Tangent-Space-Normalmap und eine kombinierte ORM-Datei (R: AO konstant weiß, G: Roughness, B: Metallic). Das Gitter ist eine geprägte Oberfläche ohne echte Löcher. Der Generator `tools/generate_textures.py` erstellt die Dateien deterministisch; dafür werden Python, NumPy und Pillow benötigt, zum Spielen nicht. Die Gitternetz-, Boden- und Materialskalen sind ein erster Ansatz und brauchen Sichtprüfung im laufenden Spiel.

Die Geometrie, die Generatorskripte und die erzeugten Texturen sind eigenständig für Moonchild erstellt und stehen unter der MIT-Lizenz dieses Repositories. Die Controller-Beschriftungen werden beim Generieren mit der auf dem Erzeugungssystem installierten freien Schrift DejaVu Sans gesetzt; die Schriftdatei selbst wird nicht mitgeliefert. `tools/generate_dj_textures.py` erzeugt die beiden Faceplates mit Pillow. Es wurden keine fremden Bild- oder Modell-Assets verwendet. Bevy und seine Abhängigkeiten unterliegen ihren jeweiligen eigenen Open-Source-Lizenzen.

## Grenzen und weitere Arbeit

Die Bewegung prüft die Raumgrenzen, Bühne, Galerie, Treppe und vereinfachte Hindernisbereiche. Es ist keine vollständige Physik und keine präzise Mesh-Kollision; Geländer und Möbel sind nur näherungsweise berücksichtigt. Keine echte Audiosynchronisierung, keine Sound-Dateien, keine volumetrischen Laser, keine NPCs und kein Multiplayer. Eine Umgebungskarte, Screen-Space-Reflexionen oder Raytracing sind nicht eingerichtet; punktuelle PBR-Glanzlichter sind keine vollständige Spiegelung. Visuelle Qualität, Schatten, UV-Nähte und Bildrate müssen auf dem Zielsystem geprüft werden.

## Prüfungen

```bash
cargo check
cargo test
cargo run --release
```

Am 6. November 2026 wurden auf einem Ubuntu-CI-Runner `cargo check`, drei Navigationstests mit `cargo test` und `cargo build` für den Stand mit Texturen, Galerie und LED-Korrektur erfolgreich ausgeführt ([Prüflauf](https://github.com/ashrynsilkstone-ui/moonchild-3d/actions/runs/42154079047)). Das Spiel wurde außerdem mit Mesa llvmpipe unter virtuellem X11 gestartet; die Protokolle zeigen keine fehlenden Texturen. Die Bilder zeigten eine zu dunkle Umgebung. Die anschließend angehobene Grundhelligkeit war bei Dokumentation dieses Tests noch nicht visuell bestätigt. Eine CI-Prüfung ersetzt keine Sicht- und Steuerungsprüfung auf CachyOS mit der RX 9070 XT.

## Lizenz

MIT – siehe LICENSE.

Am 6. November 2026 wurden `cargo check`, fünf Navigationstests und `cargo build` für die Bühnen- und Geräteänderung erfolgreich auf einem Linux-Runner ausgeführt ([Prüflauf](https://github.com/ashrynsilkstone-ui/moonchild-3d/actions/runs/42240788218)). Ein Grafiklauf mit Software-Vulkan und simulierten Tastatureingaben erreichte die Bühne und zeigte die texturierten Geräte samt Pultleuchte und Emissionskarten. Das Protokoll enthielt keine fehlenden Assets. Die Steuerung und Bildwirkung auf der Zielgrafikkarte müssen weiterhin dort geprüft werden.
