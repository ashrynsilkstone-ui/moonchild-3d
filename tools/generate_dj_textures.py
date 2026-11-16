#!/usr/bin/env python3
"""Original Moonchild media-player, mixer and jog-wheel PBR artwork (MIT).

No manufacturer artwork or third-party image is included. Labels use the locally
installed free DejaVu Sans font; the font file is not distributed with the game.
"""
from pathlib import Path
import math
import numpy as np
from PIL import Image, ImageDraw, ImageFont

OUT = Path(__file__).resolve().parents[1] / 'assets/textures/dj'
OUT.mkdir(parents=True, exist_ok=True)
FONT = next((p for p in [Path('/usr/share/fonts/TTF/DejaVuSans.ttf'),
                         Path('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf')] if p.is_file()), None)
if FONT is None:
    raise SystemExit('DejaVu Sans fehlt: installiere das DejaVu-Schriftpaket.')
WHITE = (180, 188, 194)
CYAN = (42, 175, 193)


def font(size):
    return ImageFont.truetype(str(FONT), size)


def label(d, xy, text, size=15, color=WHITE, anchor='mm'):
    d.text(xy, text, fill=color, font=font(size), anchor=anchor)


def surface(name, w, h, draw_labels):
    y, x = np.mgrid[0:h, 0:w].astype(np.float32)
    # Fine directional machining and restrained wear, not baked directional light.
    grain = np.sin(y * 2.3) * .65 + np.sin(y * .31 + np.sin(x * .011)) * .4
    variation = np.sin(x * .018) * np.sin(y * .025)
    value = 24 + grain + variation * 1.0
    a = np.stack([value, value + 1, value + 3], axis=-1)
    im = Image.fromarray(np.uint8(np.clip(a, 0, 255)))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((8, 8, w-9, h-9), radius=16, outline=(52, 55, 59), width=2)
    em = Image.new('RGB', (w, h), (0, 0, 0))
    draw_labels(d, ImageDraw.Draw(em), w, h)
    im.save(OUT / f'{name}_base.png', optimize=True)
    em.save(OUT / f'{name}_emissive.png', optimize=True)
    # OpenGL tangent normal. Keep machining shallow enough for small hardware.
    ny = np.sin(y * 2.3) * .018
    nz = np.sqrt(1 - ny * ny)
    normal = np.stack([np.full_like(x, .5), .5 + ny * .5, .5 + nz * .5], axis=-1)
    Image.fromarray(np.uint8(normal * 255)).save(OUT / f'{name}_normal.png', optimize=True)
    rough = np.uint8(np.clip(119 + variation * 4 + grain * 2, 110, 132))
    orm = np.stack([np.full_like(rough, 255), rough, np.full_like(rough, 185)], axis=-1)
    Image.fromarray(orm).save(OUT / f'{name}_orm.png', optimize=True)


def deck(d, e, w, h):
    def px(u, v): return (int((u + .165) / .33 * w), int((v + .225) / .45 * h))
    label(d, (w*.36, 35), 'MOONCHILD  /  MC-02', 20)
    # Outline behind the raised, angled display; the screen is a separate mesh.
    d.rounded_rectangle((60, 70, 620, 300), radius=12, fill=(10, 12, 15), outline=(65, 70, 75), width=3)
    for i in range(4):
        x, y = px(-.12 + i*.06, -.075)
        d.rounded_rectangle((x-44, y-20, x+44, y+20),radius=5,fill=(13,15,18),outline=(75,79,82),width=2)
        label(d, (x,y-35), ['HOT A','HOT B','HOT C','HOT D'][i], 13)
    x,y = px(-.025,.04)
    r=int(.092/.33*w)
    d.ellipse((x-r,y-r,x+r,y+r),outline=(71,77,80),width=3)
    label(d, px(-.025,.145), 'JOG  /  VINYL', 15)
    for v,text in [(.10,'CUE'),(.17,'PLAY / PAUSE')]:
        x,y=px(-.138,v)
        label(d,(x,y-45),text,13)
    x,y=px(.130,.055)
    d.rounded_rectangle((x-9,y-220,x+9,y+220),radius=5,fill=(4,5,7),outline=(88,94,99),width=2)
    for k in range(11):
        yy=y-205+k*41
        d.line((x-35,yy,x-17,yy),fill=(126,131,136),width=2)
    label(d,(x, y+257),'TEMPO',14)
    label(d,px(.125,-.190),'BROWSE',13)
    label(d,px(.125,-.111),'LOAD',13)
    label(d,(w/2,h-24),'USB MEDIA PLAYER   •   MOONCHILD ORIGINAL',12,(120,129,137))


