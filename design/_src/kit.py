# Shared tokens, icons and helpers for the VietKB settings mocks (v3).
# ---------------------------------------------------------------- tokens
LIGHT = """
--plane:#F2F0EB;--surface:#FAF9F6;--raised:#FFFFFF;--float:#FFFFFF;--sunken:#EDEAE4;
--line-1:#ECE8E1;--line-2:#E0DBD2;--line-3:#CBC5B9;--ctl-line:#8E887E;
--t1:#1C1A16;--t2:#57524A;--t3:#6B655B;--t4:#A49E92;
--hover:#F2EFE9;--hover-plane:#E9E6DF;--press:#E9E5DD;
--acc:#5844C8;--acc-hover:#4B38B4;--acc-text:#4C38BC;--acc-tint:#F3F1FD;--acc-tint-2:#E8E4FB;--acc-line:#CFC7F6;--acc-a:rgba(88,68,200,.16);--on-acc:#FFFFFF;
--focus:#5844C8;--rail:#5844C8;
--ok:#1B7A4F;--ok-tint:#E6F3EC;
--warn:#8A5200;--warn-tint:#FCF3DF;--warn-line:#E8D2A0;--warn-dot:#C98A12;
--bad:#B42318;--bad-tint:#FCEDEB;--bad-line:#F0C8C2;
--tg-off:#E2DDD4;
--key-top:#FFFFFF;--key-bot:#F6F4F0;--key-line:#D9D3C8;--key-lip:#C9C2B5;
--tile:#ECE8E1;
--e1:0 1px 1px rgba(41,33,20,.04),0 1px 2px rgba(41,33,20,.06);
--e2:0 0 0 1px rgba(41,33,20,.07),0 4px 10px -2px rgba(41,33,20,.08),0 16px 36px -8px rgba(41,33,20,.20);
--knob:0 1px 2px rgba(41,33,20,.22),0 0 0 .5px rgba(41,33,20,.08);
--pri-shadow:0 1px 2px rgba(48,30,140,.28),inset 0 1px 0 rgba(255,255,255,.14);
"""
DARK = """
--plane:#111015;--surface:#17161C;--raised:#1F1E25;--float:#26252D;--sunken:#0F0E13;
--line-1:#232229;--line-2:#2E2D36;--line-3:#403E4A;--ctl-line:#77748A;
--t1:#EEECF3;--t2:#ADAAB8;--t3:#8F8C9C;--t4:#5F5C6A;
--hover:#1D1C23;--hover-plane:#1A1920;--press:#25242C;
--acc:#6B57D8;--acc-hover:#7864DF;--acc-text:#B4A8F4;--acc-tint:#28224A;--acc-tint-2:#342C5E;--acc-line:#5A4D9E;--acc-a:rgba(150,130,245,.26);--on-acc:#FFFFFF;
--focus:#8E7BEA;--rail:#8E7BEA;
--ok:#5DCB9B;--ok-tint:#13261D;
--warn:#E9B34F;--warn-tint:#251E11;--warn-line:#4A3A1B;--warn-dot:#E9B34F;
--bad:#FF8B7D;--bad-tint:#2A1615;--bad-line:#5A2C28;
--tg-off:#2A2931;
--key-top:#2C2B33;--key-bot:#26252D;--key-line:#3D3B46;--key-lip:#0A090D;
--tile:#26252D;
--e1:inset 0 1px 0 rgba(255,255,255,.035),0 1px 2px rgba(0,0,0,.45);
--e2:0 0 0 1px #34333D,inset 0 1px 0 rgba(255,255,255,.05),0 4px 12px rgba(0,0,0,.35),0 18px 44px -8px rgba(0,0,0,.65);
--knob:0 1px 2px rgba(0,0,0,.5);
--pri-shadow:0 1px 2px rgba(0,0,0,.5),inset 0 1px 0 rgba(255,255,255,.16);
"""

