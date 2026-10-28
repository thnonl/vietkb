# VietKB settings mocks, v3: quick-settings window 720 x 560, four tabs, no sidebar, no scrolling.
# Run: python design/_src/build.py  -> design/light/*.html, design/dark/*.html, design/index.html
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from kit import LIGHT, DARK, I, ic, cb, LOGO, k

ROOT = pathlib.Path(__file__).resolve().parent.parent
W, H = 720, 560

CSS = r"""
:root{
--ui:"Segoe UI Variable Text","Segoe UI Variable","Segoe UI",system-ui,sans-serif;
--disp:"Segoe UI Variable Display","Segoe UI Variable","Segoe UI",system-ui,sans-serif;
--sm:"Segoe UI Variable Small","Segoe UI Variable","Segoe UI",system-ui,sans-serif;
--serif:"Sitka Text","Sitka",Cambria,Georgia,serif;
--mono:"Cascadia Mono","Cascadia Code",Consolas,monospace;
--ease:cubic-bezier(.2,.8,.2,1);
}
*{box-sizing:border-box;margin:0;padding:0}
html{background:var(--plane)}
body{width:720px;height:560px;overflow:hidden;font:400 14px/20px var(--ui);color:var(--t1);background:var(--plane);
 font-kerning:normal;font-feature-settings:"kern" 1;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased}
b{font-weight:600}
.num{font-variant-numeric:tabular-nums}
svg.i{width:20px;height:20px;flex:none;fill:none;stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}
svg.i.s16{width:16px;height:16px;stroke-width:1.875}
@keyframes blink{0%,49%{opacity:1}50%,100%{opacity:0}}
@keyframes pulse{0%,100%{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}50%{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 6px transparent}}
@media (prefers-reduced-motion:reduce){*{animation:none!important;transition:none!important}}

/* frame: 40 strip + 520 sheet */
.win{display:flex;flex-direction:column;height:560px}
.strip{height:40px;flex:none;display:flex;align-items:center;padding-left:24px}
.mark{display:flex;margin-right:14px}
.tabs{display:flex;gap:2px}
.tab{height:28px;display:flex;align-items:center;gap:6px;padding:0 12px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.tab.on{background:var(--raised);color:var(--t1);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2)}
.tab .c{font:500 12px/18px var(--sm);color:var(--t3);font-variant-numeric:tabular-nums}
.st{margin-left:auto;width:236px;justify-content:flex-end;display:flex;align-items:center;gap:8px;height:28px;padding:0 10px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.st b{color:var(--t1)}
.dot{width:8px;height:8px;border-radius:50%;background:var(--acc);box-shadow:0 0 0 3px var(--acc-tint-2);flex:none}
.wc{display:flex;align-self:stretch;margin-left:auto}
.st+.wc{margin-left:4px}
.wc span{width:46px;display:grid;place-items:center;color:var(--t2)}
.wc svg{width:10px;height:10px;fill:none;stroke:currentColor;stroke-width:1}
.sheet{flex:1;min-height:0;background:var(--surface);border-top:1px solid var(--line-2);display:flex;flex-direction:column;padding:0 24px 16px;position:relative}

/* status row (tab Gõ) */
.status{height:40px;flex:none;display:flex;align-items:center;gap:10px;margin:0 -24px;padding:0 24px;border-bottom:1px solid var(--line-1)}
.status .lab{font:600 14px/16px var(--ui)}
.status .hint{display:flex;align-items:center;gap:2px;margin-left:2px}
.status .f{margin-left:auto;display:flex;align-items:center;gap:8px}
.status .f small{font:400 12px/16px var(--sm);color:var(--t2);margin-left:8px}

/* rows: 46 = 4 + 20 (label 14/20) + 18 (note 12/18) + 4 */
.list{display:flex;flex-direction:column;flex:none}
.row{height:46px;flex:none;display:flex;align-items:center;gap:16px;padding:0 12px;margin:0 -12px;border-radius:8px;position:relative}
.row+.row::before{content:"";position:absolute;left:12px;right:12px;top:0;height:1px;background:var(--line-1)}
.row.gb::before{background:var(--line-2)}
.row.s{height:40px}
.row.hov{background:var(--hover)}
.row.hov::before,.row.hov+.row::before{opacity:0}
.lbl{flex:1;min-width:0}
.lbl>b{display:block;font:500 14px/20px var(--ui);color:var(--t1);white-space:nowrap}
.lbl>span{display:block;font:400 12px/18px var(--sm);color:var(--t3);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.lbl .kbd{font:400 12px/18px var(--mono);color:var(--t2)}
.lbl .lnk{color:var(--acc-text);font-weight:500}
.lbl .lnk.bad{color:var(--bad)}
.lbl>b .i{vertical-align:-4px;margin-right:4px}
.row.dis .lbl>b{color:var(--t3)}
.ctl{flex:none;display:flex;align-items:center;gap:12px}
.link{color:var(--acc-text);font:500 13px/20px var(--ui);display:inline-flex;align-items:center;gap:1px;white-space:nowrap}

/* disclosure row */
.disc{height:40px;flex:none;display:flex;align-items:center;gap:8px;padding:0 64px 0 12px;margin:0 -12px;position:relative;border-top:1px solid var(--line-2)}
.disc>.i{color:var(--t2);margin-left:-2px}
.disc b{font:500 14px/20px var(--ui)}
.disc .m{font:400 12px/18px var(--sm);color:var(--t3);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;margin-left:4px}

/* thử gõ strip: a focused field, one line */
.try{height:36px;flex:none;margin-top:auto;display:flex;align-items:center;gap:12px;padding:0 12px;border-radius:8px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}
.try .lab{font:500 12px/18px var(--sm);color:var(--t3);padding-right:12px;border-right:1px solid var(--line-2);flex:none}
.try .out{font:400 16px/24px var(--serif);color:var(--t1);white-space:nowrap;flex:1;min-width:0;overflow:hidden}
.try .m{font:400 12px/18px var(--sm);color:var(--t3);flex:none}
.caret{display:inline-block;width:1.5px;height:17px;background:var(--focus);vertical-align:-3px;margin-left:1px;animation:blink 1.06s infinite}

/* controls */
.tg{width:40px;height:24px;border-radius:12px;background:var(--tg-off);box-shadow:inset 0 0 0 1px var(--ctl-line);position:relative;flex:none}
.tg::after{content:"";position:absolute;top:4px;left:4px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:var(--knob);transition:transform .16s var(--ease)}
.tg.on{background:var(--acc);box-shadow:none}
.tg.on::after{transform:translateX(20px)}
.tg.bad.on{background:var(--bad)}
.tg.dis{opacity:.4}
.cb{width:16px;height:16px;border-radius:4.5px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--ctl-line);display:inline-grid;place-items:center;flex:none;color:#fff}
.cb svg{width:12px;height:12px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
.cb.on,.cb.mix{background:var(--acc);box-shadow:none}
.btn{height:32px;display:inline-flex;align-items:center;gap:6px;padding:0 12px;border-radius:8px;font:500 13px/20px var(--ui);color:var(--t1);background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);white-space:nowrap;flex:none}
.btn .i{color:var(--t2);margin-left:-3px}
.btn.pri{background:var(--acc);color:var(--on-acc);box-shadow:var(--pri-shadow)}
.btn.pri .i{color:inherit}
.btn.ghost{background:none;box-shadow:none;color:var(--t2)}
.btn.ghost.hov{background:var(--hover);color:var(--t1)}
.btn.icon{width:32px;padding:0;justify-content:center}
.btn.icon .i{margin:0}
.btn.sm{height:28px;padding:0 10px;border-radius:7px}
.btn.icon.sm{width:28px;padding:0}
.btn.danger,.btn.danger .i{color:var(--bad)}
.sel{height:30px;display:inline-flex;align-items:center;gap:8px;padding:0 6px 0 10px;border-radius:8px;background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui);color:var(--t1);white-space:nowrap}
.sel .i{color:var(--t3);margin-left:auto}
.seg{display:inline-flex;height:32px;padding:2px;border-radius:9px;background:var(--sunken);box-shadow:inset 0 0 0 1px var(--line-1);flex:none}
.seg>span{height:28px;display:inline-flex;align-items:center;gap:6px;padding:0 11px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.seg>span.on{background:var(--raised);color:var(--t1);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2)}
.seg.tone>span{color:var(--t3)}
.seg.tone>span.on{color:var(--acc-text)}
.seg.tone i{font-style:normal;color:var(--t4)}
.seg.tone em{font:400 15px/20px var(--serif);font-style:normal;color:var(--t2)}
.seg.tone .on em{color:var(--t1)}
.k{display:inline-flex;align-items:center;justify-content:center;height:22px;min-width:24px;padding:0 7px 1px;border-radius:6px;font:600 12px/16px var(--sm);color:var(--t1);
 background:linear-gradient(var(--key-top),var(--key-bot));box-shadow:inset 0 0 0 1px var(--key-line),inset 0 -2px 0 var(--key-lip);white-space:nowrap;flex:none}
.k.sm{height:20px;min-width:20px;padding:0 5px 1px;border-radius:5px;font-size:11.5px}
.k.m{font:500 12.5px/16px var(--mono);letter-spacing:-.01em}
.plus{font:400 12px/16px var(--sm);color:var(--t4);margin:0 1px}
.tag{display:inline-flex;align-items:center;gap:4px;height:20px;padding:0 7px;border-radius:6px;font:500 12px/18px var(--sm);white-space:nowrap;flex:none}
.tag.n{background:var(--sunken);color:var(--t2)}
.tag.warn{background:var(--warn-tint);color:var(--warn);box-shadow:inset 0 0 0 1px var(--warn-line)}
.input{height:32px;display:flex;align-items:center;gap:8px;padding:0 6px 0 10px;border-radius:8px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui);color:var(--t3)}
.input .i{color:var(--t3)}
.input .kk{margin-left:auto;display:flex;align-items:center}
.tile{width:24px;height:24px;border-radius:6px;display:grid;place-items:center;background:var(--tile);box-shadow:inset 0 0 0 1px var(--line-2);font:600 11.5px/16px var(--disp);color:var(--t2);flex:none}

/* gõ tắt tab */
.tb{height:36px;flex:none;display:flex;align-items:center;gap:8px;margin-top:16px}
.tb .input{flex:1}
.tbl{margin-top:12px;flex:none;border:1px solid var(--line-2);border-radius:10px;background:var(--raised);overflow:hidden}
.tr{display:grid;grid-template-columns:40px 112px 1fr 68px;align-items:center;height:46px;padding-left:1px;position:relative}
.tr+.tr{box-shadow:inset 0 1px 0 var(--line-1)}
.tr>.c0{display:grid;place-items:center}
.tr>.c1{display:flex}
.tr .ex{font:400 14px/20px var(--ui);color:var(--t1);display:flex;gap:8px;align-items:center;min-width:0;padding-right:12px}
.tr .ex em{font-style:normal;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;min-width:0}
.tr .acts{display:flex;justify-content:flex-end;gap:2px;padding-right:8px;opacity:0;margin-top:-2px}
.tr.hov{background:var(--hover)}
.tr.hov .acts{opacity:1}
.tr.on{background:var(--acc-tint)}
.tr.on::after{content:"";position:absolute;left:0;top:0;bottom:0;width:2px;background:var(--rail)}
.tr.th{height:36px;display:grid;box-shadow:inset 0 -1px 0 var(--line-2);font:600 12px/18px var(--sm);color:var(--t2)}
.bulk{height:36px;display:flex;align-items:center;gap:2px;padding:0 6px 0 13px;background:var(--acc-tint);box-shadow:inset 0 -1px 0 var(--acc-line);font:500 13px/20px var(--ui)}
.bulk .n{display:flex;align-items:center;gap:13px;margin-right:auto;color:var(--t1);font-weight:600}
.bulk .n small{font:400 12px/18px var(--sm);color:var(--t3)}
.bulk .vsep{width:1px;height:16px;background:var(--acc-line);margin:0 4px}
.bulk .btn.ghost{color:var(--t1)}
.bulk .btn.ghost.danger{color:var(--bad)}
.mfoot{display:flex;align-items:center;justify-content:space-between;margin-top:10px;font:400 12px/18px var(--sm);color:var(--t3);flex:none}
.mfoot .link{font:500 12px/18px var(--sm)}

/* phím tắt tab */
.intro{font:400 13px/20px var(--ui);color:var(--t2);margin-top:14px;display:flex;align-items:center;gap:4px;flex:none}
.sh{font:600 13px/20px var(--ui);color:var(--t1);padding:18px 0 6px;flex:none;border-bottom:1px solid var(--line-2)}
.sh+.list>.row:first-child::before{display:none}
.cap{width:276px;height:32px;display:flex;align-items:center;gap:3px;padding:0 5px;border-radius:8px;background:var(--sunken);box-shadow:inset 0 1px 1.5px rgba(0,0,0,.06),inset 0 0 0 1px var(--line-1);flex:none}
.cap .end{margin-left:auto;display:flex;align-items:center;gap:6px;font:400 12px/18px var(--sm);color:var(--t3);padding-right:3px}
.cap .split{margin-left:auto;align-self:stretch;width:28px;display:grid;place-items:center;border-left:1px solid var(--line-2);margin-right:-5px;color:var(--t3)}
.cap.rec{background:var(--raised);animation:pulse 1.6s var(--ease) infinite;box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}
.cap.rec .caret{height:16px;margin-left:4px;vertical-align:0}
.cap.warn{box-shadow:inset 0 1px 1.5px rgba(0,0,0,.06),inset 0 0 0 1px var(--warn-line)}
.cap.empty{background:transparent;box-shadow:none;border:1px dashed var(--line-3);padding:0 10px;font:400 13px/20px var(--ui);color:var(--t3)}
.cap.empty .end{color:var(--acc-text);font:500 13px/20px var(--ui)}
.btn.icon.xs{width:24px;height:24px;padding:0;border-radius:6px;justify-content:center}
.btn.icon.xs .i{margin:0}
.cap .end .btn.icon.xs{margin-right:-1px}
.row.rec{background:var(--acc-tint)}
.row.rec::before,.row.rec+.row::before{opacity:0}
.row.tall{height:64px;align-items:flex-start;padding-top:4px}
.row.tall .cap{margin-top:3px}
.row .msg{position:absolute;left:12px;right:12px;top:42px;display:block;font:400 12px/18px var(--sm);color:var(--warn);white-space:nowrap}
.row .msg .i{vertical-align:-3px;margin-right:4px}
.row .msg .i{color:var(--warn)}
.presets{display:flex;align-items:center;gap:6px;height:34px;padding-left:0;font:400 12px/18px var(--sm);color:var(--t3);flex:none}
.pre{display:inline-flex;align-items:center;gap:2px;height:26px;padding:0 6px 0 4px;border-radius:7px;box-shadow:inset 0 0 0 1px var(--line-2);color:var(--t2)}
.pre.on{background:var(--acc-tint);box-shadow:inset 0 0 0 1px var(--acc-line)}
.pre small{font:400 12px/18px var(--sm);color:var(--t3);margin-left:4px}
.hkfoot{margin-top:auto;display:flex;align-items:center;gap:12px;padding-top:8px;border-top:1px solid var(--line-2);flex:none}
.hkfoot .m{margin-left:auto;font:400 12px/18px var(--sm);color:var(--t3);display:flex;align-items:center;gap:3px}

/* hệ thống tab: hanging section labels 120 + 16 + 536 */
.hang{display:grid;grid-template-columns:120px 1fr;column-gap:16px;flex:none}
.hang+.hang{margin-top:14px}
.hang:first-of-type{margin-top:16px}
.hang>h3{font:600 13px/20px var(--ui);color:var(--t1);padding-top:5px;border-top:1px solid var(--line-2)}
.hang>h3.s{padding-top:10px}
.hang>h3 small{display:block;font:400 12px/18px var(--sm);color:var(--t3);margin-top:2px}
.hang>h3 .link{font:500 12px/18px var(--sm);margin-top:2px}
.hang>.list{border-top:1px solid var(--line-2)}
.hang>.list>.row:first-child::before{display:none}
.xr{height:40px;display:flex;align-items:center;gap:10px;position:relative}
.xr+.xr::before{content:"";position:absolute;left:0;right:0;top:0;height:1px;background:var(--line-1)}
.xr b{font:500 13px/20px var(--ui);flex:1}
.xr .md{font:400 12px/18px var(--sm);color:var(--t3)}
.path{font:400 12px/18px var(--mono);color:var(--t2)}
"""

