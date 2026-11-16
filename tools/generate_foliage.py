#!/usr/bin/env python3
"""Moonchild leaf maps: original deterministic procedural artwork (MIT)."""
from pathlib import Path
import numpy as np
from PIL import Image

N = 1024
out = Path(__file__).resolve().parent.parent / 'assets/textures/moonchild'
out.mkdir(parents=True, exist_ok=True)
y, x = np.mgrid[0:N, 0:N].astype(np.float32) / (N - 1)
# One leaf mapped to the whole UV rectangle; edge shapes come from the mesh.
vein = np.exp(-((x - 0.5) / 0.007) ** 2)
phase = (y * 19 + np.abs(x - 0.5) * 8) % 1
side_vein = np.exp(-((phase - 0.5) / 0.055) ** 2) * (0.4 + 0.6 * np.abs(x - 0.5))
texture = 0.5 + 0.5 * np.sin(x * 110 + np.sin(y * 40) * 2) * np.sin(y * 88)
height = 0.35 * vein + 0.12 * side_vein + 0.025 * texture
base = np.stack([30 + 16 * texture + 34 * vein + 9 * side_vein,
                 72 + 25 * texture + 43 * vein + 16 * side_vein,
                 39 + 12 * texture + 14 * vein + 5 * side_vein], axis=-1)
dy, dx = np.gradient(height)
n = np.stack([-dx * 3, -dy * 3, np.ones_like(x)], axis=-1)
n /= np.linalg.norm(n, axis=-1, keepdims=True)
normal = (n * 0.5 + 0.5) * 255
orm = np.stack([np.full_like(x, 255), 162 + 20 * texture - 8 * vein, np.zeros_like(x)], axis=-1)
for suffix, pixels in [('base', base), ('normal', normal), ('orm', orm)]:
    # Quantisation makes the repository-friendly PNGs compact, while the loader
    # expands them to RGBA for mipmaps and linear normal/ORM sampling.
    img = Image.fromarray(np.uint8(np.clip(pixels, 0, 255)), 'RGB')
    img.quantize(colors=32, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(out / f'foliage_{suffix}.png', optimize=True)