# ---------------------------------------------------------------- icons (20x20, stroke 1.5)
I = {
 'overview':'<path d="M3.75 13.25a6.25 6.25 0 1 1 12.5 0"/><path d="M10 13.25l3.1-3.4"/><circle cx="10" cy="13.25" r=".9" fill="currentColor"/><path d="M3.75 16.25h12.5"/>',
 'typing':'<path d="M5 16.25 10 6.5l5 9.75"/><path d="M6.8 12.75h6.4"/><path d="M7.5 2.75c.55 1.25 1.4 1.85 2.5 1.85s1.95-.6 2.5-1.85"/>',
 'charset':'<rect x="3.25" y="3.25" width="13.5" height="13.5" rx="2.75"/><path d="M3.25 8h13.5M8 3.25v13.5"/>',
 'macro':'<path d="M3.75 5.75h6M3.75 10h12.5M3.75 14.25h8.5"/><path d="M13.25 3.6l2.5 2.15-2.5 2.15"/>',
 'exclude':'<circle cx="10" cy="10" r="6.5"/><path d="M5.4 14.6l9.2-9.2"/>',
 'hotkey':'<rect x="3.25" y="3.25" width="13.5" height="13.5" rx="3"/><rect x="6" y="5.5" width="8" height="7.25" rx="1.75"/>',
 'system':'<path d="M3.75 6.5h7.5M14.75 6.5h1.5M3.75 13.5h1.5M8.75 13.5h7.5"/><circle cx="13" cy="6.5" r="1.75"/><circle cx="7" cy="13.5" r="1.75"/>',
 'about':'<circle cx="10" cy="10" r="6.75"/><path d="M10 9.25v4"/><circle cx="10" cy="6.6" r=".8" fill="currentColor" stroke="none"/>',
 'search':'<circle cx="9" cy="9" r="5.25"/><path d="M13 13l3.25 3.25"/>',
 'plus':'<path d="M10 4.75v10.5M4.75 10h10.5"/>',
 'import':'<path d="M10 3.75v8.5M6.75 9.25 10 12.5l3.25-3.25"/><path d="M4 13.25v1.5c0 .83.67 1.5 1.5 1.5h9c.83 0 1.5-.67 1.5-1.5v-1.5"/>',
 'export':'<path d="M10 12.25v-8.5M6.75 7 10 3.75 13.25 7"/><path d="M4 13.25v1.5c0 .83.67 1.5 1.5 1.5h9c.83 0 1.5-.67 1.5-1.5v-1.5"/>',
 'reset':'<path d="M4.6 8.75a5.6 5.6 0 1 1 .95 4.4"/><path d="M4.25 4.75v4h4"/>',
 'warn':'<path d="M8.7 4.25a1.5 1.5 0 0 1 2.6 0l5.3 9.4a1.5 1.5 0 0 1-1.3 2.25H4.7a1.5 1.5 0 0 1-1.3-2.25z"/><path d="M10 8.25v3"/><circle cx="10" cy="13.55" r=".8" fill="currentColor" stroke="none"/>',
 'check':'<path d="M5 10.25l3.25 3.25L15 6.75"/>',
 'checkc':'<circle cx="10" cy="10" r="6.75"/><path d="M7 10.25l2 2 4-4.25"/>',
 'x':'<path d="M5.75 5.75l8.5 8.5M14.25 5.75l-8.5 8.5"/>',
 'chev':'<path d="M6.5 8.25 10 11.75l3.5-3.5"/>',
 'chevup':'<path d="M6.5 11.75 10 8.25l3.5 3.5"/>',
 'chevr':'<path d="M8.25 6.5 11.75 10l-3.5 3.5"/>',
 'more':'<circle cx="5.5" cy="10" r=".9" fill="currentColor"/><circle cx="10" cy="10" r=".9" fill="currentColor"/><circle cx="14.5" cy="10" r=".9" fill="currentColor"/>',
 'edit':'<path d="M12.6 4.4a1.6 1.6 0 0 1 2.3 0l.7.7a1.6 1.6 0 0 1 0 2.3l-8.1 8.1-3.4.75.75-3.4z"/>',
 'trash':'<path d="M4.25 6h11.5M8 6V4.5h4V6M5.75 6l.7 9.75h7.1l.7-9.75"/>',
 'shield':'<path d="M10 3.25 15.5 5.25v4.5c0 3.1-2.2 5.45-5.5 7-3.3-1.55-5.5-3.9-5.5-7v-4.5z"/>',
 'shieldk':'<path d="M10 3.25 15.5 5.25v4.5c0 3.1-2.2 5.45-5.5 7-3.3-1.55-5.5-3.9-5.5-7v-4.5z"/><path d="M7.75 10l1.6 1.6 3-3.1"/>',
 'folder':'<path d="M3.25 6v8.5c0 .69.56 1.25 1.25 1.25h11c.69 0 1.25-.56 1.25-1.25v-6.5c0-.69-.56-1.25-1.25-1.25H10.1L8.4 4.75H4.5c-.69 0-1.25.56-1.25 1.25z"/>',
 'file':'<path d="M5.25 3.25h6l3.5 3.5v10H5.25z"/><path d="M11.25 3.25v3.5h3.5"/>',
 'target':'<circle cx="10" cy="10" r="5.25"/><path d="M10 2.75v3M10 14.25v3M2.75 10h3M14.25 10h3"/>',
 'lock':'<rect x="4.75" y="8.75" width="10.5" height="7.5" rx="1.75"/><path d="M7.25 8.75V6.5a2.75 2.75 0 0 1 5.5 0v2.25"/>',
 'sort':'<path d="M10 14.5v-9M6.75 8.75 10 5.5l3.25 3.25"/>',
 'restart':'<path d="M15.25 10a5.25 5.25 0 1 1-1.55-3.7"/><path d="M15.25 4.25v3h-3"/>',
 'enter':'<path d="M15.25 4.75v4.5a2 2 0 0 1-2 2H5"/><path d="M8 8.25 5 11.25l3 3"/>',
 'arrowr':'<path d="M4.75 10h10.5M11.25 6l4 4-4 4"/>',
}
def ic(n, cls=''):
    return f'<svg class="i {cls}" viewBox="0 0 20 20" aria-hidden="true">{I[n]}</svg>'