# ---------------------------------------------------------------- frame
TABS = [('go', 'Gõ', None), ('macro', 'Gõ tắt', '148'), ('hotkey', 'Phím tắt', None), ('system', 'Hệ thống', None)]
WC = ('<div class="wc">'
      '<span title="Thu nhỏ"><svg viewBox="0 0 10 10"><path d="M0 5.5h10"/></svg></span>'
      '<span title="Đóng, VietKB vẫn chạy"><svg viewBox="0 0 10 10"><path d="M.5.5l9 9M9.5.5l-9 9"/></svg></span></div>')
MARK = LOGO.replace('width="20" height="20"', 'width="18" height="18"')

def strip(active):
    tabs = ''.join(f'<span class="tab{" on" if key == active else ""}">{lab}'
                   f'{f"<span class=c>{c}</span>" if c else ""}</span>' for key, lab, c in TABS)
    # State is always visible: tab Gõ shows it in its status row, every other tab in the strip.
    # The chip slot has a fixed width on every tab (hidden on Gõ), so nothing in the strip moves.
    chip = '<span class="dot"></span><b>Tiếng Việt</b>· Telex · Unicode'
    st = (f'<span class="st" aria-hidden="true" style="visibility:hidden">{chip}</span>' if active == 'go'
          else f'<span class="st" title="Bấm để tắt tiếng Việt">{chip}</span>')
    return f'<header class="strip"><span class="mark">{MARK}</span><nav class="tabs">{tabs}</nav>{st}{WC}</header>'

