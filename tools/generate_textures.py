#!/usr/bin/env python3
"""Generate Moonchild's own repeatable PBR texture sets (CC0). Requires numpy/Pillow."""
from pathlib import Path
import numpy as np
from PIL import Image

N = 256
OUT = Path(__file__).resolve().parents[1] / 'assets/textures/moonchild'
OUT.mkdir(parents=True, exist_ok=True)
rng = np.random.default_rng(1969)
y, x = np.mgrid[0:N, 0:N] / N

def periodic_noise():
    result = np.zeros((N, N))
    for freq, amp in [(3,.32), (7,.22), (15,.14), (31,.07), (61,.035)]:
        for _ in range(5):
            a, b = rng.integers(-freq, freq+1, size=2)
            result += amp * np.cos(2*np.pi*(a*x+b*y)+rng.uniform(0, 2*np.pi))/5
    return result

def save(name, color, h, rough, metallic):
    base = np.clip(np.array(color)[None,None,:] * (1 + h[:,:,None]*.35), 0, 255).astype('uint8')
    dx = (np.roll(h,-1,1)-np.roll(h,1,1))*2.5
    dy = (np.roll(h,-1,0)-np.roll(h,1,0))*2.5
    normal = np.stack((-dx, dy, np.ones_like(dx)), axis=-1)
    normal /= np.linalg.norm(normal, axis=-1, keepdims=True)
    normal = np.clip((normal*.5+.5)*255, 0,255).astype('uint8')
    rm = np.empty((N,N,3), dtype='uint8')
    rm[:,:,0] = 255
    rm[:,:,1] = np.clip((rough + h*.08)*255,0,255)
    rm[:,:,2] = int(metallic*255)
    for suffix, data in [('base',base),('normal',normal),('orm',rm)]:
        Image.fromarray(data).save(OUT / f'{name}_{suffix}.png', optimize=True)

noise = periodic_noise()
save('concrete', [67,70,75], noise, .86, 0)
noise = periodic_noise()*.4 + .16*np.sin(2*np.pi*(y*93 + .13*np.sin(2*np.pi*x*3)))
save('steel', [130,139,147], noise, .36, 1)
noise = periodic_noise()*.45 + .035*np.sin(2*np.pi*(x*55+y*27))
save('floor', [75,79,84], noise, .27, .08)
noise = periodic_noise()*.6 + .04*np.sin(2*np.pi*x*43)*np.sin(2*np.pi*y*39)
save('leather', [116,24,37], noise, .44, 0)
# Embossed woven metal, fully tileable; no alpha/transparent holes are claimed.
noise = .30*(np.cos(2*np.pi*x*16)**8 + np.cos(2*np.pi*y*16)**8) + periodic_noise()*.12
save('grille', [91,99,105], noise, .48, .9)
