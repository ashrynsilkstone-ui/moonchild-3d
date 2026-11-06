#!/usr/bin/env python3
"""Generate Moonchild's own tileable PBR texture sets (MIT licence).

Every texture is computed from deterministic, seeded noise. Nothing is
downloaded, photographed or copied from third-party libraries.

Output per material in assets/textures/moonchild/:
  <name>_base.png    sRGB base colour (RGBA where the material has holes)
  <name>_normal.png  linear tangent-space normal map, OpenGL/glTF convention (+Y = up in the image)
  <name>_orm.png     linear R = ambient occlusion, G = roughness, B = metallic (glTF/Bevy layout)
Optional:
  <name>_emissive.png sRGB emission mask

All maps are periodic, so they tile without visible seams. Requires numpy and Pillow;
neither is needed to play the game.
"""
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / 'assets/textures/moonchild'
OUT.mkdir(parents=True, exist_ok=True)


# ---------------------------------------------------------------- helpers
def freq_grid(n):
    f = np.fft.fftfreq(n) * n
    fy, fx = np.meshgrid(f, f, indexing='ij')
    return fx, fy, np.sqrt(fx * fx + fy * fy)


def spectral(n, seed, beta=2.0, fmin=1.0, fmax=None, aniso=(1.0, 1.0)):
    """Periodic 1/f^beta noise, normalised to zero mean and unit deviation."""
    rng = np.random.default_rng(seed)
    fx, fy, _ = freq_grid(n)
    r = np.sqrt((fx * aniso[0]) ** 2 + (fy * aniso[1]) ** 2)
    amp = np.where(r >= fmin, 1.0 / np.maximum(r, 1e-6) ** (beta / 2), 0.0)
    if fmax:
        amp *= np.exp(-(r / fmax) ** 2)
    spec = np.fft.fft2(rng.standard_normal((n, n))) * amp
    out = np.real(np.fft.ifft2(spec))
    return (out - out.mean()) / (out.std() + 1e-9)


def blur(img, sigma_x, sigma_y=None):
    """Periodic Gaussian blur through the FFT (keeps maps tileable)."""
    sigma_y = sigma_x if sigma_y is None else sigma_y
    n = img.shape[0]
    fx, fy, _ = freq_grid(n)
    g = np.exp(-2 * (np.pi ** 2) * ((fx / n * sigma_x) ** 2 + (fy / n * sigma_y) ** 2))
    return np.real(np.fft.ifft2(np.fft.fft2(img) * g))


def voronoi(n, cells, seed):
    """Tileable jittered-grid Voronoi: returns F1, F2 distances (in cell units) and cell id."""
    rng = np.random.default_rng(seed)
    jitter = rng.uniform(0.08, 0.92, size=(cells, cells, 2))
    ids = rng.permutation(cells * cells).reshape(cells, cells)
    y, x = np.mgrid[0:n, 0:n] * (cells / n)
    cx, cy = np.floor(x).astype(int), np.floor(y).astype(int)
    f1 = np.full((n, n), 9.0); f2 = np.full((n, n), 9.0); cid = np.zeros((n, n), dtype=int)
    for oy in (-1, 0, 1):
        for ox in (-1, 0, 1):
            gx, gy = cx + ox, cy + oy
            wx, wy = gx % cells, gy % cells
            px = gx + jitter[wy, wx, 0]; py = gy + jitter[wy, wx, 1]
            d = np.sqrt((px - x) ** 2 + (py - y) ** 2)
            closer = d < f1
            f2 = np.where(closer, f1, np.minimum(f2, d))
            cid = np.where(closer, ids[wy, wx], cid)
            f1 = np.where(closer, d, f1)
    return f1, f2, cid


def wrapped_lines(n, segments, width, seed=None):
    """Draw anti-aliased line segments onto a periodic canvas (supersampled 2x)."""
    s = 2
    img = Image.new('L', (n * s, n * s), 0)
    d = ImageDraw.Draw(img)
    for (x0, y0, x1, y1, value) in segments:
        for ox in (-n, 0, n):
            for oy in (-n, 0, n):
                d.line([((x0 + ox) * s, (y0 + oy) * s), ((x1 + ox) * s, (y1 + oy) * s)],
                       fill=int(value * 255), width=max(1, int(width * s)))
    return np.asarray(img.resize((n, n), Image.LANCZOS), dtype=float) / 255.0