def page(theme, title, active, body):
    tokens = LIGHT if theme == 'light' else DARK
    return f'''<!doctype html>
<html lang="vi" data-theme="{theme}">
<head><meta charset="utf-8"><title>Cài đặt VietKB</title>
<meta name="viewport" content="width={W},height={H}">
<style>:root{{{tokens.strip()}color-scheme:{theme}}}
{CSS}</style></head>
<body><div class="win">
{strip(active)}
<main class="sheet">
{body}
</main></div></body></html>'''

def row(label, note, ctl, cls=''):
    return f'<div class="row {cls}"><div class="lbl"><b>{label}</b><span>{note}</span></div><div class="ctl">{ctl}</div></div>'

ON, OFF = '<span class="tg on"></span>', '<span class="tg"></span>'

# ---------------------------------------------------------------- tab Gõ
TONE = ('<span class="seg tone">'
        '<span class="on">Kiểu mới <i>·</i> <em>hòa, thúy</em></span>'
        '<span>Kiểu cũ <i>·</i> <em>hoà, thuý</em></span></span>')

def go():
    rows = [
        row('Kiểu đặt dấu', 'Hai kiểu chỉ khác nhau ở vần oa, oe, uy', TONE),
        row('Gõ dấu ở cuối từ', '<span class="kbd">hoangf</span> → hoàng: gõ dấu sau cả từ cũng được', ON),
        row('Tự sửa ưo thành ươ', '<span class="kbd">nguwoif</span> → người, chỉ cần một w', ON, 'hov'),
        row('Kiểm tra chính tả', 'Chỉ thêm dấu khi ra từ tiếng Việt: search vẫn là search', OFF, 'gb'),
        # 27-copy-decisions §A: the dependency on Kiểm tra chính tả is real; keep the disabled state.
        row('Tự động khôi phục từ gõ sai', 'Cần bật Kiểm tra chính tả · <span class="lnk">Bật</span>', '<span class="tg dis" aria-disabled="true"></span>', 'dis'),
        row('Giữ nguyên email, địa chỉ web', 'ten@congty.vn, abc.com/xyz gõ sao ra vậy, không thêm dấu', ON),
        row('Viết hoa đầu câu', 'chào. tôi → chào. Tôi, cả sau ! ? và khi xuống dòng', OFF),
        row('Gõ tắt', 'Gõ <span class="kbd">nth</span> rồi dấu cách → Nguyễn Thu Hà', f'<span class="link" title="Mở tab Gõ tắt">148 mục{ic("chevr","s16")}</span>{ON}', 'gb'),
    ]
    return f'''<div class="status">
  <span class="tg on" role="switch" aria-checked="true" title="Bật/tắt tiếng Việt (Ctrl + Shift)"></span><span class="lab">Tiếng Việt</span>
  <span class="hint">{k('Ctrl', 'Shift', cls='sm')}</span>
  <span class="f"><small>Kiểu gõ</small><span class="sel" style="width:92px">Telex{ic('chev','s16')}</span>
  <small>Bảng mã</small><span class="sel" style="width:164px">Unicode dựng sẵn{ic('chev','s16')}</span></span>
</div>
<div class="list" style="margin-top:8px">{''.join(rows)}</div>
<div class="disc">{ic('chevr')}<b>Nâng cao</b><span class="tag n"><span class="num">9</span>&nbsp;tuỳ chọn</span><span class="m">f j w z làm phụ âm, sửa lỗi dấu gợi ý, Metro, Clipboard</span></div>
<div class="try"><span class="lab">Thử gõ</span><span class="out">Nguyễn Thu Hà ở Đường Nguyễn Văn Linh, Quận 7<span class="caret"></span></span><span class="m">Telex · Unicode</span></div>'''

