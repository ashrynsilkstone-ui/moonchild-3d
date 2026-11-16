#!/usr/bin/env python3
"""Original, periodic Moonchild PBR maps. No external assets (MIT)."""
from pathlib import Path
import numpy as np
from PIL import Image

N = 1024
out = Path(__file__).resolve().parent.parent / 'assets/textures/moonchild'
out.mkdir(parents=True, exist_ok=True)
y, x = np.mgrid[0:N, 0:N].astype(np.float32) / N

def waves(seed, count, low, high):
    rng = np.random.default_rng(seed)
    result = np.zeros((N, N), np.float32)
    for _ in range(count):
        fx, fy = rng.integers(low, high + 1, 2)
        phase = rng.uniform(0, np.pi * 2)
        result += np.sin(2 * np.pi * (fx*x + fy*y) + phase)
    return result / np.sqrt(count)

def save_set(name, color, height, roughness, metallic):
    dx = (np.roll(height, -1, 1) - np.roll(height, 1, 1)) * 2.5
    dy = (np.roll(height, -1, 0) - np.roll(height, 1, 0)) * 2.5
    norm = np.stack((-dx, dy, np.ones_like(dx)), axis=-1)
    norm /= np.maximum(np.linalg.norm(norm, axis=-1, keepdims=True), 1e-6)
    normal = np.uint8(np.clip((norm * .5 + .5) * 255, 0, 255))
    ao = np.clip(248 + height * 6, 220, 255)
    orm = np.stack((ao, roughness, np.full_like(height, metallic)), axis=-1)
    for suffix, arr in [('base', color), ('normal', normal), ('orm', orm)]:
        im = Image.fromarray(np.uint8(np.clip(arr,0,255)))
        # Indexed PNG expands to RGBA on load, reducing the size without external encoders.
        im.quantize(colors=128, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(out / f'{name}_{suffix}.png', optimize=True)

# Pebbled burgundy leather: multiple periodic bands and fine grain; the normal
# is made from the very same height field as the base and roughness variations.
fine = waves(731, 36, 30, 150)
broad = waves(953, 18, 2, 12)
creases = np.sin(2*np.pi*(5*x+3*y)+0.6*np.sin(2*np.pi*y*2))
height = 0.60*fine + 0.22*broad + 0.12*creases
v = np.clip(1 + 0.10*fine + 0.08*broad, .65, 1.35)
color = np.stack((81*v+4, 20*v+2, 29*v+3),axis=-1)
rough = np.clip(132 + 16*fine + 12*broad, 96, 176)
save_set('leather_burgundy', color, height, rough, 0)

# Tight woven charcoal fabric for recessed acoustic panels; an intentionally
# rough, nonmetallic material with separately sampled warp and weft.
warp = np.cos(2*np.pi*128*x)
weft = np.cos(2*np.pi*128*y)
weave = (warp*weft + 0.35*warp + 0.35*weft) * 0.28
variation = waves(1307, 12, 2, 16)
height = weave + 0.08*variation
v = np.clip(1 + 0.18*weave + 0.04*variation, .7, 1.3)
color = np.stack((65*v, 61*v, 67*v),axis=-1)
rough = np.clip(215 + 12*weave, 190, 235)
save_set('acoustic_charcoal', color, height, rough, 0)