def normal_map(h, strength):
    """Height in pixels-relative units -> tangent-space normal (green up)."""
    dx = (np.roll(h, -1, 1) - np.roll(h, 1, 1)) * 0.5 * strength
    dy = (np.roll(h, -1, 0) - np.roll(h, 1, 0)) * 0.5 * strength
    n = np.stack((-dx, dy, np.ones_like(h)), axis=-1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    return np.clip((n * 0.5 + 0.5) * 255 + 0.5, 0, 255).astype(np.uint8)


def cavity_ao(h, radius, strength):
    """Approximate ambient occlusion: how far a texel lies below its neighbourhood."""
    diff = blur(h, radius) - h
    return np.clip(1.0 - strength * np.maximum(diff, 0), 0.0, 1.0)


def srgb(lin):
    lin = np.clip(lin, 0, 1)
    return np.where(lin <= 0.0031308, lin * 12.92, 1.055 * lin ** (1 / 2.4) - 0.055)


def lin(c):
    c = np.asarray(c, dtype=float) / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def save_set(name, albedo_lin, height, normal_strength, rough, metal, ao, alpha=None, emissive=None):
    rgb = (srgb(albedo_lin) * 255 + 0.5).astype(np.uint8)
    if alpha is not None:
        rgba = np.dstack((rgb, (np.clip(alpha, 0, 1) * 255 + 0.5).astype(np.uint8)))
        Image.fromarray(rgba, 'RGBA').save(OUT / f'{name}_base.png', optimize=True)
    else:
        Image.fromarray(rgb, 'RGB').save(OUT / f'{name}_base.png', optimize=True)
    Image.fromarray(normal_map(height, normal_strength), 'RGB').save(OUT / f'{name}_normal.png', optimize=True)
    orm = np.dstack([np.clip(c, 0, 1) for c in (ao, rough, np.broadcast_to(metal, ao.shape))])
    Image.fromarray((orm * 255 + 0.5).astype(np.uint8), 'RGB').save(OUT / f'{name}_orm.png', optimize=True)
    if emissive is not None:
        e = (srgb(emissive) * 255 + 0.5).astype(np.uint8)
        Image.fromarray(e, 'RGB').save(OUT / f'{name}_emissive.png', optimize=True)
    print('wrote', name, rgb.shape)


def tint(base_rgb, variation):
    """Base colour (sRGB 0..255) multiplied by a scalar field around 1."""
    return lin(base_rgb)[None, None, :] * variation[:, :, None]


# ---------------------------------------------------------------- concrete
def concrete():
    """Dark board-formed concrete. Tile = 2 m x 2 m: four 1 m form panels with tie holes."""
    n = 1024
    big = spectral(n, 11, beta=3.2, fmin=1, fmax=10)
    mid = spectral(n, 12, beta=2.0, fmin=4, fmax=90)
    fine = spectral(n, 13, beta=0.6, fmin=60)
    # Vertical water stains run down from tie holes and panel joints.
    streak = blur(spectral(n, 14, beta=1.0, fmin=6, fmax=200), 1.5, 40)
    f1, f2, cid = voronoi(n, 90, 15)
    pore_mask = ((cid % 7) == 0) & (f1 < 0.20)
    pores = blur(pore_mask.astype(float), 0.8)
    y, x = np.mgrid[0:n, 0:n]
    # Panel joints every 512 px (1 m) with a slightly proud lip.
    jx = np.minimum(x % 512, 512 - x % 512); jy = np.minimum(y % 512, 512 - y % 512)
    joint = np.exp(-(np.minimum(jx, jy) / 2.2) ** 2)
    lip = np.exp(-((np.minimum(jx, jy) - 5) / 3.0) ** 2) * 0.25
    ties = np.zeros((n, n))
    stain = np.zeros((n, n))
    for cx in (128, 384, 640, 896):
        for cy in (128, 384, 640, 896):
            dx = np.minimum(np.abs(x - cx), n - np.abs(x - cx)); dy = (y - cy) % n
            dyc = np.minimum(dy, n - dy)
            r = np.sqrt(dx ** 2 + dyc ** 2)
            ties += np.clip(1 - r / 11, 0, 1) ** 0.6
            stain += np.exp(-(dx / 10.0) ** 2) * np.exp(-dy / 140.0) * (dy > 0)
    height = 0.55 * fine + 0.9 * mid - 3.0 * pores - 2.5 * joint + 1.2 * lip - 4.0 * ties
    variation = 1.0 + 0.16 * big + 0.07 * mid + 0.05 * fine - 0.12 * stain - 0.06 * np.maximum(streak, 0)
    variation -= 0.35 * pores + 0.30 * joint + 0.45 * ties
    albedo = tint([66, 67, 70], np.clip(variation, 0.35, 1.5))
    rough = 0.86 + 0.05 * mid + 0.06 * pores - 0.05 * stain
    ao = cavity_ao(height, 3, 0.10) * (1 - 0.35 * ties) * (1 - 0.2 * joint)
    save_set('concrete', albedo, height, 1.6, rough, 0.0, ao)


# ---------------------------------------------------------------- polished, used floor
def floor():
    """Dark polished concrete floor, 1 m saw-cut joints, swirl scratches and traffic wear. Tile = 2 m."""
    n = 1024
    rng = np.random.default_rng(21)
    cloud = spectral(n, 22, beta=3.0, fmin=1, fmax=12)
    mottle = spectral(n, 23, beta=1.6, fmin=8, fmax=160)
    f1, _, cid = voronoi(n, 160, 24)
    chips = ((cid % 5) == 0) & (f1 < 0.28)
    chips_soft = blur(chips.astype(float), 0.7)
    y, x = np.mgrid[0:n, 0:n]
    jx = np.minimum(x % 512, 512 - x % 512); jy = np.minimum(y % 512, 512 - y % 512)
    joint = np.exp(-(np.minimum(jx, jy) / 1.6) ** 2)
    segs = []
    for _ in range(900):
        x0, y0 = rng.uniform(0, n, 2)
        ang = rng.uniform(0, np.pi) if rng.random() < 0.6 else rng.normal(0.35, 0.25)
        ln = rng.exponential(45) + 8
        segs.append((x0, y0, x0 + np.cos(ang) * ln, y0 + np.sin(ang) * ln, rng.uniform(0.25, 1.0)))
    scratches = wrapped_lines(n, segs, 1.0)
    # Curved polishing swirls.
    swirl = []
    for _ in range(60):
        cx, cy = rng.uniform(0, n, 2); r = rng.uniform(40, 160); a0 = rng.uniform(0, 2 * np.pi)
        pts = [(cx + r * np.cos(a0 + t), cy + r * np.sin(a0 + t)) for t in np.linspace(0, rng.uniform(0.4, 1.4), 18)]
        for p, q in zip(pts, pts[1:]):
            swirl.append((p[0], p[1], q[0], q[1], 0.35))
    swirls = wrapped_lines(n, swirl, 1.0)
    wear = np.clip(0.5 + 0.5 * spectral(n, 25, beta=3.4, fmin=1, fmax=8), 0, 1)
    scuffs = np.clip(spectral(n, 26, beta=1.2, fmin=20, fmax=200) - 1.6, 0, None)
    height = 0.25 * mottle - 2.5 * joint - 0.7 * scratches - 0.3 * swirls + 0.3 * chips_soft
    variation = 1.0 + 0.10 * cloud + 0.05 * mottle + 0.55 * chips_soft - 0.35 * joint - 0.25 * scuffs + 0.08 * scratches
    albedo = tint([46, 48, 53], np.clip(variation, 0.4, 2.0))
    rough = 0.16 + 0.16 * wear + 0.30 * scratches + 0.12 * swirls + 0.25 * joint + 0.15 * scuffs + 0.02 * mottle
    ao = cavity_ao(height, 2, 0.12) * (1 - 0.3 * joint)
    save_set('floor', albedo, height, 1.2, rough, 0.0, ao)


# ---------------------------------------------------------------- brushed steel
def steel():
    """Brushed stainless steel; brushing runs along U. Tile = 1 m."""
    n = 512
    rng = np.random.default_rng(31)
    brushed = blur(rng.standard_normal((n, n)), 60, 0.6)
    brushed = (brushed - brushed.mean()) / brushed.std()
    brushed2 = blur(rng.standard_normal((n, n)), 14, 0.4); brushed2 /= brushed2.std()
    smudge = spectral(n, 32, beta=3.0, fmin=1, fmax=10)
    segs = []
    for _ in range(140):
        x0, y0 = rng.uniform(0, n, 2); ang = rng.uniform(0, np.pi); ln = rng.exponential(25) + 4
        segs.append((x0, y0, x0 + np.cos(ang) * ln, y0 + np.sin(ang) * ln, rng.uniform(0.3, 1)))
    scratches = wrapped_lines(n, segs, 1.0)
    height = 0.35 * brushed + 0.25 * brushed2 - 0.6 * scratches
    variation = 1.0 + 0.05 * brushed + 0.04 * brushed2 + 0.05 * smudge
    albedo = tint([176, 180, 186], variation)
    rough = 0.30 + 0.04 * brushed2 + 0.10 * np.clip(smudge, 0, None) + 0.15 * scratches
    ao = np.ones((n, n)) * 0.98
    save_set('steel', albedo, height, 1.0, rough, 1.0, ao)


def darksteel():
    """Blackened, satin powder-coated structural steel with mill-scale mottling. Tile = 1 m."""
    n = 512
    rng = np.random.default_rng(41)
    scale = spectral(n, 42, beta=2.6, fmin=2, fmax=40)
    orange = spectral(n, 43, beta=1.0, fmin=40, fmax=220)  # powder-coat orange peel
    brushed = blur(rng.standard_normal((n, n)), 30, 0.7); brushed /= brushed.std()
    segs = []
    for _ in range(90):
        x0, y0 = rng.uniform(0, n, 2); ang = rng.uniform(0, np.pi); ln = rng.exponential(18) + 3
        segs.append((x0, y0, x0 + np.cos(ang) * ln, y0 + np.sin(ang) * ln, rng.uniform(0.3, 1)))
    scratches = wrapped_lines(n, segs, 1.0)
    height = 0.35 * orange - 0.5 * scratches + 0.1 * brushed
    variation = 1.0 + 0.12 * scale + 0.03 * orange + 0.9 * scratches
    albedo = tint([52, 54, 58], np.clip(variation, 0.5, 2.5))
    rough = 0.42 + 0.06 * scale + 0.04 * orange - 0.1 * scratches
    metal = np.clip(0.55 + 0.4 * scratches, 0, 1)
    ao = cavity_ao(height, 2, 0.05)
    save_set('darksteel', albedo, height, 0.9, rough, metal, ao)


# ---------------------------------------------------------------- leather
def leather():
    """Bordeaux pebble-grain upholstery leather. Tile = 0.6 m."""
    n = 512
    f1, f2, cid = voronoi(n, 72, 51)
    edge = f2 - f1
    pebble = np.clip(edge / 0.35, 0, 1) ** 0.6
    creases = np.abs(blur(spectral(n, 52, beta=2.0, fmin=3, fmax=40), 1.0))
    crease_lines = 0.45 * np.exp(-(creases / 0.10) ** 2)
    fold = spectral(n, 53, beta=3.0, fmin=1, fmax=8)
    rng = np.random.default_rng(54)
    cell_tone = rng.normal(0, 1, size=cid.max() + 1)[cid]
    height = 1.0 * pebble - 1.3 * crease_lines + 0.15 * fold
    variation = 1.0 + 0.18 * (pebble - 0.6) - 0.35 * crease_lines + 0.04 * cell_tone + 0.08 * fold
    albedo = tint([112, 20, 32], np.clip(variation, 0.4, 1.4))
    rough = 0.50 - 0.10 * pebble + 0.15 * crease_lines + 0.03 * fold
    ao = cavity_ao(height, 2.5, 0.25) * (1 - 0.25 * crease_lines)
    save_set('leather', albedo, height, 2.2, rough, 0.0, ao)


# ---------------------------------------------------------------- perforated grille (with holes)
def grille():
    """Hex-perforated steel sheet with real transparent holes (alpha mask). Tile = 0.4 m."""
    n = 512
    y, x = np.mgrid[0:n, 0:n].astype(float)
    cols, rows = 16, 18  # rows even and close to cols*2/sqrt(3) for a near-regular hex lattice
    pitch_x, pitch_y = n / cols, n / rows
    best = np.full((n, n), 1e9)
    for oy in (-1, 0, 1):
        for ox in (-1, 0, 1):
            r = np.floor(y / pitch_y) + oy
            shift = (r % 2) * 0.5
            c = np.floor(x / pitch_x - shift) + ox
            cx = (c + 0.5 + shift) * pitch_x; cy = (r + 0.5) * pitch_y
            best = np.minimum(best, np.sqrt((x - cx) ** 2 + (y - cy) ** 2))
    radius = pitch_x * 0.31
    alpha = np.clip((best - radius) / 1.2 + 0.5, 0, 1)
    bevel = np.clip((best - radius) / 4.0, 0, 1)
    brushed = blur(np.random.default_rng(61).standard_normal((n, n)), 20, 0.5); brushed /= brushed.std()
    height = 2.0 * bevel ** 0.5 + 0.08 * brushed
    variation = 0.85 + 0.15 * bevel + 0.03 * brushed
    albedo = tint([118, 122, 128], variation)
    rough = 0.38 + 0.05 * brushed
    ao = 0.65 + 0.35 * bevel
    save_set('grille', albedo, height, 2.0, rough, 1.0, ao, alpha=alpha)


def treadplate():
    """Aluminium-look diamond tread plate for stair treads and stage edges. Tile = 0.5 m."""
    n = 512
    y, x = np.mgrid[0:n, 0:n].astype(float)
    cell = n / 10
    u = (x % cell) / cell - 0.5; v = (y % cell) / cell - 0.5
    lozenge = np.zeros((n, n))
    for (cx, cy, ang) in [(-0.25, -0.25, 0.785), (0.25, 0.25, 0.785), (0.25, -0.25, -0.785), (-0.25, 0.25, -0.785)]:
        du, dv = u - cx, v - cy
        du = (du + 0.5) % 1 - 0.5; dv = (dv + 0.5) % 1 - 0.5
        a = np.cos(ang) * du + np.sin(ang) * dv; b = -np.sin(ang) * du + np.cos(ang) * dv
        d = (a / 0.17) ** 2 + (b / 0.045) ** 2
        lozenge = np.maximum(lozenge, np.clip(1 - d, 0, 1) ** 0.5)
    wear = np.clip(0.5 + 0.5 * spectral(n, 71, beta=2.8, fmin=1, fmax=10), 0, 1)
    brushed = blur(np.random.default_rng(72).standard_normal((n, n)), 25, 0.6); brushed /= brushed.std()
    height = 2.2 * lozenge + 0.05 * brushed
    variation = 0.9 + 0.12 * lozenge * wear + 0.04 * brushed - 0.08 * (1 - wear)
    albedo = tint([150, 154, 160], variation)
    rough = 0.42 - 0.18 * lozenge * wear + 0.05 * brushed
    ao = cavity_ao(height, 3, 0.18)
    save_set('treadplate', albedo, height, 1.6, rough, 1.0, ao)


# ---------------------------------------------------------------- backlight diffuser (emission)
def backlight():
    """Frosted acrylic light panel with a subtle hex diffuser structure. Tile = 1 m.

    The emissive map is greyscale; the colour is chosen in the material, so the
    same texture serves cyan, magenta and warm red panels.
    """
    n = 512
    f1, f2, _ = voronoi(n, 24, 81)
    glow = 0.70 + 0.30 * np.clip((f2 - f1) / 0.5, 0, 1)
    hot = np.exp(-(f1 / 0.35) ** 2) * 0.25
    haze = 0.08 * spectral(n, 82, beta=2.5, fmin=1, fmax=12)
    e = np.clip(glow + hot + haze, 0, 1.2) / 1.2
    height = 0.6 * np.clip((f2 - f1) / 0.5, 0, 1)
    albedo = tint([200, 200, 205], np.ones((n, n)))
    ao = np.ones((n, n))
    save_set('backlight', albedo, height, 0.6, np.full((n, n), 0.25), 0.0, ao,
             emissive=np.dstack([e, e, e]))


if __name__ == '__main__':
    concrete(); floor(); steel(); darksteel(); leather(); grille(); treadplate(); backlight()