# ---------------------------------------------------------------- tab Gõ tắt
MACROS = [
    ('ctdd', 'Công ty TNHH Đông Dương', '', ''),
    ('dc', 'được', '', ''),
    ('dchi', 'Số 12 Đường Nguyễn Văn Linh, Phường Tân Phong,', '+1 dòng', ''),
    ('hcm', 'Thành phố Hồ Chí Minh', '', ''),
    ('kg', 'Kính gửi Quý khách hàng,', '', ''),
    ('mail', 'ha.nth@dongduong.com.vn', '', 'on'),
    ('mst', 'Mã số thuế: 0312 456 789', '', ''),
    ('nth', 'Nguyễn Thu Hà', '', 'hov'),
]

def macro():
    rows = []
    for t, e, x, st in MACROS:
        tag = f'<span class="tag n">{ic("enter","s16")}{x}</span>' if x else ''
        rows.append(f'<div class="tr {st}"><span class="c0">{cb("on" if st == "on" else "")}</span>'
                    f'<span class="c1"><span class="k m">{t}</span></span><span class="ex"><em>{e}</em>{tag}</span>'
                    f'<span class="acts"><span class="btn ghost icon sm" title="Sửa">{ic("edit")}</span>'
                    f'<span class="btn ghost icon sm" title="Tuỳ chọn khác">{ic("more")}</span></span></div>')
    # With a selection the 36 px header row ("Từ tắt · Nội dung") is replaced in place by this 36 px bar.
    bulk = (f'<div class="bulk"><span class="n">{cb("mix")}<span>Đã chọn <span class="num">1</span> <small>trong <span class="num">148</span></small></span></span>'
            f'<span class="btn ghost sm">{ic("edit")}Sửa</span><span class="btn ghost sm">{ic("export")}Xuất…</span>'
            f'<span class="btn ghost sm danger">{ic("trash")}Xoá</span><span class="vsep"></span>'
            f'<span class="btn ghost sm" style="gap:6px">Bỏ chọn <span class="k sm">Esc</span></span></div>')
    return f'''<div class="tb"><div class="input">{ic('search')}<span>Tìm từ tắt hoặc nội dung</span><span class="kk">{k('Ctrl','F',cls='sm')}</span></div>
<span class="btn">{ic('import')}Nhập từ tệp…</span><span class="btn">{ic('export')}Xuất ra tệp…</span><span class="btn pri">{ic('plus')}Thêm</span></div>
<div class="tbl">{bulk}{''.join(rows)}</div>
<div class="mfoot"><span>Đang hiện <span class="num">8</span> trong <span class="num">148</span> mục · gõ vào ô tìm để lọc</span><span class="link">Mở bảng gõ tắt{ic('chevr','s16')}</span></div>'''