def mixer(d, e, w, h):
    def px(u,v):return (int((u+.215)/.43*w),int((v+.225)/.45*h))
    label(d,(w/2,33),'MOONCHILD  /  MX-04',23)
    for i,x in enumerate([-.145,-.075,-.005,.065]):
        label(d,px(x,-.192),f'CH {i+1}',16)
        for z,text in [(-.155,'TRIM'),(-.095,'HI'),(-.035,'MID'),(.025,'LOW / FILTER')]:
            xx,yy=px(x,z)
            d.ellipse((xx-26,yy-26,xx+26,yy+26),outline=(91,97,103),width=2)
            label(d,(xx,yy+43),text,12)
        xx,yy=px(x,.137)
        d.rounded_rectangle((xx-7,yy-100,xx+7,yy+100),radius=3,fill=(4,5,7),outline=(79,86,93),width=2)
        for j in range(9):d.line((xx+16,yy-92+j*23,xx+27,yy-92+j*23),fill=(123,131,138),width=2)
        label(d,px(x,.074),'CUE',12)
    for z,text in [(-.15,'MASTER'),(-.075,'BOOTH'),(0,'HEADPHONES')]:
        label(d,px(.16,z+.035),text,12)
    x,y=px(.15,.10)
    d.rounded_rectangle((x-90,y-60,x+90,y+60),radius=8,fill=(4,9,12),outline=(77,85,91),width=2)
    label(d,(x,y-18),'FX  /  ECHO',14,CYAN)
    label(e,(x,y-18),'FX  /  ECHO',14,CYAN)
    label(d,(x,y+18),'1 / 2',18,(172,204,209))
    label(e,(x,y+18),'1 / 2',18,(172,204,209))
    for col in range(2):
        for j in range(13):
            x,y=px(.13+col*.02,-.18+j*.012)
            color=(39,163,112) if j<8 else ((189,147,49) if j<11 else (191,55,59))
            d.rectangle((x-7,y-4,x+7,y+4),fill=color)
            e.rectangle((x-7,y-4,x+7,y+4),fill=color)
    x,y=px(-.025,.198)
    d.rounded_rectangle((x-260,y-7,x+260,y+7),radius=3,fill=(5,6,8),outline=(91,97,102),width=2)
    label(d,(x,y-25),'CROSSFADER',13)


def jog():
    n=768
    y,x=np.mgrid[0:n,0:n].astype(np.float32)
    r=np.sqrt((x-n/2)**2+(y-n/2)**2)/(n/2)
    rings=np.sin(r*550)*1.2
    v=42+rings
    im=Image.fromarray(np.uint8(np.clip(np.stack([v,v+2,v+4],axis=-1),0,255)))
    d=ImageDraw.Draw(im)
    for a in range(0,360,6):
        t=math.radians(a)
        p=[(n/2+math.cos(t)*n*s,n/2+math.sin(t)*n*s) for s in [.439,.454]]
        d.line(p,fill=(178,184,187),width=2)
    d.ellipse((n*.31,n*.31,n*.69,n*.69),fill=(10,16,20),outline=(63,111,118),width=3)
    label(d,(n/2,n/2-10),'MC',56,(182,202,209))
    label(d,(n/2,n/2+42),'JOG CONTROL',15,(101,142,150))
    im.save(OUT/'jog_base.png',optimize=True)
    normal=Image.new('RGB',(n,n),(128,128,255));normal.save(OUT/'jog_normal.png',optimize=True)
    orm=np.zeros((n,n,3),dtype=np.uint8);orm[:,:,0]=255;orm[:,:,1]=np.uint8(np.clip(116+np.sin(r*550)*3,105,126));orm[:,:,2]=220
    Image.fromarray(orm).save(OUT/'jog_orm.png',optimize=True)


def display():
    w,h=1024,400
    im=Image.new('RGB',(w,h),(3,10,15));d=ImageDraw.Draw(im)
    label(d,(28,27),'MOONCHILD  /  TRACK 01',30,(149,205,215),anchor='lt')
    label(d,(30,79),'03:24.08',52,(211,226,230),anchor='lt')
    label(d,(w-30,42),'128.0',50,(177,216,225),anchor='rt')
    label(d,(w-30,104),'BPM',24,(80,142,156),anchor='rt')
    d.line((25,174,w-25,174),fill=(31,67,79),width=2)
    for x in range(26,w-26,3):
        hh=10+int(63*abs(math.sin(x*.041)*math.sin(x*.011)))
        d.line((x,255-hh,x,255+hh),fill=(37,139+hh//2,169+hh//3),width=2)
    d.line((w//2,176,w//2,336),fill=(215,102,85),width=3)
    label(d,(30,360),'HOT CUE A        00:48      /      06:12',22,(108,160,170),anchor='lt')
    im.save(OUT/'display_base.png',optimize=True)

surface('deck',768,1024,deck)
surface('mixer',1024,1024,mixer)
jog()
display()
