#!/usr/bin/env python3
"""Generate an original Moonchild bottle sleeve; no external image assets."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
out = Path(__file__).resolve().parent.parent / 'assets/textures/moonchild/bottle_label.png'
out.parent.mkdir(parents=True, exist_ok=True)
im = Image.new('RGB', (1024, 512), (22, 15, 21))
d = ImageDraw.Draw(im)
for y in range(512):
    t = abs(y - 256) / 256
    d.line((0, y, 1024, y), fill=(int(40 - 12*t), int(20 - 5*t), int(31 - 8*t)))
for y in (30, 42, 469, 481):
    d.line((0, y, 1024, y), fill=(169, 131, 74), width=3)
for x in (0, 1023):
    d.line((x, 65, x, 447), fill=(83, 64, 48), width=3)
font_path = next((p for p in [Path('/usr/share/fonts/TTF/DejaVuSans.ttf'),
                               Path('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf')] if p.is_file()), None)
if font_path is None:
    raise SystemExit('DejaVu Sans fehlt: installiere das DejaVu-Schriftpaket.')
font = ImageFont.truetype(str(font_path), 77)
small = ImageFont.truetype(str(font_path), 29)
d.text((512, 220), 'MOONCHILD', font=font, anchor='mm', fill=(224, 205, 161), stroke_width=0)
d.text((512, 305), 'N O C T U R N E', font=small, anchor='mm', fill=(180, 152, 102))
im.save(out, optimize=True)
