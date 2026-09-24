from PIL import Image, ImageDraw, ImageFilter
from pathlib import Path
S=1024
root=Path(r'E:\Browser\src-tauri\icons')
# rounded-square mask
rr=Image.new('L',(S,S),0); d=ImageDraw.Draw(rr); d.rounded_rectangle((38,38,S-38,S-38),radius=220,fill=255)
# background vertical/radial-ish gradient
bg=Image.new('RGBA',(S,S),(0,0,0,0)); p=bg.load()
for y in range(S):
    for x in range(S):
        dx=(x-S*.48)/(S*.72); dy=(y-S*.40)/(S*.78); r=min(1,(dx*dx+dy*dy)**.5)
        t=1-r
        p[x,y]=(int(7+14*t),int(10+23*t),int(17+39*t),255)
bg.putalpha(rr)
# glows
orb=Image.new('RGBA',(S,S),(0,0,0,0)); od=ImageDraw.Draw(orb)
od.ellipse((120,80,900,860),fill=(72,105,255,34)); orb=orb.filter(ImageFilter.GaussianBlur(110)); bg=Image.alpha_composite(bg,orb)
# orbit glow + line
ol=Image.new('RGBA',(S,S),(0,0,0,0)); od=ImageDraw.Draw(ol)
od.arc((105,170,925,825),198,355,fill=(104,239,255,80),width=18); glow=ol.filter(ImageFilter.GaussianBlur(26)); bg=Image.alpha_composite(bg,glow); bg=Image.alpha_composite(bg,ol)
# V mask
vm=Image.new('L',(S,S),0); vd=ImageDraw.Draw(vm); vd.line([(220,286),(492,780),(792,260)],fill=255,width=92,joint='curve')
# gradient fill for V
vg=Image.new('RGBA',(S,S),(0,0,0,0)); q=vg.load()
for y in range(S):
    for x in range(S):
        t=x/(S-1)
        if t<.5:
            u=t/.5; c=(int(113+(106-113)*u),int(244+(139-244)*u),int(255+(255-255)*u),255)
        else:
            u=(t-.5)/.5; c=(int(106+(155-106)*u),int(139+(99-139)*u),int(255+(255-255)*u),255)
        q[x,y]=c
vg.putalpha(vm)
# V glow
vglow=vg.copy(); vglow.putalpha(vm.filter(ImageFilter.GaussianBlur(34))); bg=Image.alpha_composite(bg,vglow); bg=Image.alpha_composite(bg,vg)
# inner highlight
hi=Image.new('RGBA',(S,S),(0,0,0,0)); hd=ImageDraw.Draw(hi); hd.line([(232,292),(493,751),(780,264)],fill=(235,253,255,42),width=15,joint='curve'); bg=Image.alpha_composite(bg,hi)
# orbital dot
sd=Image.new('RGBA',(S,S),(0,0,0,0)); dd=ImageDraw.Draw(sd); dd.ellipse((782,220,824,262),fill=(177,252,255,255)); sdg=sd.filter(ImageFilter.GaussianBlur(22)); bg=Image.alpha_composite(bg,sdg); bg=Image.alpha_composite(bg,sd)
# exports
master=bg
master.resize((512,512),Image.Resampling.LANCZOS).save(root/'icon.png')
for name,size in [('32x32.png',32),('128x128.png',128),('128x128@2x.png',256),('StoreLogo.png',50),('Square30x30Logo.png',30),('Square44x44Logo.png',44),('Square71x71Logo.png',71),('Square89x89Logo.png',89),('Square107x107Logo.png',107),('Square142x142Logo.png',142),('Square150x150Logo.png',150),('Square284x284Logo.png',284),('Square310x310Logo.png',310)]:
    master.resize((size,size),Image.Resampling.LANCZOS).save(root/name)
master.resize((256,256),Image.Resampling.LANCZOS).save(root/'icon.ico',format='ICO',sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
try:
    master.save(root/'icon.icns',format='ICNS')
except Exception as e:
    print('ICNS skipped',e)
print('VEYRA_ICONS_OK', (root/'icon.ico').stat().st_size)
