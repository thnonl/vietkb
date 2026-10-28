# VietKB settings mocks, v4 — "EVKey/UniKey-simple": one quick window + two small secondary windows + tray menu.
# Run: python design/_src/build.py -> design/{light,dark}/{quick,hotkeys,advanced,tray}.html + design/index.html
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from kit import LIGHT, ic, LOGO, k

# Dark theme v4.7 — Windows 11 Settings layering (neutral greys), lighter mực tím with ink text on fills.
DARK = '''
--plane:#202020;--surface:#272727;--raised:#333333;--float:#2C2C2C;--sunken:#1C1C1C;
--line-1:#333333;--line-2:#3A3A3A;--line-3:#4A4A4A;--ctl-line:#8A8A8A;
--t1:#FFFFFF;--t2:#C5C5C5;--t3:#A8A8A8;--t4:#8A8A8A;
--hover:#3A3A3A;--hover-plane:#2D2D2D;--press:#2B2B2B;
--acc:#8B7BFF;--acc-hover:#9C8CFF;--acc-text:#B9AEFF;--acc-tint:#3A3458;--acc-tint-2:#463E6B;--acc-line:#6A5FB0;--acc-a:rgba(139,123,255,.30);--on-acc:#1C1C1A;
--focus:#B9AEFF;--rail:#8B7BFF;--knob-on:#1C1C1A;
--ok:#6CCB9F;--ok-tint:#1F3A2E;
--warn:#E9B34F;--warn-tint:#3A3020;--warn-line:#5A4A2A;--warn-dot:#E9B34F;
--bad:#FF99A4;--bad-tint:#442726;--bad-line:#6B3A38;
--tg-off:#333333;
--key-top:#3A3A3A;--key-bot:#333333;--key-line:#4A4A4A;--key-lip:#161616;
--tile:#333333;
--e1:inset 0 1px 0 rgba(255,255,255,.05);
--e2:0 0 0 1px #3A3A3A,0 8px 24px rgba(0,0,0,.42);
--knob:0 1px 1px rgba(0,0,0,.35);
--pri-shadow:inset 0 1px 0 rgba(255,255,255,.18);
'''

ROOT = pathlib.Path(__file__).resolve().parent.parent

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
html,body{background:var(--plane)}
body{overflow:hidden;font:400 14px/20px var(--ui);color:var(--t1);font-kerning:normal;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased}
svg.i{width:20px;height:20px;flex:none;fill:none;stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}
svg.i.s16{width:16px;height:16px;stroke-width:1.875}
@keyframes blink{0%,49%{opacity:1}50%,100%{opacity:0}}
@keyframes pulse{0%,100%{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}50%{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 6px transparent}}
@media (prefers-reduced-motion:reduce){*{animation:none!important}}
.num{font-variant-numeric:tabular-nums}