# ---------------------------------------------------------------- tab Phím tắt
def hk(label, note, field, cls='', msg=''):
    return (f'<div class="row {cls}"><div class="lbl"><b>{label}</b><span>{note}</span>{msg}</div>'
            f'<div class="ctl">{field}</div></div>')

def hotkey():
    # Every action (✕ cancel, ↺ reset, preset ▾) lives inside the well, so all wells end on x = 696.
    f_toggle = f'<div class="cap">{k("Ctrl","Shift")}<span class="split" title="Chọn nhanh">{ic("chev","s16")}</span></div>'
    f_rec = (f'<div class="cap rec">{k("Ctrl","Shift")}<span class="plus">+</span><span class="caret"></span>'
             f'<span class="end"><span class="k sm">Esc</span>huỷ<span class="btn ghost icon xs" title="Huỷ">{ic("x","s16")}</span></span></div>')
    f_method = f'<div class="cap">{k("Ctrl","Alt","V")}</div>'
    f_restore = f'<div class="cap warn">{k("Ctrl","Shift","Z")}<span class="end"><span class="tag warn">{ic("warn","s16")}Dễ trùng</span></span></div>'
    f_open = f'<div class="cap">{k("Ctrl","Alt","K")}<span class="end"><span class="btn ghost icon xs hov" title="Đặt lại mặc định">{ic("reset","s16")}</span></span></div>'
    f_clip = '<div class="cap empty">Chưa đặt<span class="end">Đặt phím</span></div>'
    msg = f'<div class="msg">{ic("warn","s16")}Cũng là phím Làm lại ở nhiều ứng dụng; không có từ để khôi phục thì vẫn Làm lại</div>'
    presets = ''.join(f'<span class="pre{" on" if on else ""}">{k(*ks, cls="sm")}{f"<small>{n}</small>" if n else ""}</span>'
                      for ks, n, on in ((('Ctrl', 'Shift'), 'mặc định', True), (('Alt', 'Z'), 'như UniKey', False),
                                        (('Ctrl', '`'), '', False)))
    return f'''<div class="intro">Bấm vào ô, nhấn tổ hợp mới. <span class="k sm">Esc</span> để huỷ, <span class="k sm">Backspace</span> để xoá.</div>
<div class="sh">Chuyển chế độ</div>
<div class="list">
{hk('Bật/tắt tiếng Việt', 'Ctrl + Shift + T trong trình duyệt vẫn mở lại tab', f_toggle)}
{hk('Đổi bảng mã', 'Unicode → TCVN3 → VNI Windows → Unicode tổ hợp', f_rec, 'rec')}
{hk('Đổi kiểu gõ', 'Telex → VNI → VIQR', f_method)}
</div>
<div class="presets">Chọn nhanh:{presets}</div>
<div class="sh">Công cụ</div>
<div class="list">
{hk('Khôi phục từ gõ sai', 'Đưa từ vừa gõ về đúng phím đã nhấn: gôgle → google', f_restore, 'tall', msg)}
{hk('Mở cài đặt', 'Mặc định là Ctrl + Alt + S', f_open)}
{hk('Chuyển mã chữ đã chép', 'Đổi chữ đã chép sang bảng mã đang dùng, rồi dán như thường', f_clip)}
</div>
<div class="hkfoot"><span class="btn ghost sm" style="margin-left:-10px">{ic('reset')}Đặt lại tất cả</span>
<span class="m">Windows đã dùng: {k('Win','L',cls='sm')}&nbsp; {k('Alt','Tab',cls='sm')}&nbsp; {k('Ctrl','Alt','Del',cls='sm')}</span></div>'''

