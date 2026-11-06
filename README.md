# MOONCHILD 3D · Version 0.1

Ein kostenloser, selbst gebauter begehbarer Techno-Club in **Rust / Bevy 0.20.1**. Alle 3D-Objekte sind prozedural erstellt. Keine externen Modelle, kostenpflichtigen Assets oder Musikrechte erforderlich.

## Voraussetzungen (CachyOS / Arch Linux)

```bash
sudo pacman -S --needed rustup base-devel clang lld pkgconf alsa-lib libx11 libxcursor libxi libxrandr vulkan-tools
rustup default stable
```

## Starten

```bash
git clone https://github.com/ashrynsilkstone-ui/moonchild-3d.git
cd moonchild-3d
cargo run --release
```

Das erstmalige Bauen lädt Bevy und seine Abhängigkeiten herunter und kompiliert sie. Folge-Starts sind schneller. Vulkan-fähige Grafiktreiber vorausgesetzt.

## Steuerung

| Taste | Funktion |
|---|---|
| W A S D | Gehen |
| Shift | Schneller gehen |
| Rechte Maustaste gedrückt + Maus | Umsehen |
| 1 | Neon / Violett-Cyan |
| 2 | Rave / Rot-Blau |
| 3 | Chill / Blau-Türkis |
| 4 | Strobe / Blitzmodus |
| L | Licht an/aus |
| + / - | Tempo und BPM ändern |
| Escape | Beenden |

## Enthalten

- 24 × 20 m Clubraum, Wände, Decke, Träger und Säulen
- DJ-Bühne, Decks, Mixer, Lautsprecher, animierte LED-Wand
- Tanzfläche mit Neonlinien, Podeste, Side-Lightings
- 5 animierte bewegliche Spotlights und 4 pulsierende Point-Lights
- HDR und Bloom für leuchtende Materialien
- First-Person-Steuerung; einfache Begrenzung an Außenwänden
- Vier Lichtprogramme, BPM- und Geschwindigkeitsregler
- Rust-Quellcode, ohne benötigte externe Assets

## Was noch fehlt

Keine echte Audiosynchronisierung, keine Sound-Dateien, kein physikalisch korrektes Laser-Volumen, keine NPCs, keine detaillierten importierten 3D-Modelle, keine Multiplayer-Logik. Die Kamera nutzt vorerst einfache Raumgrenzen statt vollständiger Kollisionsphysik. Ein 3D-Bild allein wird damit noch nicht 1:1 reproduziert; dafür wären detaillierte Modelle, Texturen und eventuell Nebelshader nötig.

## Nächste sinnvolle Ausbaustufen

v0.2: Umkleide/Foyer, Bar und Lounge, echte Kollisionsphysik, GLB-Modelle, in-game Lichtpult.

v0.3: Freie, rechtssicher verwendbare Musik / eigene Tracks, beatgesteuertes Licht, visuelle Nebel- und Laser-Shader.

v0.4: DJ-Animationen, Besucher-NPCs und speicherbare Club-Konfigurationen.

## Lizenz

MIT — siehe LICENSE. Alle hier erzeugten Code-/Geometrie-Inhalte sind frei verwendbar.

*Hinweis: Prototyp bisher nicht auf dem Zielsystem kompiliert.*
