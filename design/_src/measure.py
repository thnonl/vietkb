# Layout audit (v4): injects a script into a copy of each mock, runs headless Edge --dump-dom, and reports
# text overflow (any nowrap/overflow-hidden element whose content is wider than its box), elements that
# leave the window, interactive control count, row heights and the vertical stack.
import html, json, pathlib, re, subprocess, sys
ROOT = pathlib.Path(__file__).resolve().parent.parent
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
SIZES = {'quick': (480, 384), 'hotkeys': (520, 460), 'advanced': (560, 496), 'tray': (440, 360)}
JS = r"""
<script>
const r=e=>e.getBoundingClientRect(),R=v=>Math.round(v*10)/10,W=document.documentElement.offsetWidth,H=document.documentElement.offsetHeight,out={};
const all=[...document.body.querySelectorAll('*')];
out.overflow=all.filter(e=>{const s=getComputedStyle(e);return (s.whiteSpace==='nowrap'||s.overflow==='hidden')&&e.scrollWidth>e.clientWidth+1&&e.clientWidth>0})
  .map(e=>e.className+': '+e.textContent.trim().slice(0,50));
out.outside=all.filter(e=>{const b=r(e);return b.width&&(b.right>W+0.5||b.bottom>H+0.5||b.left<-0.5)}).map(e=>e.className+' '+R(r(e).right)+','+R(r(e).bottom));
out.controls=document.querySelectorAll('.sheet .tg,.sheet .sel,.sheet .btn,.sheet .lk,.sheet .cb,.sheet .seg,.sheet .well,.sheet .numf').length;
out.stack=[...document.querySelectorAll('.cap-bar,.sheet>*')].map(e=>(e.className||e.tagName).split(' ')[0]+' '+R(r(e).top)+'-'+R(r(e).bottom));
out.ctlRight=[...new Set([...document.querySelectorAll('.r .c')].map(e=>R(r(e.lastElementChild).right)))];
const CTL='.tg,.sel,.btn,.seg,.numf,.well,.cb,.lk';
out.rows=[...document.querySelectorAll('.r,.sr,.ck,.foot,.intro,.mi')].map(row=>{
  const rb=r(row),c=[...row.querySelectorAll(CTL)].filter(e=>!e.parentElement.closest(CTL)).map(e=>{const b=r(e);return {k:e.className.split(' ')[0],t:R(b.top),b:R(b.bottom),m:R((b.top+b.bottom)/2)}});
  const labs=[...row.querySelectorAll(':scope label,:scope .l b,:scope .ttl')].map(e=>{const b=r(e);return R((b.top+b.bottom)/2)});
  const boxH=[...new Set(c.filter(x=>['sel','btn','seg','numf','well'].includes(x.k)).map(x=>R(x.b-x.t)))];
  const mids=[...c.map(x=>x.m),...labs];const spread=mids.length?R(Math.max(...mids)-Math.min(...mids)):0;
  return {row:row.className.split(' ')[0]+'@'+R(rb.top),ctl:c.map(x=>x.k+' '+x.t+'-'+x.b).join(' | '),boxH,spread};
}).filter(x=>x.ctl);
out.misaligned=out.rows.filter(x=>x.spread>0.6||x.boxH.length>1).map(x=>x.row+' spread '+x.spread+' heights '+x.boxH);
const P=document.createElement('pre');P.id='M';P.textContent=JSON.stringify(out);document.body.appendChild(P);
</script>"""
for theme in sys.argv[1:] or ['light', 'dark']:
    for fn, (w, h) in SIZES.items():
        src = (ROOT / theme / f'{fn}.html').read_text(encoding='utf-8')
        tmp = ROOT / '_src' / f'_m_{fn}.html'
        tmp.write_text(src.replace('</body>', JS + '</body>'), encoding='utf-8')
        dom = subprocess.run([EDGE, '--headless=new', '--disable-gpu', f'--window-size={w},{h}', '--virtual-time-budget=3000',
                              '--dump-dom', tmp.as_uri()], capture_output=True, text=True, encoding='utf-8').stdout
        tmp.unlink()
        m = re.search(r'<pre id="M">(.*?)</pre>', dom, re.S)
        print(f'== {theme}/{fn} ({w}x{h})')
        for kk, v in json.loads(html.unescape(m.group(1))).items():
            print(f'  {kk}: {v}')