# ---------------------------------------------------------------- tab Hệ thống
def system():
    seg = lambda *opts: '<span class="seg">' + ''.join(f'<span class="{"on" if i == 0 else ""}">{o}</span>' for i, o in enumerate(opts)) + '</span>'
    excl = ''.join(f'<div class="xr"><span class="tile">{t}</span><b>{n}</b><span class="md">{m}</span></div>'
                   for t, n, m in (('T', 'Windows Terminal', 'Tắt tiếng Việt'), ('C', 'Visual Studio Code', 'Tắt tiếng Việt'),
                                   ('A', 'AutoCAD 2024', 'Bảng mã TCVN3')))
    return f'''<section class="hang"><h3>Khởi động</h3><div class="list">
{row('Khởi động cùng Windows', 'Bật máy là gõ được ngay, biểu tượng nằm cạnh đồng hồ', ON)}
{row(ic('shield','s16') + 'Chạy với quyền Admin', 'Cần mở lại VietKB, Windows sẽ hỏi xác nhận · <span class="lnk bad">Mở lại ngay</span>', '<span class="tg bad on"></span>')}
</div></section>
<section class="hang"><h3>Giao diện</h3><div class="list">
{row('Âm thanh khi bật/tắt', 'Kêu “tách” nhỏ mỗi lần bật hoặc tắt tiếng Việt', OFF)}
{row('Ngôn ngữ hiển thị', 'Chữ trên cửa sổ và menu của VietKB, không đổi cách gõ', seg('Tiếng Việt', 'English'))}
{row('Màu nền', 'Theo Windows: tự đổi khi Windows chuyển sáng hoặc tối', seg('Theo Windows', 'Sáng', 'Tối'))}
</div></section>
<section class="hang"><h3>Cập nhật</h3><div class="list">
{row('Tự kiểm tra cập nhật', 'Đang dùng bản <span class="num">0.1.0</span> · chỉ xem có bản mới, không gửi gì khác', '<span class="btn sm">Kiểm tra ngay</span>' + OFF)}
{row('Tệp cài đặt', '<span class="path">%APPDATA%\\VietKB\\config.toml</span>', f'<span class="btn sm">{ic("folder")}Mở thư mục</span><span class="btn sm danger">{ic("reset")}Đặt lại tất cả…</span>')}
</div></section>
<section class="hang"><h3 class="s">Loại trừ ứng dụng<small><span class="num">6</span> ứng dụng</small><span class="link">Quản lý{ic('chevr','s16')}</span></h3><div class="list">{excl}</div></section>'''

