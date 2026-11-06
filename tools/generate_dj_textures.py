#!/usr/bin/env python3
"""Draw original Moonchild DJ equipment faceplates; Pillow is needed only to regenerate."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import math, random
P=Path(__file__).resolve().parents[1]/'assets/textures/dj'; P.mkdir(parents=True,exist_ok=True)
rng=random.Random(84177)
try:
    ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 16)
    def font(n): return ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', n)
except OSError:
    def font(n): return ImageFont.load_default()
# DejaVu is used only by the generator; the resulting painted labels are original content.
DARK=(16,20,26); LINE=(82,91,103); WHITE=(199,209,215); CYAN=(45,201,217); BLUE=(60,144,228); RED=(223,71,112)

def panel(w,h):
    im=Image.new('RGB',(w,h),DARK); d=ImageDraw.Draw(im)
    for y in range(h):
        g=int(22+8*y/h+rng.randrange(-2,3)); d.line((0,y,w,y),fill=(g,g+4,g+9))
    d.rounded_rectangle((12,12,w-12,h-12),radius=27,outline=(104,114,124),width=4)
    d.rounded_rectangle((24,24,w-24,h-24),radius=20,outline=(5,7,10),width=6)
    for x in (45,w-45):
        for y in (45,h-45):
            d.ellipse((x-9,y-9,x+9,y+9),fill=(104,109,111),outline=(2,2,3),width=3);d.line((x-4,y-4,x+4,y+4),fill=(22,23,25),width=2)
    return im,d

def label(d,x,y,text,size=18,fill=WHITE): d.text((x,y),text,font=font(size),fill=fill)
def button(d,x,y,w,h,text='',color=None):
    d.rounded_rectangle((x,y,x+w,y+h),radius=7,fill=(5,7,10),outline=(94,103,108),width=3)
    d.rounded_rectangle((x+5,y+5,x+w-5,y+h-8),radius=4,fill=(46,54,60),outline=(22,25,28),width=2)
    if color: d.line((x+8,y+h-5,x+w-8,y+h-5),fill=color,width=4)
    if text: label(d,x+8,y+8,text,15,(215,225,229))
def knob(d,x,y,r,labeltext=''):
    d.ellipse((x-r-3,y-r-3,x+r+3,y+r+3),fill=(3,4,7),outline=(87,95,100),width=3)
    d.ellipse((x-r+4,y-r+4,x+r-4,y+r-4),fill=(53,59,64),outline=(124,130,134),width=2)
    d.line((x,y-r+8,x,y-r//4),fill=(224,229,228),width=4)
    if labeltext: label(d,x-r,y+r+6,labeltext,13,(168,177,182))

def deck():
    w,h=1536,900; im,d=panel(w,h)
    label(d,78,52,'MOONCHILD',28,(214,222,226)); label(d,79,88,'PRO PLAYER  /  MC-01',14,(142,156,166))
    label(d,1060,64,'DIGITAL MEDIA PLAYER',16,(147,161,166))
    d.rounded_rectangle((125,115,1035,290),radius=13,fill=(4,8,13),outline=(72,83,89),width=5)
    d.rectangle((145,135,1015,270),fill=(6,20,29))
    label(d,160,143,'MOONCHILD     TRACK 01',19,(124,213,222));label(d,160,173,'03:24.08',28,(219,235,239));label(d,736,148,'128.0 BPM',20,(193,223,233))
    for x in range(160,995,4):
        hh=5+int(30*abs(math.sin(x*.032)*math.sin(x*.009)))+rng.randrange(0,5)
        d.line((x,239-hh,x,239+hh),fill=(31,153+int(hh),180+int(hh/2)),width=2)
    cx,cy,r=510,550,230
    for rad,col in [(r+18,(4,5,7)),(r+8,(91,98,103)),(r,(12,15,19)),(r-12,(90,96,99)),(r-19,(19,23,28)),(r-32,(44,49,54)),(r-42,(21,25,30)),(r-68,(40,45,49)),(r-72,(11,14,17))]:
        d.ellipse((cx-rad,cy-rad,cx+rad,cy+rad),fill=col)
    for a in range(0,360,5):
        t=math.radians(a);r0=r-29;r1=r-20
        d.line((cx+math.cos(t)*r0,cy+math.sin(t)*r0,cx+math.cos(t)*r1,cy+math.sin(t)*r1),fill=(161,166,168),width=3)
    d.ellipse((cx-110,cy-110,cx+110,cy+110),fill=(10,20,25),outline=(75,145,153),width=4)
    label(d,cx-68,cy-32,'MC',45,(201,220,222));label(d,cx-76,cy+28,'JOG WHEEL',13,(137,169,173))
    for i,(name,col) in enumerate([('CUE',CYAN),('PLAY',CYAN),('SYNC',BLUE),('SHIFT',None)]): button(d,86+i*112,797,100,60,name,col)
    for i in range(4): button(d,865+(i%2)*105,395+(i//2)*80,92,60,str(i+1),RED if i==0 else BLUE)
    label(d,865,360,'HOT CUES',16)
    for i,t in enumerate(['LOOP IN','OUT','RELOOP','BEAT']): button(d,859+i*105,598,92,53,t,CYAN if i==0 else None)
    knob(d,908,735,34,'BROWSE');knob(d,1015,735,28,'SELECT')
    label(d,1212,315,'TEMPO',17);d.rounded_rectangle((1275,345,1302,742),radius=8,fill=(4,5,6),outline=LINE,width=2)
    for y in range(365,730,30): d.line((1235,y,1262 if (y-365)%60 else 1270,y),fill=(149,163,167),width=2)
    d.rounded_rectangle((1248,514,1332,555),radius=5,fill=(104,112,115),outline=(196,202,203),width=3)
    label(d,1207,764,'PITCH RANGE',13);button(d,1200,790,126,48,'MASTER',BLUE)
    im.save(P/'deck_base.png',optimize=True)
    # Only lit displays and colored indicators emit; the metal faceplate stays dark.
    mask=Image.new('RGB',(w,h),(0,0,0)); mp=mask.load(); pix=im.load()
    for yy in range(h):
        for xx in range(w):
            rr,gg,bb=pix[xx,yy]
            screen=145<=xx<=1015 and 135<=yy<=270
            colored=(max(rr,gg,bb)>100 and max(rr,gg,bb)-min(rr,gg,bb)>45)
            if screen or colored: mp[xx,yy]=(rr,gg,bb)
    mask.save(P/'deck_emissive.png',optimize=True)

def mixer():
    w,h=1200,860;im,d=panel(w,h)
    label(d,60,45,'MOONCHILD  /  CLUB MIXER',25);label(d,750,51,'FOUR CHANNEL',15,(153,166,172))
    for ch,x in enumerate([230,410,590,770]):
        label(d,x-15,98,f'CH {ch+1}',17)
        for j,name in enumerate(['TRIM','HI','MID','LOW','FILTER']): knob(d,x,170+j*88,25,name)
        d.rounded_rectangle((x-10,640,x+10,783),radius=4,fill=(4,5,7),outline=(98,105,109),width=2)
        for y in range(648,782,15): d.line((x+18,y,x+27,y),fill=(120,132,137),width=2)
        y=715-(ch%3)*20;d.rounded_rectangle((x-34,y,x+34,y+30),radius=5,fill=(126,135,139),outline=(229,230,226),width=2)
        button(d,x-36,582,72,39,'CUE',CYAN if ch==1 else None)
    for i,(t,c) in enumerate([('MASTER',CYAN),('BOOTH',BLUE),('PHONES',None),('FX',RED)]): knob(d,1015,145+i*104,30,t)
    for ch in range(2):
        for j in range(16): d.rectangle((925+ch*22,622-j*13,937+ch*22,630-j*13),fill=(32,160,112) if j<10 else ((215,166,52) if j<14 else (215,65,76)))
    d.rounded_rectangle((305,816,785,830),radius=3,fill=(3,4,5),outline=LINE,width=2)
    d.rounded_rectangle((514,802,576,844),radius=5,fill=(130,139,143),outline=(225,228,227),width=2)
    label(d,70,790,'HEADPHONES',12);label(d,930,760,'LEVEL',13)
    im.save(P/'mixer_base.png',optimize=True)
    mask=Image.new('RGB',(w,h),(0,0,0));mp=mask.load();pix=im.load()
    for yy in range(h):
        for xx in range(w):
            rr,gg,bb=pix[xx,yy]
            if max(rr,gg,bb)>100 and max(rr,gg,bb)-min(rr,gg,bb)>45: mp[xx,yy]=(rr,gg,bb)
    mask.save(P/'mixer_emissive.png',optimize=True)
deck();mixer()