/* window: 36 caption + sheet (content + 56 footer) */
.win{display:flex;flex-direction:column;height:100%}
.cap-bar{height:36px;flex:none;display:flex;align-items:center;padding-left:16px;gap:8px}
.cap-bar .ttl{font:600 13px/20px var(--ui);color:var(--t1)}
.cap-bar .ttl small{font:400 13px/20px var(--ui);color:var(--t3);margin-left:4px}
.cap-bar .drag{flex:1;align-self:stretch}
.thm{width:24px;height:24px;flex:none;margin-right:6px;border-radius:6px;display:grid;place-items:center;color:var(--t2)}
.thm.hov{background:var(--hover-plane);color:var(--t1)}
.thm svg{width:16px;height:16px;fill:none;stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}
.wc{display:flex;align-self:stretch}
.wc span{width:44px;display:grid;place-items:center;color:var(--t2)}
.wc span.hov{background:var(--hover-plane);color:var(--t1)}
.wc span.x.hov{background:#C42B1C;color:#fff}
.wc svg{width:10px;height:10px;fill:none;stroke:currentColor;stroke-width:1}
.sheet{flex:1;min-height:0;display:flex;flex-direction:column;background:var(--surface);border-top:1px solid var(--line-2);padding:0 20px}

/* rows: one column, label left, control right */
.r{height:44px;flex:none;display:flex;align-items:center;gap:12px;position:relative}
.r+.r{box-shadow:inset 0 1px 0 var(--line-1)}
.r.first{box-shadow:none}
.r .l{flex:1;min-width:0;display:flex;align-items:center;gap:8px;white-space:nowrap}
.r .l b{font:500 14px/20px var(--ui);color:var(--t1)}
.r .l .h{font:400 13px/20px var(--ui);color:var(--t3)}
.r .l .h.serif{font:400 15px/20px var(--serif);color:var(--t2)}
.r .l .hk{font:400 13px/20px var(--ui);color:var(--t3);font-variant-numeric:tabular-nums}
.r .c{flex:none;display:flex;align-items:center;gap:10px}
.r.main .l b{font-weight:600}
.r.pair{gap:24px}
.r.pair .grp{display:flex;align-items:center;gap:8px;flex:1}
.r.pair .grp+.grp{justify-content:flex-end}
.r.pair label{font:400 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.sep{height:1px;flex:none;background:var(--line-2);margin:4px 0}

/* footer band, pinned */
.foot{height:56px;flex:none;margin:auto -20px 0;padding:0 20px;display:flex;align-items:center;gap:4px;border-top:1px solid var(--line-2)}
.foot .sp{flex:1}
.lk{display:inline-flex;align-items:center;gap:2px;padding:0 8px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.lk:first-child{margin-left:-8px}
.lk.hov{background:var(--hover);color:var(--t1)}
.lk .i{color:var(--t3)}

/* controls (same anatomy as v3) */
.tg{width:40px;height:24px;border-radius:12px;background:var(--tg-off);box-shadow:inset 0 0 0 1px var(--ctl-line);position:relative;flex:none}
.tg::after{content:"";position:absolute;top:4px;left:4px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:var(--knob)}
.tg.on{background:var(--acc);box-shadow:none}
.tg.on::after{transform:translateX(16px);background:var(--knob-on,#fff)}
.cb{width:16px;height:16px;border-radius:4.5px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--ctl-line);display:inline-grid;place-items:center;flex:none;color:var(--on-acc)}
.cb svg{width:12px;height:12px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
.cb.on{background:var(--acc);box-shadow:none}
.btn{display:inline-flex;align-items:center;gap:6px;padding:0 14px;border-radius:8px;font:500 13px/20px var(--ui);color:var(--t1);background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);white-space:nowrap;flex:none}
.btn.sm{padding:0 10px}
.btn.pri{background:var(--acc);color:var(--on-acc);box-shadow:var(--pri-shadow);min-width:84px;justify-content:center}
.btn.ghost{background:none;box-shadow:none;color:var(--t2)}
.btn .i{color:var(--t2);margin-left:-3px}
.sel{display:inline-flex;align-items:center;gap:8px;padding:0 6px 0 10px;border-radius:8px;background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui);color:var(--t1);white-space:nowrap}
.sel .i{color:var(--t3);margin-left:auto}
.seg{display:inline-flex;padding:2px;border-radius:9px;background:var(--sunken);box-shadow:inset 0 0 0 1px var(--line-1);flex:none}
.seg>span{height:28px;display:inline-flex;align-items:center;padding:0 10px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.seg>span.on{background:var(--raised);color:var(--t1);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2)}
.k{display:inline-flex;align-items:center;justify-content:center;height:22px;min-width:24px;padding:0 7px 1px;border-radius:6px;font:600 12px/16px var(--sm);color:var(--t1);
 background:linear-gradient(var(--key-top),var(--key-bot));box-shadow:inset 0 0 0 1px var(--key-line),inset 0 -2px 0 var(--key-lip);white-space:nowrap;flex:none}
.k.sm{height:20px;min-width:20px;padding:0 5px 1px;border-radius:5px;font-size:11.5px}
.plus{font:400 12px/16px var(--sm);color:var(--t4);margin:0 1px}
.tag{display:inline-flex;align-items:center;gap:4px;height:20px;padding:0 7px;border-radius:6px;font:500 12px/18px var(--sm);white-space:nowrap}
.tag.warn{background:var(--warn-tint);color:var(--warn);box-shadow:inset 0 0 0 1px var(--warn-line)}
.caret{display:inline-block;width:1.5px;height:16px;background:var(--focus);margin-left:4px;animation:blink 1.06s infinite}

/* row controls: one height, one centre line (rows use align-items:center; nothing is nudged) */
.sel,.numf,.btn,.seg,.well,.lk{height:32px;box-sizing:border-box}

/* hotkeys window: the keycap well */
.intro{font:400 13px/20px var(--ui);color:var(--t2);display:flex;align-items:center;gap:4px;height:44px;flex:none;white-space:nowrap}
.well{width:248px;display:flex;align-items:center;gap:3px;padding:0 5px;border-radius:8px;background:var(--sunken);box-shadow:inset 0 1px 1.5px rgba(0,0,0,.06),inset 0 0 0 1px var(--line-1);flex:none}
.well .end{margin-left:auto;display:flex;align-items:center;gap:6px;font:400 12px/18px var(--sm);color:var(--t3);padding-right:3px}
.well.rec{background:var(--raised);animation:pulse 1.6s var(--ease) infinite;box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}
.well.warn{box-shadow:inset 0 1px 1.5px rgba(0,0,0,.06),inset 0 0 0 1px var(--warn-line)}
.well.empty{background:transparent;box-shadow:none;border:1px dashed var(--line-3);padding:0 10px;font:400 13px/20px var(--ui);color:var(--t3)}
.well.empty .end{color:var(--acc-text);font:500 13px/20px var(--ui)}
.r.rec{background:var(--acc-tint);margin:0 -20px;padding:0 20px}
.r.rec+.r{box-shadow:none}

/* advanced window: EVKey-style checkbox grid */
.sh{font:600 13px/20px var(--ui);color:var(--t1);padding:10px 0 2px;border-bottom:1px solid var(--line-2);flex:none}
.cg{display:grid;grid-template-columns:1fr 1fr;column-gap:20px;flex:none}
.ck{height:28px;display:flex;align-items:center;gap:8px;font:400 13px/20px var(--ui);color:var(--t1);white-space:nowrap}
.sr{height:36px;display:grid;grid-template-columns:auto auto 1fr auto auto;align-items:center;column-gap:8px;flex:none}
.sr label{font:400 13px/20px var(--ui);color:var(--t1);white-space:nowrap}
.sr .sp{flex:1}
.numf{width:84px;display:inline-flex;align-items:center;justify-content:space-between;padding:0 10px;border-radius:8px;background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui)}
.numf small{font:400 12px/18px var(--sm);color:var(--t3)}

/* tray mock */
.desk{position:relative;width:100%;height:100%;background:var(--plane);overflow:hidden}
.task{position:absolute;left:0;right:0;bottom:0;height:40px;background:var(--float);box-shadow:0 -1px 0 var(--line-2);display:flex;align-items:center;justify-content:flex-end;gap:2px;padding-right:12px;font:400 12px/16px var(--sm);color:var(--t2)}
.task .ti{width:32px;height:32px;display:grid;place-items:center;border-radius:6px}
.task .ti.on{background:var(--hover)}
.trayv{width:18px;height:18px;border-radius:5px;background:var(--acc);color:#fff;font:700 11px/18px var(--disp);text-align:center}
.task .clock{margin-left:8px;text-align:right;line-height:16px}
.menu{position:absolute;background:var(--float);border-radius:10px;box-shadow:var(--e2);padding:5px;font:400 13px/20px var(--ui);color:var(--t1)}
.mi{height:30px;display:flex;align-items:center;gap:10px;padding:0 10px 0 8px;border-radius:6px;white-space:nowrap}
.mi .ck2{width:16px;display:grid;place-items:center;color:var(--acc-text);flex:none}
.mi .ck2 svg{width:14px;height:14px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
.mi .ks{margin-left:auto;color:var(--t3);font:400 12px/18px var(--sm);padding-left:24px}
.mi .ar{margin-left:auto;color:var(--t3)}
.mi.hl{background:var(--acc-tint);}
.mi b{font-weight:600}
.ms{height:1px;background:var(--line-2);margin:4px 6px}
"""

CHK = '<svg viewBox="0 0 12 12"><path d="M2.5 6.25 5 8.6 9.5 3.6"/></svg>'
WC_GLYPH_MIN = '<svg viewBox="0 0 10 10"><path d="M0 5.5h10"/></svg>'
WC_GLYPH_X = '<svg viewBox="0 0 10 10"><path d="M.5.5l9 9M9.5.5l-9 9"/></svg>'
MARK = LOGO.replace('width="20" height="20"', 'width="16" height="16"')

THEME_ICON = {
    'system': ('Theo Windows', '<circle cx="8" cy="8" r="5.75"/><path d="M8 2.25a5.75 5.75 0 0 1 0 11.5z" fill="currentColor" stroke="none"/>'),
    'light': ('Sáng', '<circle cx="8" cy="8" r="2.75"/><path d="M8 1.5v1.5M8 13v1.5M1.5 8H3M13 8h1.5M3.4 3.4l1.06 1.06M11.54 11.54l1.06 1.06M3.4 12.6l1.06-1.06M11.54 4.46l1.06-1.06"/>'),
    'dark': ('Tối', '<path d="M13.25 9.6A5.5 5.5 0 0 1 6.4 2.75a5.5 5.5 0 1 0 6.85 6.85z"/>'),
}
THEME = {'v': 'system'}

def theme_btn(hov=False):
    name, svg = THEME_ICON[THEME['v']]
    return (f'<span class="thm{" hov" if hov else ""}" role="button" tabindex="0" title="Màu nền: {name}" '
            f'aria-label="Màu nền: {name}. Bấm để đổi"><svg viewBox="0 0 16 16">{svg}</svg></span>')

def capbar(title, sub='', hov=''):
    s = f'<small>{sub}</small>' if sub else ''
    return (f'<header class="cap-bar"><span style="display:flex">{MARK}</span><span class="ttl">{title}{s}</span><span class="drag"></span>'
            f'{theme_btn(hov == "thm")}<div class="wc"><span title="Thu nhỏ" class="{"hov" if hov == "min" else ""}">{WC_GLYPH_MIN}</span>'
            f'<span title="Đóng, VietKB vẫn chạy" class="x {"hov" if hov == "x" else ""}">{WC_GLYPH_X}</span></div></header>')

def page(theme, w, h, body, title='VietKB'):
    tokens = LIGHT if theme == 'light' else DARK
    return f'''<!doctype html>
<html lang="vi" data-theme="{theme}">
<head><meta charset="utf-8"><title>{title}</title>
<meta name="viewport" content="width={w},height={h}">
<style>:root{{{tokens.strip()}color-scheme:{theme}}}
html,body{{width:{w}px;height:{h}px}}
{CSS}</style></head>
<body>{body}</body></html>'''

ON, OFF = '<span class="tg on"></span>', '<span class="tg"></span>'

def r(label, ctl, hint='', cls=''):
    return f'<div class="r {cls}"><div class="l"><b>{label}</b>{hint}</div><div class="c">{ctl}</div></div>'

# ---------------------------------------------------------------- quick window (main)
QW, QH = 480, 384

def quick():
    rows = [
        r('Bật tiếng Việt', ON, '<span class="hk">Ctrl + Shift</span>', 'first main'),
        f'''<div class="r pair"><div class="grp"><label>Kiểu gõ</label><span class="sel" style="width:96px">Telex{ic("chev","s16")}</span></div>
<div class="grp"><label>Bảng mã</label><span class="sel" style="width:164px">Unicode dựng sẵn{ic("chev","s16")}</span></div></div>''',
    ]
    opts = [
        r('Đặt dấu kiểu mới', ON, '<span class="h serif">hòa, thúy</span>', 'first'),
        r('Kiểm tra chính tả', ON),
        r('Cho phép gõ tắt', f'<span class="btn sm">Bảng gõ tắt…</span>{ON}'),
        r('Loại trừ ứng dụng', '<span class="btn sm">Quản lý…</span>', '<span class="h"><span class="num">6</span> ứng dụng</span>'),
    ]
    foot = (f'<footer class="foot"><span class="lk">Phím tắt…</span><span class="lk hov">Nâng cao…</span><span class="sp"></span>'
            f'<span class="btn ghost">Thoát VietKB</span><span class="btn pri">Đóng</span></footer>')
    return (f'<div class="win">{capbar("VietKB", hov="min")}<main class="sheet">'
            f'<div style="height:4px;flex:none"></div>{"".join(rows)}<div class="sep"></div>{"".join(opts)}{foot}</main></div>')

# ---------------------------------------------------------------- hotkeys window
HW, HH = 520, 460

def hotkeys():
    w = lambda *ks: f'<div class="well">{k(*ks)}</div>'
    rows = [
        r('Bật/tắt tiếng Việt', w('Ctrl', 'Shift'), cls='first'),
        r('Đổi bảng mã', f'<div class="well rec">{k("Ctrl","Shift")}<span class="plus">+</span><span class="caret"></span>'
                         f'<span class="end"><span class="k sm">Esc</span>huỷ</span></div>', cls='rec'),
        r('Đổi kiểu gõ', w('Ctrl', 'Alt', 'V')),
        r('Khôi phục từ gõ sai', f'<div class="well warn">{k("Ctrl","Shift","Z")}<span class="end"><span class="tag warn">{ic("warn","s16")}Dễ trùng</span></span></div>'),
        r('Mở cài đặt', w('Ctrl', 'Alt', 'S')),
        r('Chuyển mã chữ đã chép', '<div class="well empty">Chưa đặt<span class="end">Đặt phím</span></div>'),
        r('Kiểu chuyển bật/tắt', '<span class="seg"><span class="on">Khi nhả phím</span><span>Khi nhấn phím</span></span>'),
    ]
    foot = (f'<footer class="foot"><span class="lk">{ic("reset","s16")}Đặt lại tất cả</span><span class="sp"></span>'
            f'<span class="btn pri">Đóng</span></footer>')
    intro = f'<div class="intro">Bấm vào ô, nhấn tổ hợp mới. <span class="k sm">Esc</span> để huỷ, <span class="k sm">Backspace</span> để xoá.</div>'
    return f'<div class="win">{capbar("Phím tắt", "— VietKB", hov="thm")}<main class="sheet">{intro}{"".join(rows)}{foot}</main></div>'

# ---------------------------------------------------------------- advanced window
AW, AH = 560, 496

def advanced():
    ck = lambda label, on=False: f'<div class="ck"><span class="cb {"on" if on else ""}">{CHK if on else ""}</span>{label}</div>'
    go = [ck('Gõ dấu ở cuối từ', True), ck('Tự sửa ưo thành ươ', True),
          ck('Viết hoa đầu câu'), ck('Giữ nguyên email, địa chỉ web', True),
          ck('Cho phép f j w z làm phụ âm'), ck('Tự động khôi phục từ gõ sai'),
          ck('Gõ tắt cả khi tắt tiếng Việt'), ck('Viết hoa theo từ tắt', True)]
    compat = [ck('Hỗ trợ ứng dụng Metro', True), ck('Gửi chữ qua Clipboard'), ck('Tắt khi đổi bàn phím khác', True)]
    system = [ck('Khởi động cùng Windows', True), ck('Chạy với quyền Admin'),
              ck('Âm thanh khi bật/tắt'), ck('Tự kiểm tra cập nhật')]
    sel = lambda t, wpx: f'<span class="sel" style="width:{wpx}px">{t}{ic("chev","s16")}</span>'
    foot = (f'<footer class="foot"><span class="lk">{ic("reset","s16")}Đặt lại tất cả…</span><span class="sp"></span>'
            f'<span class="btn pri">Đóng</span></footer>')
    return (f'<div class="win">{capbar("Nâng cao", "— VietKB", hov="x")}<main class="sheet">'
            f'<div class="sh">Gõ</div><div class="cg">{"".join(go)}</div>'
            f'<div class="sh">Tương thích</div><div class="cg">{"".join(compat)}</div>'
            f'<div class="sr"><label>Sửa lỗi dấu gợi ý</label>{sel("Cách 1", 96)}<span class="sp"></span>'
            f'<label>Độ trễ giữa các phím</label><span class="numf"><span class="num">0</span><small>µs</small></span></div>'
            f'<div class="sh">Hệ thống</div><div class="cg">{"".join(system)}</div>'
            f'<div class="sr"><label>Ngôn ngữ hiển thị</label>{sel("Tiếng Việt", 116)}<span class="sp"></span>'
            f'<label>Màu nền</label>{sel(THEME_ICON[THEME["v"]][0], 132)}</div>'
            f'{foot}</main></div>')

# ---------------------------------------------------------------- tray menu
TW, TH = 440, 360

def tray():
    chk = f'<span class="ck2"><svg viewBox="0 0 14 14"><path d="M3 7.25 5.75 10 11 4.25"/></svg></span>'
    dot = '<span class="ck2"><span style="width:6px;height:6px;border-radius:50%;background:currentColor"></span></span>'
    none = '<span class="ck2"></span>'
    main = (f'<div class="menu" style="right:12px;bottom:48px;width:236px">'
            f'<div class="mi">{chk}<b>Bật tiếng Việt</b><span class="ks">Ctrl+Shift</span></div>'
            f'<div class="ms"></div>'
            f'<div class="mi hl">{none}Kiểu gõ<span class="ar">{ic("chevr","s16")}</span></div>'
            f'<div class="mi">{none}Bảng mã<span class="ar">{ic("chevr","s16")}</span></div>'
            f'<div class="ms"></div>'
            f'<div class="mi">{none}Mở cài đặt…</div>'
            f'<div class="mi">{none}Phím tắt…</div>'
            f'<div class="ms"></div>'
            f'<div class="mi">{none}Thoát VietKB</div></div>')
    sub = (f'<div class="menu" style="right:250px;bottom:124px;width:140px">'
           f'<div class="mi">{dot}<b>Telex</b></div><div class="mi">{none}VNI</div><div class="mi">{none}VIQR</div></div>')
    task = ('<div class="task"><span class="ti">' + ic('chevup', 's16') + '</span><span class="ti on" title="VietKB · Tiếng Việt · Telex · Unicode">'
            '<span class="trayv">V</span></span><span class="clock"><span class="num">09:41</span><br><span class="num">29/10/2026</span></span></div>')
    return f'<div class="desk">{main}{sub}{task}</div>'

SCREENS = [('quick', 'Cài đặt VietKB', QW, QH, quick), ('hotkeys', 'Phím tắt — VietKB', HW, HH, hotkeys),
           ('advanced', 'Nâng cao — VietKB', AW, AH, advanced), ('tray', 'Menu ở biểu tượng VietKB', TW, TH, tray)]

def build():
    for theme in ('light', 'dark'):
        d = ROOT / theme
        d.mkdir(parents=True, exist_ok=True)
        THEME['v'] = theme   # mock state: light files show 'Sáng', dark files 'Tối'
        for fn, title, w, h, body in SCREENS:
            (d / f'{fn}.html').write_text(page(theme, w, h, body(), title), encoding='utf-8')
    frames = ''.join(f'''<section><h2>{t} <span>{fn}.html · {w} × {h}</span></h2><div class="pair">
<figure><figcaption>Sáng</figcaption><iframe src="light/{fn}.html" style="width:{w}px;height:{h}px" title="{t} — sáng"></iframe></figure>
<figure><figcaption>Tối</figcaption><iframe src="dark/{fn}.html" style="width:{w}px;height:{h}px" title="{t} — tối"></iframe></figure></div></section>''' for fn, t, w, h, _ in SCREENS)
    idx = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB v4 — cài đặt kiểu EVKey/UniKey</title>
<style>
body{{margin:0;background:#E7E4DE;font:400 14px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1A16;padding:40px 48px 80px}}
header{{display:flex;align-items:center;gap:12px}} header h1{{font:600 26px/36px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0}}
header p{{margin:0 0 0 24px;color:#57524A}} .states{{display:flex;gap:14px;margin-left:24px;color:#57524A}} .states span{{display:flex;gap:6px;align-items:center}} .states svg{{width:16px;height:16px;fill:none;stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}}
section{{margin-top:36px}} h2{{font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;margin:0 0 12px}}
h2 span{{font:400 12.5px/18px "Cascadia Mono",Consolas,monospace;color:#6B655B;margin-left:8px}}
.pair{{display:flex;gap:48px;align-items:flex-start}} figure{{margin:0}}
figcaption{{font:500 12px/18px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-bottom:8px}}
iframe{{display:block;border:0;border-radius:8px;box-shadow:0 0 0 1px rgba(0,0,0,.10),0 24px 60px -20px rgba(30,20,10,.35)}}
</style></head><body>
<header>{LOGO.replace('var(--acc)', '#5844C8')}<h1>VietKB · cài đặt nhanh (v4)</h1><div class="states">{''.join(f'<span><svg viewBox="0 0 16 16">{v[1]}</svg>{v[0]}</span>' for v in THEME_ICON.values())}</div><p>1 cửa sổ chính + 2 cửa sổ phụ + menu ở biểu tượng. Đặc tả: DESIGN-SPEC.md</p></header>
{frames}</body></html>'''
    (ROOT / 'index.html').write_text(idx, encoding='utf-8')

if __name__ == '__main__':
    build()
    print('ok')