SCREENS = [('go', 'Gõ', go), ('macro', 'Gõ tắt', macro), ('hotkey', 'Phím tắt', hotkey), ('system', 'Hệ thống', system)]

TEST = 'Ấ Ầ Ẩ Ẫ Ậ Ế Ổ Ở Ữ Ỹ người Đường thuở khuỷu Gõ tắt'
RAMP = [
    ('Nhãn dòng', '500 14/20 Text', "500 14px/20px 'Segoe UI Variable Text','Segoe UI'"),
    ('Thử gõ (đầu ra)', '400 16/24 Sitka Text', "400 16px/24px 'Sitka Text','Sitka',Cambria"),
    ('Mẫu kiểu dấu', '400 15/20 Sitka Text', "400 15px/20px 'Sitka Text','Sitka',Cambria"),
    ('Tab · nút · ô chọn', '500 13/20 Text', "500 13px/20px 'Segoe UI Variable Text','Segoe UI'"),
    ('Giải thích', '400 12/18 Small', "400 12px/18px 'Segoe UI Variable Small','Segoe UI'"),
    ('Từ tắt', '500 12.5/16 Cascadia Mono', "500 12.5px/18px 'Cascadia Mono',Consolas"),
]

def build():
    for theme in ('light', 'dark'):
        d = ROOT / theme
        d.mkdir(parents=True, exist_ok=True)
        for fn, title, body in SCREENS:
            (d / f'{fn}.html').write_text(page(theme, title, fn, body()), encoding='utf-8')
    spec = ''.join(f'<div class="r"><code>{n} — {d}</code><div style="font:{f}">{TEST}</div></div>' for n, d, f in RAMP)
    frames = ''.join(f'''<section><h2>{t} <span>{fn}.html</span></h2><div class="pair">
<figure><figcaption>Sáng</figcaption><iframe src="light/{fn}.html" title="{t} — sáng"></iframe></figure>
<figure><figcaption>Tối</figcaption><iframe src="dark/{fn}.html" title="{t} — tối"></iframe></figure></div></section>''' for fn, t, _ in SCREENS)
    idx = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB v3 — cài đặt nhanh 720 × 560</title>
