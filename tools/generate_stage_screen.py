#!/usr/bin/env python3
"""Generate Moonchild's own high-resolution stage-screen artwork (MIT)."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import numpy as np
W,H=2048,1024
out=Path(__file__).resolve().parent.parent/'assets/textures/moonchild/stage_screen.png'
out.parent.mkdir(parents=True, exist_ok=True)
y,x=np.mgrid[0:H,0:W].astype(np.float32)
r=np.sqrt(((x-W/2)/(W/2))**2+((y-H/2)/(H/2))**2)
wave=(np.sin(x/120+y/210)+1)*0.5
rgb=np.stack([8+8*(1-r.clip(0,1))+4*wave, 9+7*(1-r.clip(0,1))+2*wave, 18+15*(1-r.clip(0,1))+7*wave],axis=-1)
bg=Image.fromarray(np.uint8(np.clip(rgb,0,255)),'RGB').convert('RGBA')
glow=Image.new('RGBA',(W,H)); d=ImageDraw.Draw(glow)
for i in range(5):
    rr=240+i*82
    box=(W//2-rr,H//2-rr,W//2+rr,H//2+rr)
    d.arc(box,195,345,fill=(16,155,210,110),width=9)
    d.arc(box,15,165,fill=(200,23,104,100),width=9)
for i in range(20):
    px=70+i*100; size=18+(i*47)%105
    d.rounded_rectangle((px,H-110-size,px+10,H-110),radius=5,fill=(70,160,200,90))
bg=Image.alpha_composite(bg,glow.filter(ImageFilter.GaussianBlur(24)))
bg=Image.alpha_composite(bg,glow)
d=ImageDraw.Draw(bg)
font_path=next((p for p in [Path('/usr/share/fonts/TTF/DejaVuSans.ttf'),Path('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf')] if p.is_file()),None)
if font_path is None: raise SystemExit('DejaVu Sans fehlt: installiere das DejaVu-Schriftpaket.')
font=ImageFont.truetype(str(font_path),148)
small=ImageFont.truetype(str(font_path),30)
d.text((W//2,H//2-5),'M O O N C H I L D',font=font,anchor='mm',fill=(218,222,224,255),stroke_width=0)
d.text((W//2,H//2+112),'N I G H T   •   M U S I C   •   L I G H T',font=small,anchor='mm',fill=(147,182,191,255))
bg.convert('RGB').save(out,optimize=True)
