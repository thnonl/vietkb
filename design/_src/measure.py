# Layout audit: injects a measuring script into a copy of each screen, runs headless Edge (--dump-dom),
# prints overflowing text, control right edges, toggle x, row label-block centring and vertical extents.
import html, json, pathlib, re, subprocess, sys
ROOT = pathlib.Path(__file__).resolve().parent.parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
JS = r"""
<script>
const r=e=>e.getBoundingClientRect(), R=v=>Math.round(v*10)/10, out={};
out.overflow=[...document.querySelectorAll('.lbl>span,.lbl>b,.m,.msg,.ex em,.out,.md,.tab,.btn,.pre,.intro,.mfoot span,.st,.xr b,.hang h3,.sel')]
  .filter(e=>e.scrollWidth>e.clientWidth+1).map(e=>e.className+': '+e.textContent.trim().slice(0,60));
out.ctlRight=[...new Set([...document.querySelectorAll('.row .ctl,.status .f,.tb,.tbl,.mfoot,.hkfoot,.xr')].map(e=>{const c=e.lastElementChild||e;return R(r(c).right)}))];
out.toggleX=[...new Set([...document.querySelectorAll('.row .tg')].map(e=>R(r(e).left)+'-'+R(r(e).right)))];
out.centre=[...document.querySelectorAll('.row:not(.tall)')].map(e=>{const a=r(e),b=r(e.querySelector('.lbl'));return R((b.top-a.top)-(a.bottom-b.bottom))}).filter(v=>Math.abs(v)>0.6);
out.tabs=[...document.querySelectorAll('.tab')].map(e=>R(r(e).left));
out.blocks=[...document.querySelectorAll('.status,.list,.disc,.try,.sh,.presets,.hkfoot,.intro,.tb,.tbl,.mfoot,.hang,.bulk')].map(e=>e.className.split(' ')[0]+' '+R(r(e).top)+'-'+R(r(e).bottom));
out.last=R(Math.max(...[...document.querySelectorAll('.sheet *')].map(e=>r(e).bottom)));
const P=document.createElement('pre');P.id='M';P.textContent=JSON.stringify(out);document.body.appendChild(P);
</script>"""
for theme in sys.argv[1:] or ['light']:
    for fn in ['go', 'macro', 'hotkey', 'system']:
        src = (ROOT / theme / f'{fn}.html').read_text(encoding='utf-8')
        tmp = ROOT / '_src' / f'_m_{fn}.html'
        tmp.write_text(src.replace('</body>', JS + '</body>'), encoding='utf-8')
        dom = subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--window-size=720,560', '--virtual-time-budget=3000',
                              '--dump-dom', tmp.as_uri()], capture_output=True, text=True, encoding='utf-8').stdout
        tmp.unlink()
        m = re.search(r'<pre id="M">(.*?)</pre>', dom, re.S)
        print(f'== {theme}/{fn}')
        for kk, v in json.loads(html.unescape(m.group(1))).items():
            print(f'  {kk}: {v}')