CHECK = '<svg viewBox="0 0 12 12"><path d="M2.5 6.25 5 8.6 9.5 3.6"/></svg>'
DASH = '<svg viewBox="0 0 12 12"><path d="M3 6h6"/></svg>'
def cb(state=''):
    inner = CHECK if state == 'on' else DASH if state == 'mix' else ''
    return f'<span class="cb {state}">{inner}</span>'

# Logo "Móc": a dấu hỏi (hook above) over a baseline stroke, on an ink tile. Designed on a 16 grid.
LOGO = ('<svg width="20" height="20" viewBox="0 0 16 16" aria-label="VietKB">'
        '<rect width="16" height="16" rx="4.5" style="fill:var(--acc)"/>'
        '<path d="M6.05 5.35c0-1.2.9-2.05 2.05-2.05 1.15 0 2 .8 2 1.85 0 .9-.6 1.4-1.35 1.75-.45.2-.65.5-.65.95v.55" '
        'fill="none" stroke="#fff" stroke-width="1.45" stroke-linecap="round" stroke-linejoin="round"/>'
        '<path d="M4.75 11.75h6.5" stroke="#fff" stroke-width="1.45" stroke-linecap="round"/></svg>')

def k(*keys, cls=''):
    out = []
    for i, key in enumerate(keys):
        if i: out.append('<span class="plus">+</span>')
        out.append(f'<span class="k {cls}">{key}</span>')
    return ''.join(out)

