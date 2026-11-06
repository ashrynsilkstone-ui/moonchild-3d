# Asset-Herkunft

Alle Assets dieses Repositories wurden für Moonchild selbst erzeugt. Es wurden keine fremden Bild-, Modell-, Logo- oder Sounddateien verwendet. Die Inhalte stehen unter der MIT-Lizenz des Repositories.

| Pfad | Herkunft | Werkzeug |
|---|---|---|
| `assets/textures/moonchild/*.png` | prozedural aus gesetztem Rauschen, Voronoi-Zellen und gezeichneten Linien berechnet | `tools/generate_textures.py` (Python, NumPy, Pillow) |
| `assets/textures/dj/*.png` | prozedural gezeichnete DJ-Faceplates. Die Beschriftung wird beim Erzeugen mit der lokal installierten freien Schrift DejaVu Sans gesetzt; die Schriftdatei wird nicht mitgeliefert | `tools/generate_dj_textures.py` |
| Club-Geometrie | zur Laufzeit erzeugte Meshes | `src/shapes.rs`, `src/club_section.rs`, `src/main.rs`, `src/dj_gear.rs` |
| Umgebungs-Cubemap | zur Laufzeit prozedural gemalt | `src/look.rs` (`environment_cubemap`) |
| `docs/screenshots/*.jpg` | In-Engine-Screenshots aus dem laufenden Spiel, keine Bildgenerierung | `src/capture.rs` |

Logos oder Markenzeichen aus Referenzbildern wurden nicht übernommen.

Laufzeitabhängigkeiten: Bevy und seine Crates unterliegen ihren eigenen Open-Source-Lizenzen (MIT/Apache-2.0). Bevys eingebettete Standard-UI-Schrift wird für die Bildschirmtexte verwendet.