<style>
body{{margin:0;background:#E7E4DE;font:400 14px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1A16;padding:40px 48px 80px;min-width:1560px}}
header{{display:flex;align-items:center;gap:12px}}
header h1{{font:600 26px/36px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0;letter-spacing:-.012em}}
header p{{margin:0 0 0 auto;color:#57524A}}
section{{margin-top:36px}}
h2{{font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;margin:0 0 12px}}
h2 span{{font:400 12.5px/18px "Cascadia Mono",Consolas,monospace;color:#6B655B;margin-left:8px}}
.pair{{display:flex;gap:48px}}
figure{{margin:0}}
figcaption{{font:500 12px/18px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-bottom:8px}}
iframe{{display:block;border:0;width:{W}px;height:{H}px;border-radius:8px;box-shadow:0 0 0 1px rgba(0,0,0,.10),0 24px 60px -20px rgba(30,20,10,.35)}}
.r{{display:grid;grid-template-columns:300px 1fr;align-items:center;gap:24px;padding:6px 0;border-top:1px solid #D9D4CA}}
.r code{{font:400 12.5px/18px "Cascadia Mono",Consolas,monospace;color:#57524A}}
.r div{{background:rgba(88,68,200,.08);overflow:hidden;white-space:nowrap;width:fit-content;padding-right:8px}}
</style></head><body>
<header>{LOGO.replace('var(--acc)', '#5844C8')}<h1>VietKB · cài đặt nhanh</h1><p>4 tab × sáng/tối, đúng kích thước {W} × {H}, không cuộn. Đặc tả: DESIGN-SPEC.md</p></header>
<section><h2>Thang chữ — kiểm tra dấu chồng <span>nền tím nhạt = hộp dòng, overflow: hidden</span></h2>{spec}</section>
{frames}
</body></html>'''
    (ROOT / 'index.html').write_text(idx, encoding='utf-8')

if __name__ == '__main__':
    build()
    print('ok')
