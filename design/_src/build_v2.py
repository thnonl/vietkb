# VietKB design generator. Emits standalone HTML (inline CSS, no JS, no assets).
# Run: python design/_src/build.py   -> design/light/*.html, design/dark/*.html, design/index.html
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent

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

CSS = r"""
:root{
--ui:"Segoe UI Variable Text","Segoe UI Variable","Segoe UI",system-ui,sans-serif;
--disp:"Segoe UI Variable Display","Segoe UI Variable","Segoe UI",system-ui,sans-serif;
--sm:"Segoe UI Variable Small","Segoe UI Variable","Segoe UI",system-ui,sans-serif;
--serif:"Sitka Text","Sitka",Cambria,Georgia,serif;
--serif-h:"Sitka Heading","Sitka Subheading","Sitka",Cambria,Georgia,serif;
--mono:"Cascadia Mono","Cascadia Code",Consolas,monospace;
--ease:cubic-bezier(.2,.8,.2,1);
}
*{box-sizing:border-box;margin:0;padding:0}
html{background:var(--plane)}
body{width:1040px;height:720px;overflow:hidden;font:400 14px/22px var(--ui);color:var(--t1);background:var(--plane);
 font-kerning:normal;font-feature-settings:"kern" 1;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased}
b{font-weight:600}
.num{font-variant-numeric:tabular-nums}
svg.i{width:20px;height:20px;flex:none;fill:none;stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}
svg.i.s16{width:16px;height:16px;stroke-width:1.875}
@keyframes blink{0%,49%{opacity:1}50%,100%{opacity:0}}
@keyframes pulse{0%,100%{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}50%{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 6px transparent}}
@media (prefers-reduced-motion:reduce){*{animation:none!important;transition:none!important}}

/* ---------- frame ---------- */
.win{display:grid;grid-template-columns:232px 808px;grid-template-rows:720px;height:720px;overflow:hidden}
.side{display:flex;flex-direction:column;padding:0 12px 14px;background:var(--plane);min-height:0;overflow:hidden}
.brand{height:48px;display:flex;align-items:center;gap:10px;padding:0 12px;font:600 15px/22px var(--disp);letter-spacing:-.005em}
.brand .v{margin-left:auto;font:500 12px/18px var(--sm);color:var(--t3)}
.nav{display:flex;flex-direction:column;gap:2px}
.nav+.nav{margin-top:14px}
.navtop{margin-top:10px}
.ni{height:36px;display:flex;align-items:center;gap:12px;padding:0 10px 0 12px;border-radius:8px;color:var(--t2);font:500 14px/22px var(--ui);transition:background .08s}
.ni .i{color:var(--t3)}
.ni .c{margin-left:auto;font:500 12px/18px var(--sm);color:var(--t3);font-variant-numeric:tabular-nums}
.ni.hov{background:var(--hover-plane);color:var(--t1)}
.ni.on{background:var(--raised);color:var(--t1);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2)}
.ni.on .i{color:var(--acc-text)}
.sidefoot{margin-top:auto}
.priv{display:flex;gap:10px;align-items:flex-start;padding:12px 12px 0;margin-top:8px;border-top:1px solid var(--line-2);font:400 12px/18px var(--sm);color:var(--t3)}
.priv .i{color:var(--t3);margin-top:-1px}

.main{background:var(--surface);border-left:1px solid var(--line-2);display:flex;flex-direction:column;position:relative;min-width:0;min-height:0;overflow:hidden}
.strip{height:48px;flex:none;display:flex;align-items:center;gap:2px;padding-left:31px}
.chip{height:28px;display:inline-flex;align-items:center;gap:6px;padding:0 6px 0 8px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.chip.hov{background:var(--hover)}
.chip b{color:var(--t1);font-weight:600}
.chip .i{color:var(--t3)}
.dot{width:8px;height:8px;border-radius:50%;background:var(--acc);box-shadow:0 0 0 3px var(--acc-tint-2);flex:none;margin-right:2px}
.vsep{width:1px;height:14px;background:var(--line-2);margin:0 4px}
.ctx{margin-left:auto;display:flex;align-items:center;gap:8px;font:400 13px/20px var(--ui);color:var(--t3);padding-right:8px;white-space:nowrap}
.ctx b{color:var(--t1);font-weight:500}
.ctx.rec{color:var(--warn);background:var(--warn-tint);box-shadow:inset 0 0 0 1px var(--warn-line);height:28px;padding:0 10px;border-radius:7px;margin-right:8px;font-weight:500}
.ctx.rec .rd{width:6px;height:6px;border-radius:50%;background:var(--warn-dot);animation:blink 1.06s infinite}
.wc{display:flex;align-self:stretch}
.wc span{width:46px;display:grid;place-items:center;color:var(--t2)}
.wc svg{width:10px;height:10px;fill:none;stroke:currentColor;stroke-width:1}

.page{flex:1;min-height:0;padding:12px 40px 16px 39px;position:relative;display:flex;flex-direction:column}
.head{display:flex;align-items:flex-start;gap:24px;margin-bottom:20px;flex:none}
.head h1{font:600 26px/36px var(--disp);letter-spacing:-.012em;color:var(--t1)}
.head h1 .cnt{font:500 14px/22px var(--ui);color:var(--t3);letter-spacing:0;margin-left:10px;font-variant-numeric:tabular-nums}
.head p{font:400 14px/22px var(--ui);color:var(--t2);margin-top:2px}
.head .act{margin-left:auto;display:flex;gap:8px;align-items:center;padding-top:4px;position:relative}

/* ---------- sections & rows ---------- */
.sh{display:flex;align-items:baseline;gap:8px;font:600 13px/20px var(--ui);color:var(--t1);padding-bottom:8px}
.sh .m{font-weight:400;color:var(--t3)}
.sh .r{margin-left:auto}
.rows{border-top:1px solid var(--line-2)}
.row{display:flex;align-items:center;gap:24px;min-height:52px;padding:5px 12px;margin:0 -12px;border-radius:8px;position:relative;transition:background .08s}
.row.s{min-height:44px}
.row+.row::before{content:"";position:absolute;left:12px;right:12px;top:0;height:1px;background:var(--line-1)}
.row.hov{background:var(--hover)}
.row.hov::before,.row.hov+.row::before{opacity:0}
.lbl{flex:1;min-width:0}
.lbl>b{display:block;font:500 14px/22px var(--ui);color:var(--t1)}
.lbl>span{display:block;font:400 13px/20px var(--ui);color:var(--t3)}
.lbl>span .k{vertical-align:1px}
.ctl{flex:none;display:flex;align-items:center;gap:8px}
.hang{display:grid;grid-template-columns:152px 1fr;column-gap:24px}
.hang+.hang{margin-top:12px}
.hang>h3{font:600 13px/20px var(--ui);color:var(--t1);padding-top:7px}
.hang>h3.s{padding-top:12px}
.hang>.rows{border-top:none}
.hang>.rows>.row:first-child::before{content:"";position:absolute;left:12px;right:12px;top:0;height:1px;background:var(--line-2)}

/* ---------- controls ---------- */
.tg{width:36px;height:20px;border-radius:10px;background:var(--tg-off);box-shadow:inset 0 0 0 1px var(--ctl-line);position:relative;flex:none;transition:background .16s var(--ease)}
.tg::after{content:"";position:absolute;top:2px;left:2px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:var(--knob);transition:transform .16s var(--ease)}
.tg.on{background:var(--acc);box-shadow:none}
.tg.dis{opacity:.4}
.tg.on::after{transform:translateX(16px)}
.tg.bad.on{background:var(--bad)}
.tg.lg{width:52px;height:30px;border-radius:15px}
.tg.lg::after{width:24px;height:24px;top:3px;left:3px}
.tg.lg.on::after{transform:translateX(22px)}
.cb{width:16px;height:16px;border-radius:4.5px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--ctl-line);display:inline-grid;place-items:center;flex:none;color:#fff}
.cb svg{width:12px;height:12px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
.cb.on,.cb.mix{background:var(--acc);box-shadow:none}
.rd0{width:16px;height:16px;border-radius:50%;box-shadow:inset 0 0 0 1px var(--ctl-line);background:var(--raised);flex:none}
.rd0.on{box-shadow:inset 0 0 0 5px var(--acc);background:#fff}
.btn{height:32px;display:inline-flex;align-items:center;gap:6px;padding:0 12px;border-radius:8px;font:500 13px/20px var(--ui);color:var(--t1);background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);white-space:nowrap;flex:none}
.btn .i{color:var(--t2);margin-left:-3px}
.btn.pri{background:var(--acc);color:var(--on-acc);box-shadow:var(--pri-shadow)}
.btn.pri .i{color:inherit}
.btn.pri.press{background:var(--acc-hover)}
.btn.ghost{background:none;box-shadow:none;color:var(--t2)}
.btn.icon{width:32px;padding:0;justify-content:center}
.btn.icon .i{margin:0}
.btn.ghost.hov{background:var(--hover);color:var(--t1)}
.btn.danger{color:var(--bad)}
.btn.danger .i{color:var(--bad)}
.btn.sm{height:28px;padding:0 10px;border-radius:7px}
.btn:disabled,.btn.dis{color:var(--t4);box-shadow:inset 0 0 0 1px var(--line-1);background:transparent}
.sel{height:32px;display:inline-flex;align-items:center;gap:8px;padding:0 6px 0 12px;border-radius:8px;background:var(--raised);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui);color:var(--t1);white-space:nowrap}
.sel .i{color:var(--t3);margin-left:auto}
.sel .m{color:var(--t3)}
.seg{display:inline-flex;height:32px;padding:2px;border-radius:9px;background:var(--sunken);box-shadow:inset 0 0 0 1px var(--line-1);flex:none}
.seg span{height:28px;display:inline-flex;align-items:center;padding:0 12px;border-radius:7px;font:500 13px/20px var(--ui);color:var(--t2);white-space:nowrap}
.seg span.on{background:var(--raised);color:var(--t1);box-shadow:var(--e1),inset 0 0 0 1px var(--line-2)}
.seg .serif{font:400 15px/20px var(--serif)}
.seg.tone{height:46px;border-radius:10px}
.seg.tone span{height:42px;flex-direction:column;align-items:flex-start;justify-content:center;padding:0 12px 0 10px;border-radius:8px}
.seg.tone span small{font:500 12px/18px var(--sm);color:var(--t3)}
.seg.tone span.on small{color:var(--acc-text)}
.seg.tone span em{font:400 15px/20px var(--serif);font-style:normal;color:var(--t2)}
.seg.tone span.on em{color:var(--t1)}
.row.tall{min-height:56px}
.k{display:inline-flex;align-items:center;justify-content:center;height:22px;min-width:24px;padding:0 7px 1px;border-radius:6px;font:600 12px/16px var(--sm);color:var(--t1);
 background:linear-gradient(var(--key-top),var(--key-bot));box-shadow:inset 0 0 0 1px var(--key-line),inset 0 -2px 0 var(--key-lip);white-space:nowrap;flex:none}
.k.sm{height:20px;min-width:20px;padding:0 5px 1px;border-radius:5px;font-size:11.5px}
.k.m{font:500 12.5px/16px var(--mono);letter-spacing:-.01em}
.plus{font:400 12px/16px var(--sm);color:var(--t4);margin:0 1px}
.tag{display:inline-flex;align-items:center;gap:4px;height:20px;padding:0 7px;border-radius:6px;font:500 12px/18px var(--sm);white-space:nowrap}
.tag.n{background:var(--sunken);color:var(--t2)}
.tag.acc{background:var(--acc-tint-2);color:var(--acc-text)}
.tag.warn{background:var(--warn-tint);color:var(--warn);box-shadow:inset 0 0 0 1px var(--warn-line)}
.tag.ok{color:var(--ok);padding:0}
.input{height:32px;display:flex;align-items:center;gap:8px;padding:0 6px 0 10px;border-radius:8px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui);color:var(--t3)}
.input .i{color:var(--t3)}
.input .k{margin-left:auto}
.input.focus{box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}
.input.dis{background:transparent;box-shadow:inset 0 0 0 1px var(--line-1);color:var(--t4)}
.input.dis .i{color:var(--t4)}
.btn.dis .i{color:var(--t4)}
.caret{display:inline-block;width:1.5px;height:1.05em;background:var(--acc);vertical-align:-.15em;margin-left:1px;animation:blink 1.06s infinite}
.tile{width:32px;height:32px;border-radius:8px;display:grid;place-items:center;background:var(--tile);box-shadow:inset 0 0 0 1px var(--line-2);font:600 13px/16px var(--disp);color:var(--t2);flex:none}
.tile.sm{width:24px;height:24px;border-radius:6px;font-size:11.5px}
.tile.xs{width:16px;height:16px;border-radius:4px;font-size:9.5px}
.pop{position:absolute;z-index:5;background:var(--float);border-radius:12px;box-shadow:var(--e2)}
.toast{position:absolute;right:24px;bottom:16px;z-index:6;display:flex;align-items:center;gap:12px;height:44px;padding:0 6px 0 12px;border-radius:10px;background:var(--float);box-shadow:var(--e2);font:500 13px/20px var(--ui);color:var(--t1)}
.toast .i{color:var(--ok)}
.toast .cd{font:500 12px/18px var(--sm);color:var(--t3);font-variant-numeric:tabular-nums}
.tip{position:absolute;z-index:6;padding:6px 10px;border-radius:8px;background:var(--float);box-shadow:var(--e2);font:400 12px/18px var(--sm);color:var(--t2);white-space:nowrap;display:flex;align-items:center;gap:4px}
.tip b{color:var(--t1);font-weight:600}

/* ---------- overview ---------- */
.inst{display:grid;grid-template-columns:388px 1fr;border-radius:12px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--line-2);flex:none}
.inst-l{padding:26px 24px 24px;display:flex;gap:18px;border-right:1px solid var(--line-1)}
.inst-l .tg{margin-top:28px}
.live{display:flex;align-items:center;gap:7px;font:500 12px/18px var(--sm);color:var(--ok)}
.live i{width:6px;height:6px;border-radius:50%;background:currentColor;box-shadow:0 0 0 3px var(--ok-tint)}
.state{font:600 32px/44px var(--disp);letter-spacing:-.022em;margin-top:4px}
.hint{display:flex;align-items:center;gap:4px;font:400 13px/20px var(--ui);color:var(--t2);margin-top:6px;white-space:nowrap}
.hint b{color:var(--t1);font-weight:600;margin-left:3px}
.inst-r{padding:8px 20px 8px 24px;display:flex;flex-direction:column;justify-content:center}
.kv{display:flex;align-items:center;justify-content:space-between;height:52px}
.kv+.kv{border-top:1px solid var(--line-1)}
.kv>span{font:500 14px/22px var(--ui)}
.kv>span small{display:block;font:400 12px/18px var(--sm);color:var(--t3)}
.bh{display:flex;align-items:baseline;gap:8px;margin:28px 0 10px;flex:none}
.bh h2{font:600 15px/22px var(--ui)}
.bh .m{margin-left:auto;display:flex;align-items:center;gap:6px;font:400 12px/18px var(--sm);color:var(--t3)}
.try{border-radius:12px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a);flex:none}
.try .out{padding:18px 20px 16px;font:400 22px/34px var(--serif-h);color:var(--t1);letter-spacing:-.003em;white-space:nowrap}
.try .raw{display:flex;gap:12px;align-items:center;padding:9px 20px 10px;border-top:1px solid var(--line-1)}
.try .raw .lab{font:500 12px/18px var(--sm);color:var(--t3);flex:none}
.try .raw code{font:400 12.5px/18px var(--mono);color:var(--t2);flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.try .raw code s{text-decoration:underline;text-decoration-color:var(--acc-line);text-underline-offset:3px;text-decoration-thickness:1px;color:var(--acc-text);font-weight:400}
.try .raw .m{font:400 12px/18px var(--sm);color:var(--t3);flex:none}
.link{color:var(--acc-text);font-weight:500;display:inline-flex;align-items:center;gap:2px}

/* ---------- hotkeys ---------- */
.hk{display:flex;align-items:center;gap:16px;min-height:56px;padding:6px 12px;margin:0 -12px;border-radius:8px;position:relative}
.hk+.hk::before{content:"";position:absolute;left:12px;right:12px;top:0;height:1px;background:var(--line-1)}
.hk.wrap{flex-wrap:wrap;row-gap:0;padding-bottom:10px}
.hk .msg{flex-basis:100%;display:flex;gap:8px;align-items:flex-start;font:400 13px/20px var(--ui);color:var(--warn);padding-left:0;margin-top:2px}
.hk .msg .i{margin-top:0}
.hk .msg span{color:var(--t2)}
.hk.cap-on{background:var(--acc-tint)}
.hk.cap-on::before,.hk.cap-on+.hk::before{opacity:0}
.cap{width:268px;height:32px;display:flex;align-items:center;gap:3px;padding:0 5px;border-radius:8px;background:var(--sunken);box-shadow:inset 0 1px 1.5px rgba(0,0,0,.06),inset 0 0 0 1px var(--line-1);flex:none;position:relative}
.cap .ph{font:400 13px/20px var(--ui);color:var(--t3);margin-left:6px}
.cap .end{margin-left:auto;display:flex;align-items:center;gap:6px;font:400 12px/18px var(--sm);color:var(--t3);padding-right:3px}
.cap .split{margin-left:auto;align-self:stretch;width:30px;display:grid;place-items:center;border-left:1px solid var(--line-2);margin-right:-5px;color:var(--t3)}
.cap.rec{background:var(--raised);animation:pulse 1.6s var(--ease) infinite;box-shadow:inset 0 0 0 1px var(--focus),0 0 0 3px var(--acc-a)}
.cap.rec .caret{height:16px;margin-left:4px}
.cap.warn{box-shadow:inset 0 1px 1.5px rgba(0,0,0,.06),inset 0 0 0 1px var(--warn-line)}
.cap.empty{background:transparent;box-shadow:none;border:1px dashed var(--line-3);padding:0 10px}
.cap.empty .ph{margin:0;color:var(--t3)}
.cap.empty .end{color:var(--acc-text);font:500 13px/20px var(--ui)}
.slot{width:32px;height:32px;flex:none;display:grid;place-items:center;position:relative}
.hkfoot{display:flex;align-items:center;gap:16px;margin-top:auto;padding-top:12px;border-top:1px solid var(--line-2);flex:none}
.hkfoot .m{margin-left:auto;font:400 12px/18px var(--sm);color:var(--t3);display:flex;align-items:center;gap:4px}

/* ---------- macros ---------- */
.mtg{display:grid;grid-template-columns:auto auto;gap:6px 12px;align-items:center;justify-items:end;font:500 13px/20px var(--ui)}
.mtg span{color:var(--t1)}
.mtg span.m{color:var(--t2);font-weight:400}
.bar{display:flex;align-items:center;gap:8px;margin-bottom:12px;flex:none}
.bar .input{width:340px}
.bar .sp{flex:1}
.tbl{flex:1;min-height:0;border-radius:12px;background:var(--raised);border:1px solid var(--line-2);position:relative;overflow:hidden;display:flex;flex-direction:column}
.tscroll{flex:1;min-height:0;overflow-y:scroll}
.tscroll::-webkit-scrollbar{width:12px}
.tscroll::-webkit-scrollbar-thumb{background:var(--line-3);border-radius:6px;border:4px solid transparent;background-clip:content-box}
.tscroll::-webkit-scrollbar-track{background:transparent;margin:40px 0 8px}
.tr{display:grid;grid-template-columns:44px 168px 1fr 76px;align-items:center;height:44px;padding-left:3px;position:relative}
.tr>span:nth-child(2){display:flex;align-items:center;justify-content:flex-start}
.tr+.tr{box-shadow:inset 0 1px 0 var(--line-1)}
.tr.th{height:36px;position:sticky;top:0;z-index:2;background:var(--raised);box-shadow:inset 0 -1px 0 var(--line-2);font:600 12px/18px var(--sm);color:var(--t2)}
.tr.th span{display:flex;align-items:center;gap:2px}
.tr.th .i{color:var(--t3)}
.tr>.c0{display:grid;place-items:center}
.tr .ex{font:400 14px/22px var(--ui);color:var(--t1);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;padding-right:16px;display:flex;gap:8px;align-items:center;min-width:0}
.tr .ex>em{font-style:normal;overflow:hidden;text-overflow:ellipsis;min-width:0}
.tr .acts{display:flex;justify-content:flex-end;gap:2px;padding-right:8px;opacity:0;margin-top:-2px}
.tr.hov{background:var(--hover)}
.tr.hov .acts{opacity:1}
.tr.on{background:var(--acc-tint)}
.tr.on::after{content:"";position:absolute;left:0;top:0;bottom:0;width:2px;background:var(--rail)}
.tr.on+.tr.on{box-shadow:inset 0 1px 0 var(--acc-tint-2)}
.btn.icon.sm{width:28px;height:28px}
.bulk{flex:none;display:flex;align-items:center;gap:4px;height:42px;padding:0 8px 0 17px;background:var(--acc-tint);border-top:1px solid var(--acc-line);font:500 13px/20px var(--ui);white-space:nowrap}
.bulk .n{color:var(--t1);font-weight:600;margin:0 auto 0 0;font-variant-numeric:tabular-nums;display:flex;align-items:center;gap:14px}
.bulk .n small{font:400 13px/20px var(--ui);color:var(--t3)}
.bulk .vsep{height:20px;margin:0 4px}
.bulk .btn.ghost.sm{color:var(--t1)}
.bulk .btn.ghost.danger{color:var(--bad)}

/* ---------- charsets ---------- */
.cs{display:grid;grid-template-columns:236px 236px 1fr;align-items:center;column-gap:16px;min-height:52px;padding:5px 12px 5px 14px;margin:0 -12px;position:relative;border-radius:8px}
.cs+.cs::before{content:"";position:absolute;left:12px;right:12px;top:0;height:1px;background:var(--line-1)}
.cs.th{min-height:28px;padding-top:0;padding-bottom:8px;font:500 12px/18px var(--sm);color:var(--t3);border-bottom:1px solid var(--line-2);border-radius:0;margin-bottom:0}
.cs.th+.cs::before{display:none}
.cs .nm{display:flex;gap:12px;align-items:flex-start}
.cs .nm .rd0{margin-top:3px}
.cs .nm b{display:block;font:500 14px/22px var(--ui);white-space:nowrap}
.cs .nm>div>span{display:block;font:400 12px/18px var(--sm);color:var(--t3)}
.cs .smp{font:400 16px/24px var(--serif);color:var(--t1);white-space:nowrap}
.cs .smp.bad{color:var(--t2)}
.cs .by{font:400 12.5px/18px var(--mono);color:var(--t3);white-space:nowrap;display:flex;gap:6px}
.cs .by s{text-decoration:none;color:var(--t1);font-weight:600;background:var(--sunken);border-radius:4px;padding:0 3px;margin:0 -3px}
.cs.on{background:var(--acc-tint)}
.cs.on::after{content:"";position:absolute;left:0;top:10px;bottom:10px;width:2px;border-radius:2px;background:var(--rail)}
.cs.on::before,.cs.on+.cs::before{opacity:0}
.cs.on .by s{background:var(--acc-tint-2);color:var(--acc-text)}
.cs.hov{background:var(--hover)}
.cs.hov::before,.cs.hov+.cs::before{opacity:0}
.saved{display:inline-flex;align-items:center;gap:3px;font:500 12px/18px var(--sm);color:var(--ok);margin-left:8px;vertical-align:1px}
.fnote{font:400 12px/18px var(--sm);color:var(--t3);padding:10px 0 0 30px}

/* ---------- exclude ---------- */
.ex-row .tile{margin-right:-8px}
.ex-row .path{display:block;font:400 12px/18px var(--mono);color:var(--t3);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:330px}
.ex-row .path.den{font-family:var(--sm);display:flex;gap:6px;align-items:center}
.ex-row .sel{width:212px}
.picker{right:-1px;top:44px;width:356px;padding:6px}
.picker .input{margin:2px 2px 6px}
.plabel{display:flex;justify-content:space-between;padding:8px 10px 4px;font:500 12px/18px var(--sm);color:var(--t3)}
.pi{display:flex;align-items:center;gap:10px;height:40px;padding:0 10px;border-radius:8px;font:500 13px/20px var(--ui);color:var(--t1)}
.pi .x{font:400 12px/18px var(--mono);color:var(--t3);margin-left:auto}
.pi.hl{background:var(--acc-tint);box-shadow:inset 0 0 0 1px var(--acc-line)}
.pi.hl .x{color:var(--acc-text)}
.pi.dis{color:var(--t4)}
.pi.dis .tile{opacity:.55}
.pi.dis .x{color:var(--t4);font-family:var(--sm)}
.pi .i{color:var(--t3)}
.pdiv{height:1px;background:var(--line-2);margin:6px -6px}
.pfoot{display:flex;gap:6px;align-items:center;padding:8px 10px 4px;font:400 12px/18px var(--sm);color:var(--t3)}

.sugg{display:flex;align-items:center;gap:8px;margin-top:16px;padding-top:16px;border-top:1px solid var(--line-2)}
.sugg .m{font:400 13px/20px var(--ui);color:var(--t3);margin-right:4px}
.sg{height:28px;display:inline-flex;align-items:center;gap:7px;padding:0 6px 0 6px;border-radius:7px;box-shadow:inset 0 0 0 1px var(--line-2);font:500 13px/20px var(--ui);color:var(--t2)}
.sg .i{color:var(--t3)}

/* ---------- typing ---------- */
.grid2{display:grid;grid-template-columns:352px 352px;column-gap:24px;row-gap:16px;flex:none}
.grid2 .lbl>span{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.kbd{font:400 12.5px/20px var(--mono);color:var(--t2)}
.lnk{color:var(--acc-text);font-weight:500}
.row.dis .lbl>b{color:var(--t3)}
.adv{margin-top:16px;border:1px solid var(--line-2);border-radius:12px;flex:none}
.adv-h{display:flex;align-items:center;gap:10px;height:44px;padding:0 6px 0 12px;border-bottom:1px solid var(--line-1);font:600 13px/20px var(--ui)}
.adv-h>.i{color:var(--t2);margin-right:-2px}
.adv-h .m{margin-left:auto;font:400 12px/18px var(--sm);color:var(--t3)}
.adv .grid2.in{grid-template-columns:1fr 1fr;padding:0 12px}
.adv .rows{border-top:none}
.num-in{height:32px;width:88px;display:inline-flex;align-items:center;justify-content:space-between;padding:0 10px;border-radius:8px;background:var(--raised);box-shadow:inset 0 0 0 1px var(--line-2);font:400 13px/20px var(--ui);color:var(--t1)}
.num-in .u{color:var(--t3)}

/* ---------- macros empty ---------- */
.mempty{flex:1;min-height:0;border:1px solid var(--line-2);border-radius:12px;background:var(--raised);display:grid;grid-template-columns:1fr 320px;overflow:hidden}
.em-l{padding:0 40px 24px 40px;display:flex;flex-direction:column;align-items:flex-start;justify-content:center}
.em-demo{display:flex;align-items:center;gap:12px;margin-bottom:28px}
.em-demo .cap{width:auto;gap:5px;padding:0 5px}
.em-demo>.i{color:var(--t3)}
.em-out{font:400 22px/34px var(--serif-h);color:var(--t1)}
.em-out .caret{height:22px;vertical-align:-4px}
.mempty h2{font:600 20px/28px var(--disp);letter-spacing:-.008em;color:var(--t1)}
.mempty .em-l p{font:400 14px/22px var(--ui);color:var(--t2);margin-top:6px;max-width:400px}
.em-act{display:flex;gap:8px;margin-top:24px}
.em-r{border-left:1px solid var(--line-1);background:var(--surface);padding:24px 24px 20px;display:flex;flex-direction:column}
.em-r .em-note{margin-top:auto}
.em-r .sh{padding-bottom:10px}
.exr{display:flex;align-items:center;gap:8px;height:52px;border-top:1px solid var(--line-1)}
.exr .tc{width:52px;flex:none;display:flex}
.exr .btn .i{color:var(--acc-text)}
.exr .o{font:400 14px/22px var(--ui);color:var(--t1);flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.exr .btn{margin-right:-6px}
.em-note{font:400 12px/18px var(--sm);color:var(--t3);padding-top:12px;border-top:1px solid var(--line-1)}

/* ---------- system ---------- */
.notice{display:flex;align-items:center;gap:10px;margin:2px 0 6px;padding:7px 8px 7px 12px;border-radius:8px;background:var(--bad-tint);box-shadow:inset 0 0 0 1px var(--bad-line);font:400 13px/20px var(--ui);color:var(--t1)}
.notice .i{color:var(--bad)}
.notice .btn{margin-left:auto;height:28px}
.row .ico-bad{color:var(--bad)}
.lbl b .i{vertical-align:-4px;margin-right:4px}
.path-c{font:400 12.5px/20px var(--mono);color:var(--t2)}
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

# ---------------------------------------------------------------- frame
NAV = [
    [('overview', 'overview', 'Tổng quan', None)],
    [('typing', 'typing', 'Gõ tiếng Việt', None), ('charsets', 'charset', 'Bảng mã', None), ('macros', 'macro', 'Gõ tắt', '148')],
    [('exclude', 'exclude', 'Loại trừ ứng dụng', '6'), ('hotkeys', 'hotkey', 'Phím tắt', None), ('system', 'system', 'Hệ thống', None)],
]

def sidebar(active, hover=None):
    groups = []
    for gi, g in enumerate(NAV):
        items = []
        for key, icon, label, cnt in g:
            cls = 'ni' + (' on' if key == active else '') + (' hov' if key == hover else '')
            c = f'<span class="c">{cnt}</span>' if cnt else ''
            items.append(f'<div class="{cls}">{ic(icon)}<span>{label}</span>{c}</div>')
        groups.append(f'<nav class="nav{" navtop" if gi == 0 else ""}">{"".join(items)}</nav>')
    return f'''<aside class="side">
<div class="brand">{LOGO}<span>VietKB</span><span class="v num">0.1</span></div>
{''.join(groups)}
<div class="sidefoot">
<nav class="nav"><div class="ni">{ic('about')}<span>Giới thiệu</span></div></nav>
<div class="priv">{ic('shieldk')}<span>Không ghi lại, không gửi đi bất kỳ phím nào bạn gõ.</span></div>
</div></aside>'''

WC = ('<div class="wc">'
      '<span><svg viewBox="0 0 10 10"><path d="M0 5.5h10"/></svg></span>'
      '<span><svg viewBox="0 0 10 10"><rect x=".5" y=".5" width="9" height="9" rx="1.5"/></svg></span>'
      '<span><svg viewBox="0 0 10 10"><path d="M.5.5l9 9M9.5.5l-9 9"/></svg></span></div>')

def strip(ctx=None, hov=None):
    chips = [
        ('state', f'<span class="dot"></span><b>Tiếng Việt</b>{ic("chev","s16")}'),
        ('method', f'Telex{ic("chev","s16")}'),
        ('charset', f'Unicode dựng sẵn{ic("chev","s16")}'),
    ]
    parts = []
    for i, (key, html) in enumerate(chips):
        if i: parts.append('<span class="vsep"></span>')
        parts.append(f'<span class="chip{" hov" if key == hov else ""}">{html}</span>')
    if ctx is None:
        ctx = f'<span class="ctx">Đang ở <span class="tile xs">Z</span><b>Zalo</b> · theo cài đặt chung</span>'
    return f'<header class="strip">{"".join(parts)}{ctx}{WC}</header>'

def page(theme, title, active, body, strip_html=None, nav_hover=None):
    tokens = LIGHT if theme == 'light' else DARK
    return f'''<!doctype html>
<html lang="vi" data-theme="{theme}">
<head><meta charset="utf-8"><title>VietKB — {title} ({'sáng' if theme == 'light' else 'tối'})</title>
<meta name="viewport" content="width=1040,height=720">
<style>:root{{{tokens.strip()}color-scheme:{theme}}}
{CSS}</style></head>
<body><div class="win">
{sidebar(active, nav_hover)}
<main class="main">
{strip_html or strip()}
<div class="page">
{body}
</div></main></div></body></html>'''

# ---------------------------------------------------------------- screens
# Tone style: default is kiểu mới (modern_tone = true). Two-line pill: name (sans 12) over example (Sitka 15).
TONE = ('<span class="seg tone">'
        '<span class="on"><small>Kiểu mới</small><em>hoà, thuý</em></span>'
        '<span><small>Kiểu cũ</small><em>hòa, thúy</em></span></span>')

# Telex for "Nguyễn Thu Hà ở Đường Nguyễn Văn Linh, Quận 7" (modifier keys marked):
#   Nguyeenx -> Nguy + ee(ê) + n + x(ngã on ê) = Nguyễn ; Thu ; Haf -> Ha + f(huyền) = Hà
#   owr -> o + w(ơ) + r(hỏi) = ở ; DDuwowngf -> DD(Đ) u+w(ư) o+w(ơ) ng + f(huyền on ơ) = Đường
#   Vawn -> V a+w(ă) n = Văn ; Linh ; Quaanj -> Qu + aa(â) + n + j(nặng on â) = Quận ; 7
RAW = ('Nguy<s>ee</s>n<s>x</s> Thu Ha<s>f</s> o<s>wr</s> <s>DD</s>u<s>w</s>o<s>w</s>ng<s>f</s> '
       'Nguy<s>ee</s>n<s>x</s> Va<s>w</s>n Linh, Qua<s>a</s>n<s>j</s> 7')

def overview():
    return f'''<div class="head"><div><h1>Tổng quan</h1><p>Mọi thay đổi được lưu ngay khi bạn chỉnh.</p></div></div>
<section class="inst">
  <div class="inst-l">
    <div class="tg lg on" role="switch" aria-checked="true"></div>
    <div>
      <div class="live"><i></i>Bộ gõ đang hoạt động</div>
      <div class="state">Tiếng Việt</div>
      <div class="hint">{k('Ctrl','Shift',cls='sm')}&nbsp;rồi nhả để chuyển sang<b>English</b></div>
    </div>
  </div>
  <div class="inst-r">
    <div class="kv"><span>Kiểu gõ<small>aa → â, w → ư, s → sắc</small></span><span class="seg"><span class="on">Telex</span><span>VNI</span><span>VIQR</span></span></div>
    <div class="kv"><span>Bảng mã<small>Chuẩn hiện nay</small></span><span class="sel" style="width:172px">Unicode dựng sẵn{ic('chev','s16')}</span></div>
  </div>
</section>
<div class="bh"><h2>Thử gõ</h2><span class="m">{ic('lock','s16')}Chỉ để thử — không lưu, xoá khi đóng cửa sổ</span></div>
<div class="try">
  <div class="out">Nguyễn Thu Hà ở Đường Nguyễn Văn Linh, Quận 7<span class="caret"></span></div>
  <div class="raw"><span class="lab">Phím đã gõ</span><code>{RAW}</code><span class="m">Telex · Unicode</span></div>
</div>
<div class="bh"><h2>Hay dùng</h2></div>
<div class="rows">
  <div class="row tall hov"><div class="lbl"><b>Kiểu đặt dấu</b><span>Vị trí dấu thanh trong vần oa, oe, uy.</span></div><div class="ctl">{TONE}</div></div>
  <div class="row"><div class="lbl"><b>Kiểm tra chính tả</b><span>Đang tắt: dấu được đặt cả vào “search”, “excel”.</span></div><div class="ctl"><span class="tg"></span></div></div>
  <div class="row"><div class="lbl"><b>Gõ tắt</b><span><span class="num">148</span> mục · gõ <span class="k m sm">nth</span> thành Nguyễn Thu Hà</span></div><div class="ctl"><span class="link">Mở bảng{ic('chevr','s16')}</span><span class="tg on"></span></div></div>
</div>'''

def hk_row(label, desc, field, slot='', cls='', msg=''):
    return (f'<div class="hk {cls}"><div class="lbl"><b>{label}</b><span>{desc}</span></div>'
            f'{field}<div class="slot">{slot}</div>{msg}</div>')

def hotkeys():
    f_toggle = f'<div class="cap">{k("Ctrl","Shift")}<span class="tag n" style="margin-left:6px">Nhả để chuyển</span><span class="split">{ic("chev","s16")}</span></div>'
    f_rec = f'<div class="cap rec">{k("Ctrl","Shift")}<span class="plus">+</span><span class="caret"></span><span class="end"><span class="k sm">Esc</span>huỷ</span></div>'
    f_method = f'<div class="cap">{k("Ctrl","Alt","V")}</div>'
    f_macro = f'<div class="cap">{k("Ctrl","Alt","G")}</div>'
    f_restore = f'<div class="cap warn">{k("Ctrl","Shift","Z")}<span class="end"><span class="tag warn">{ic("warn","s16")}Trùng</span></span></div>'
    f_open = f'<div class="cap">{k("Ctrl","Alt","K")}</div>'
    f_clip = f'<div class="cap empty"><span class="ph">Chưa đặt</span><span class="end">Đặt phím</span></div>'
    reset_hov = f'<span class="btn ghost icon hov" title="Về mặc định">{ic("reset")}</span>'
    msg = (f'<div class="msg">{ic("warn","s16")}<span>Trùng “Làm lại” trong Word, Photoshop. Khi không có từ để khôi phục, phím vẫn đến ứng dụng.</span></div>')
    return f'''<div class="head"><div><h1>Phím tắt</h1><p>Nhấn vào ô để ghi tổ hợp mới. <span class="k sm">Esc</span> để huỷ, <span class="k sm">Backspace</span> để bỏ trống.</p></div></div>
<div class="sh">Chuyển chế độ</div>
<div class="rows">
{hk_row('Bật/tắt tiếng Việt', 'Ctrl + Shift + T trong trình duyệt vẫn mở lại tab như thường.', f_toggle)}
{hk_row('Đổi bảng mã', 'Unicode → TCVN3 → VNI Windows → Unicode', f_rec, f'<span class="btn ghost icon">{ic("x")}</span>', 'cap-on')}
{hk_row('Đổi kiểu gõ', 'Telex → VNI → VIQR', f_method)}
{hk_row('Bật/tắt gõ tắt', 'Giữ nguyên chế độ tiếng Việt', f_macro)}
</div>
<div class="sh" style="margin-top:20px">Công cụ</div>
<div class="rows">
{hk_row('Khôi phục từ gõ sai', 'Đưa từ vừa gõ về đúng các phím đã bấm', f_restore, '', 'wrap', msg)}
{hk_row('Mở cài đặt', 'Đã đổi từ Ctrl + Alt + S', f_open, reset_hov)}
{hk_row('Chuyển mã clipboard', 'Đổi văn bản đã sao chép sang bảng mã đang dùng', f_clip)}
</div>
<div class="hkfoot"><span class="btn ghost" style="margin-left:-12px">{ic('reset')}Khôi phục tất cả</span>
<span class="m">Không gán được phím hệ thống: {k('Win','L',cls='sm')}&nbsp; {k('Alt','Tab',cls='sm')}&nbsp; {k('Ctrl','Alt','Del',cls='sm')}</span></div>'''

def hotkeys_strip():
    ctx = f'<span class="ctx rec"><span class="rd"></span>Đang ghi phím — các phím tắt tạm dừng</span>'
    return strip(ctx=ctx)

MACROS = [
    ('ctdd', 'Công ty TNHH Đông Dương', '', 'on'),
    ('dc', 'được', '', ''),
    ('dchi', 'Số 12 Đường Nguyễn Văn Linh, Phường Tân Phong,', '+1 dòng', ''),
    ('hcm', 'Thành phố Hồ Chí Minh', '', ''),
    ('kg', 'Kính gửi Quý khách hàng,', '', ''),
    ('mail', 'ha.nth@dongduong.com.vn', '', 'on'),
    ('mst', 'Mã số thuế: 0312 456 789', '', 'on'),
    ('ngta', 'người ta', '', ''),
    ('nth', 'Nguyễn Thu Hà', '', 'hov'),
    ('sdt', '0903 418 265', '', ''),
    ('stk', 'STK 0071 0012 34567 — Vietcombank, CN Tân Định', '', ''),
    ('tgian', 'thời gian', '', ''),
    ('tn', 'thuở nhỏ', '', ''),
    ('tt', 'Trân trọng,', '+2 dòng', ''),
]

def macros():
    rows = []
    for trig, exp, extra, st in MACROS:
        x = f'<span class="tag n">{ic("enter","s16")}{extra}</span>' if extra else ''
        rows.append(f'<div class="tr {st}"><span class="c0">{cb("on" if st == "on" else "")}</span>'
                    f'<span><span class="k m">{trig}</span></span><span class="ex"><em>{exp}</em>{x}</span>'
                    f'<span class="acts"><span class="btn ghost icon sm">{ic("edit")}</span><span class="btn ghost icon sm">{ic("more")}</span></span></div>')
    return f'''<div class="head"><div><h1>Gõ tắt<span class="cnt">148 mục</span></h1><p>Gõ từ viết tắt rồi nhấn <span class="k sm">Space</span> hoặc <span class="k sm">Enter</span> để mở rộng.</p></div>
<div class="act mtg"><span>Bật gõ tắt</span><span class="tg on"></span><span class="m">Cả khi tắt tiếng Việt</span><span class="tg"></span></div></div>
<div class="bar"><div class="input">{ic('search')}<span>Tìm theo từ tắt hoặc nội dung</span>{k('Ctrl','F',cls='sm')}</div><span class="sp"></span>
<span class="btn">{ic('import')}Nhập…</span><span class="btn">{ic('export')}Xuất…</span><span class="btn pri">{ic('plus')}Thêm gõ tắt</span></div>
<div class="tbl"><div class="tscroll">
<div class="tr th"><span class="c0">{cb('mix')}</span><span>Từ tắt{ic('sort','s16')}</span><span>Nội dung</span><span></span></div>
{''.join(rows)}
</div>
<div class="bulk"><span class="n">{cb('on')}<span>Đã chọn <span class="num">3</span> <small>trong <span class="num">148</span> mục</small></span></span><span class="btn ghost sm">{ic('export')}Xuất mục đã chọn</span><span class="btn ghost sm danger">{ic('trash')}Xoá</span><span class="vsep"></span><span class="btn ghost sm" style="gap:6px">Bỏ chọn <span class="k sm">Esc</span></span></div>
</div>'''

CHARSETS = [
    ('Unicode dựng sẵn', 'Mặc định · mọi ứng dụng hiện đại', 'Tiếng Việt có dấu', False, 'V i <s>U+1EC7</s> t', 'on'),
    ('Unicode tổ hợp', 'Dấu là ký tự riêng, ghép sau chữ', 'Tiếng Việt có dấu', False, 'V i e <s>U+0323</s> <s>U+0302</s> t', ''),
    ('TCVN3 (ABC)', 'Phông .VnTime, .VnArial', 'TiÕng ViÖt cã dÊu', True, '56 69 <s>D6</s> 74', 'hov'),
    ('VNI Windows', 'Phông VNI-Times, VNI-Helve', 'Tieáng Vieät coù daáu', True, '56 69 65 <s>E4</s> 74', ''),
    ('VISCII', 'Văn bản và phông thập niên 90', 'Tiªng Vi®t có d¤u', True, '56 69 <s>AE</s> 74', ''),
    ('VIQR', 'Chỉ ký tự ASCII, cho email cũ', "Tie^'ng Vie^.t co' da^'u", True, 'V i e <s>^</s> <s>.</s> t', ''),
]

def charsets():
    rows = []
    for name, desc, sample, bad, by, st in CHARSETS:
        saved = f'<span class="saved">{ic("check","s16")}Đã lưu</span>' if st == 'on' else ''
        rows.append(f'<div class="cs {st}"><div class="nm"><span class="rd0 {"on" if st == "on" else ""}"></span>'
                    f'<div><b>{name}{saved}</b><span>{desc}</span></div></div>'
                    f'<div class="smp{" bad" if bad else ""}">{sample}</div><div class="by">{by}</div></div>')
    return f'''<div class="head"><div><h1>Bảng mã</h1><p>Cách chữ có dấu được mã hoá trước khi gửi vào ứng dụng. BK HCM1 và VPS sẽ có ở bản sau.</p></div></div>
<div class="cs th"><span style="padding-left:28px">Bảng mã</span><span>Hiển thị với phông thường</span><span>Chữ “Việt” khi gửi đi</span></div>
{''.join(rows)}
<div class="sh" style="margin-top:22px">Tương thích</div>
<div class="rows">
  <div class="row"><div class="lbl"><b>Sửa lỗi dấu khi có gợi ý</b><span>Trình duyệt và Excel tự điền chữ gợi ý làm mất dấu vừa gõ.</span></div><div class="ctl"><span class="seg"><span>Tắt</span><span class="on">Cách 1</span><span>Cách 2</span></span></div></div>
  <div class="row s"><div class="lbl"><b>Hỗ trợ ứng dụng Metro</b></div><div class="ctl"><span class="tg on"></span></div></div>
  <div class="row"><div class="lbl"><b>Gửi phím qua Clipboard</b><span>Chỉ bật khi ứng dụng không nhận phím. Nội dung Clipboard được trả lại sau đó.</span></div><div class="ctl"><span class="tg"></span></div></div>
</div>'''

RULES = [
    ('T', 'Windows Terminal', r'C:\Program Files\WindowsApps\Microsoft.WindowsTerminal\WindowsTerminal.exe', 'Tắt tiếng Việt', ''),
    ('C', 'Visual Studio Code', r'C:\Users\thuha\AppData\Local\Programs\Microsoft VS Code\Code.exe', 'Tắt tiếng Việt', ''),
    ('L', 'Liên Minh Huyền Thoại', r'D:\Riot Games\League of Legends\LeagueClient.exe', 'Tắt tiếng Việt', ''),
    ('A', 'AutoCAD 2024', r'C:\Program Files\Autodesk\AutoCAD 2024\acad.exe', 'Tuỳ chỉnh <span class="m">· TCVN3</span>', ''),
    ('X', 'Microsoft Excel', r'C:\Program Files\Microsoft Office\root\Office16\EXCEL.EXE', 'Tuỳ chỉnh <span class="m">· Clipboard</span>', ''),
    ('M', 'MsMpEng.exe', None, 'Tắt tiếng Việt', ''),
]
PICK = [
    ('Z', 'Zalo', 'Zalo.exe', 'hl'),
    ('G', 'Google Chrome', 'chrome.exe', ''),
    ('W', 'Microsoft Word', 'WINWORD.EXE', ''),
    ('N', 'Notepad++', 'notepad++.exe', ''),
    ('X', 'Microsoft Excel', 'Đã thêm', 'dis'),
    ('T', 'Windows Terminal', 'Đã thêm', 'dis'),
]

def exclude():
    rows = []
    for t, name, path, mode, st in RULES:
        p = (f'<span class="path">{path}</span>' if path else
             f'<span class="path den">{ic("lock","s16")}Không đọc được đường dẫn — cần quyền Admin</span>')
        rows.append(f'<div class="row ex-row {st}"><span class="tile">{t}</span><div class="lbl"><b>{name}</b>{p}</div>'
                    f'<div class="ctl"><span class="sel">{mode}{ic("chev","s16")}</span><span class="btn ghost icon">{ic("more")}</span></div></div>')
    sugg = ''.join(f'<span class="sg"><span class="tile xs">{t}</span>{n}{ic("plus","s16")}</span>'
                   for t, n in (('S', 'Steam'), ('V', 'Valorant'), ('P', 'PowerShell'), ('I', 'IntelliJ IDEA')))
    picks = ''.join(f'<div class="pi {st}"><span class="tile sm">{t}</span>{n}<span class="x">{x}</span></div>' for t, n, x, st in PICK)
    pop = f'''<div class="pop picker">
<div class="input focus">{ic('search')}<span>Tìm ứng dụng đang chạy</span></div>
<div class="plabel"><span>Đang chạy</span><span class="num">14</span></div>
{picks}
<div class="pdiv"></div>
<div class="pi">{ic('file')}Chọn tệp .exe…</div>
<div class="pi dis">{ic('target')}Nhấp chọn một cửa sổ<span class="x"><span class="tag n">Sắp có</span></span></div>
<div class="pfoot">{k('↑',cls='sm')}{k('↓',cls='sm')} chọn &nbsp;·&nbsp; {k('Enter',cls='sm')} thêm</div>
</div>'''
    return f'''<div class="head"><div><h1>Loại trừ ứng dụng</h1><p>Tự tắt hoặc đổi cách gõ khi bạn chuyển sang các ứng dụng này.</p></div>
<div class="act"><span class="btn pri press">{ic('plus')}Thêm ứng dụng</span>{pop}</div></div>
<div class="rows">
  <div class="row"><div class="lbl"><b>Nhớ bật/tắt theo từng ứng dụng</b><span>Mỗi ứng dụng giữ trạng thái tiếng Việt riêng khi bạn chuyển qua lại.</span></div><div class="ctl"><span class="tg"></span></div></div>
</div>
<div class="sh" style="margin-top:24px">Quy tắc <span class="m num">6</span></div>
<div class="rows">{''.join(rows)}</div>
<div class="sugg"><span class="m">Thường được loại trừ</span>{sugg}</div>'''

def system():
    return f'''<div class="head"><div><h1>Hệ thống</h1><p>Khởi động, thông báo và dữ liệu cấu hình.</p></div></div>
<section class="hang"><h3>Khởi động</h3><div class="rows">
  <div class="row"><div class="lbl"><b>Khởi động cùng Windows</b><span>Chạy ở khay hệ thống, dùng khoảng <span class="num">14</span> MB bộ nhớ.</span></div><div class="ctl"><span class="tg on"></span></div></div>
  <div class="row s"><div class="lbl"><b>Mở cửa sổ này khi khởi động</b></div><div class="ctl"><span class="tg"></span></div></div>
  <div class="row"><div class="lbl"><b>{ic('shield','ico-bad')}Chạy với quyền Admin</b><span>Để gõ được trong Task Manager, Regedit. Windows sẽ hỏi UAC mỗi lần khởi động.</span></div><div class="ctl"><span class="tg bad on"></span></div></div>
  <div class="notice">{ic('restart')}Có hiệu lực sau khi khởi động lại VietKB.<span class="btn danger">Khởi động lại ngay</span></div>
</div></section>
<section class="hang"><h3 class="s">Thông báo</h3><div class="rows">
  <div class="row s"><div class="lbl"><b>Thông báo khi bật/tắt tiếng Việt</b></div><div class="ctl"><span class="tg"></span></div></div>
  <div class="row"><div class="lbl"><b>Hiện chữ V / E cạnh con trỏ</b><span>Trong <span class="num">0,6</span> giây mỗi lần chuyển chế độ.</span></div><div class="ctl"><span class="tg on"></span></div></div>
</div></section>
<section class="hang"><h3 class="s">Khay và giao diện</h3><div class="rows">
  <div class="row s"><div class="lbl"><b>Nhấp vào biểu tượng khay</b></div><div class="ctl"><span class="sel" style="width:196px">Bật/tắt tiếng Việt{ic('chev','s16')}</span></div></div>
  <div class="row"><div class="lbl"><b>Giao diện</b><span>Chủ đề và ngôn ngữ, áp dụng ngay</span></div><div class="ctl"><span class="seg"><span class="on">Theo hệ thống</span><span>Sáng</span><span>Tối</span></span><span class="seg"><span class="on">VI</span><span>EN</span></span></div></div>
</div></section>
<section class="hang"><h3>Dữ liệu</h3><div class="rows">
  <div class="row"><div class="lbl"><b>Tệp cấu hình</b><span class="path-c">%APPDATA%\\VietKB\\config.toml</span></div><div class="ctl"><span class="btn">{ic('folder')}Mở thư mục</span><span class="btn">{ic('export')}Xuất…</span></div></div>
  <div class="row s"><div class="lbl"><b>Khôi phục toàn bộ cài đặt</b></div><div class="ctl"><span class="btn danger">{ic('reset')}Khôi phục…</span></div></div>
</div></section>
<div class="toast">{ic('checkc')}Đã khôi phục mặc định cho Thông báo<span class="cd">4 giây</span><span class="btn sm">Hoàn tác</span></div>'''

def trow(label, desc, ctl, cls=''):
    return f'<div class="row {cls}"><div class="lbl"><b>{label}</b><span>{desc}</span></div><div class="ctl">{ctl}</div></div>'

def group(title, rows):
    return f'<section><div class="sh">{title}</div><div class="rows">{"".join(rows)}</div></section>'

def typing():
    on, off = '<span class="tg on"></span>', '<span class="tg"></span>'
    sel = f'<span class="sel" style="width:112px">Telex{ic("chev","s16")}</span>'
    g1 = group('Dấu thanh', [
        trow('Kiểu gõ', 'aa → â, ow → ơ, s → dấu sắc', sel),
        trow('Kiểu đặt dấu', 'Dấu trong oa, oe, uy', TONE, 'tall'),
        trow('Bỏ dấu ở cuối từ', '<span class="kbd">hoangf</span> → hoàng, dấu gõ sau cả từ', on),
        trow('Tự sửa ưo thành ươ', '<span class="kbd">nguwoif</span> → người, chỉ cần một w', on, 'hov'),
    ])
    g2 = group('Chính tả và khôi phục', [
        trow('Kiểm tra chính tả', 'Không đặt dấu vào “search”, “excel”', off),
        trow('Tự động khôi phục từ gõ sai', f'Cần bật Kiểm tra chính tả · <span class="lnk">Bật</span>', '<span class="tg dis" aria-disabled="true"></span>', 'dis'),
        trow('Bảo vệ đường dẫn và mã', 'Không thêm dấu trong URL, email, mã lệnh', on),
        trow('Viết hoa sau dấu câu', 'Sau dấu . ! ? và khi xuống dòng', off),
    ])
    g3 = group('Tương thích ứng dụng', [
        trow('Tắt khi bàn phím không phải US', 'Ví dụ bàn phím Pháp, Đức, Nhật', on),
        trow('Loại trừ ứng dụng', '<span class="num">6</span> ứng dụng có quy tắc riêng', f'<span class="link">Quản lý{ic("chevr","s16")}</span>'),
    ])
    g4 = group('Gõ tắt', [
        trow('Bật gõ tắt', f'<span class="num">148</span> mục · <span class="lnk">Mở bảng gõ tắt</span>', on),
        trow('Gõ tắt cả khi tắt tiếng Việt', 'Từ tắt vẫn mở rộng khi đang ở English', off),
    ])
    adv_l = ''.join([
        trow('Dùng f, j, w, z làm phụ âm', 'Cho tên riêng, từ mượn: Facebook, jazz', off),
        trow('Giữ hoa/thường theo từ tắt', '<span class="kbd">NTH</span> → NGUYỄN THU HÀ', on),
    ])
    adv_r = ''.join([
        trow('Độ trễ giữa các phím', 'Cho ứng dụng cũ hay bỏ sót phím', '<span class="num-in"><span class="num">0</span><span class="u">µs</span></span>'),
        trow('Luôn gửi qua Clipboard', '<span class="num">2</span> ứng dụng: AutoCAD, Excel', f'<span class="link">Sửa{ic("chevr","s16")}</span>'),
    ])
    return f'''<div class="head"><div><h1>Gõ tiếng Việt</h1><p>Cách bộ gõ đặt dấu, bắt lỗi và làm việc với từng ứng dụng.</p></div>
<div class="act"><span class="btn ghost">{ic('reset')}Khôi phục mặc định</span></div></div>
<div class="grid2">{g1}{g2}{g3}{g4}</div>
<section class="adv"><div class="adv-h">{ic('chev')}<b>Nâng cao</b><span class="tag n"><span class="num">4</span>&nbsp;tuỳ chọn</span><span class="m">Ít khi cần đổi</span><span class="btn ghost sm">Thu gọn</span></div>
<div class="grid2 in"><div class="rows">{adv_l}</div><div class="rows">{adv_r}</div></div></section>'''

def macros_empty():
    ex = [('nth', 'Nguyễn Thu Hà'), ('vn', 'Việt Nam'), ('hcm', 'Thành phố Hồ Chí Minh'),
          ('tks', 'Cảm ơn anh/chị nhiều.'), ('ctdd', 'Công ty TNHH Đông Dương')]
    exrows = ''.join(f'<div class="exr"><span class="tc"><span class="k m">{t}</span></span><span class="o">{o}</span><span class="btn ghost icon sm" title="Thêm">{ic("plus")}</span></div>' for t, o in ex)
    return f'''<div class="head"><div><h1>Gõ tắt<span class="cnt">0 mục</span></h1><p>Gõ từ viết tắt rồi nhấn <span class="k sm">Space</span> hoặc <span class="k sm">Enter</span> để mở rộng.</p></div>
<div class="act mtg"><span>Bật gõ tắt</span><span class="tg on"></span><span class="m">Cả khi tắt tiếng Việt</span><span class="tg"></span></div></div>
<div class="bar"><div class="input dis">{ic('search')}<span>Chưa có gì để tìm</span></div><span class="sp"></span>
<span class="btn">{ic('import')}Nhập…</span><span class="btn dis">{ic('export')}Xuất…</span></div>
<div class="mempty">
  <div class="em-l">
    <div class="em-demo"><span class="cap"><span class="k m">nth</span><span class="k sm">Space</span></span>{ic('arrowr')}<span class="em-out">Nguyễn Thu Hà<span class="caret"></span></span></div>
    <h2>Chưa có gõ tắt nào</h2>
    <p>Gõ một từ ngắn rồi nhấn Space, VietKB thay bằng cả cụm bạn đã lưu: tên, địa chỉ, câu chào.</p>
    <div class="em-act"><span class="btn pri">{ic('plus')}Thêm gõ tắt</span><span class="btn">{ic('import')}Nhập từ tệp…</span></div>
  </div>
  <div class="em-r"><div class="sh">Hoặc bắt đầu với</div>{exrows}<p class="em-note">Mẫu chỉ được thêm khi bạn bấm Thêm; sửa hoặc xoá lúc nào cũng được.</p></div>
</div>'''

SCREENS = [
    ('overview', 'Tổng quan', 'overview', overview, None, 'macros'),
    ('typing', 'Gõ tiếng Việt', 'typing', typing, None, None),
    ('hotkeys', 'Phím tắt', 'hotkeys', hotkeys, hotkeys_strip, None),
    ('macros', 'Gõ tắt', 'macros', macros, None, None),
    ('macros-empty', 'Gõ tắt — chưa có mục', 'macros', macros_empty, None, None),
    ('charsets', 'Bảng mã', 'charsets', charsets, None, None),
    ('exclude', 'Loại trừ ứng dụng', 'exclude', exclude, None, None),
    ('system', 'Hệ thống', 'system', system, None, None),
]

TEST = 'Ấ Ầ Ẩ Ẫ Ậ Ế Ổ Ở Ữ Ỹ người Đường thuở khuỷu Gõ tắt'
RAMP = [
    ('Hiển thị · trạng thái', '600 32/44 Display −0.022em', '600 32px/44px "Segoe UI Variable Display","Segoe UI"', '-.022em'),
    ('Tiêu đề trang', '600 26/36 Display −0.012em', '600 26px/36px "Segoe UI Variable Display","Segoe UI"', '-.012em'),
    ('Thử gõ (đầu ra)', '400 22/34 Sitka Heading', '400 22px/34px "Sitka Heading","Sitka",Cambria', '-.003em'),
    ('Mẫu bảng mã', '400 16/24 Sitka Text', '400 16px/24px "Sitka Text","Sitka",Cambria', '0'),
    ('Tiêu đề nhóm', '600 15/22 Text', '600 15px/22px "Segoe UI Variable Text","Segoe UI"', '0'),
    ('Nhãn dòng', '500 14/22 Text', '500 14px/22px "Segoe UI Variable Text","Segoe UI"', '0'),
    ('Mô tả · nút', '400 13/20 Text', '400 13px/20px "Segoe UI Variable Text","Segoe UI"', '0'),
    ('Chú thích · phím', '500 12/18 Small', '500 12px/18px "Segoe UI Variable Small","Segoe UI"', '0'),
    ('Mã · từ tắt', '400 12.5/18 Cascadia Mono', '400 12.5px/18px "Cascadia Mono",Consolas', '0'),
]
SPECIMEN = ''.join(f'<div class="r"><code>{n} — {d}</code><div style="font:{f.replace(chr(34), chr(39))};letter-spacing:{t}">{TEST}</div></div>'
                   for n, d, f, t in RAMP)

def build():
    for theme in ('light', 'dark'):
        d = ROOT / theme
        d.mkdir(parents=True, exist_ok=True)
        for fn, title, active, body, st, nh in SCREENS:
            html = page(theme, title, active, body(), st() if st else None, nh)
            if fn == 'macros-empty':
                html = html.replace('<span class="c">148</span>', '')
            (d / f'{fn}.html').write_text(html, encoding='utf-8')
    frames = []
    for fn, title, *_ in SCREENS:
        frames.append(f'''<section><h2>{title} <span>{fn}.html</span></h2><div class="pair">
<figure><figcaption>Sáng</figcaption><iframe src="light/{fn}.html" width="1040" height="720" loading="lazy" title="{title} — sáng"></iframe></figure>
<figure><figcaption>Tối</figcaption><iframe src="dark/{fn}.html" width="1040" height="720" loading="lazy" title="{title} — tối"></iframe></figure>
</div></section>''')
    idx = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8"><title>VietKB — bộ màn hình thiết kế</title>
<style>
body{{margin:0;background:#E7E4DE;font:400 14px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;color:#1C1A16;padding:40px 48px 80px;min-width:2180px}}
header{{display:flex;align-items:center;gap:12px;margin-bottom:8px}}
header h1{{font:600 26px/36px "Segoe UI Variable Display","Segoe UI",sans-serif;margin:0;letter-spacing:-.012em}}
header p{{margin:0 0 0 auto;color:#57524A}}
section{{margin-top:40px}}
h2{{font:600 15px/22px "Segoe UI Variable Text","Segoe UI",sans-serif;margin:0 0 12px}}
h2 span{{font:400 12.5px/18px "Cascadia Mono",Consolas,monospace;color:#6B655B;margin-left:8px}}
.pair{{display:flex;gap:48px}}
figure{{margin:0}}
figcaption{{font:500 12px/18px "Segoe UI Variable Small","Segoe UI",sans-serif;color:#6B655B;margin-bottom:8px}}
.spec .r{{display:grid;grid-template-columns:260px 1fr;align-items:center;gap:24px;padding:6px 0;border-top:1px solid #D9D4CA}}
.spec .r code{{font:400 12.5px/18px "Cascadia Mono",Consolas,monospace;color:#57524A}}
.spec .r div{{background:rgba(88,68,200,.08);overflow:hidden;white-space:nowrap}}
iframe{{display:block;border:0;width:1040px;height:720px;border-radius:10px;box-shadow:0 0 0 1px rgba(0,0,0,.10),0 24px 60px -20px rgba(30,20,10,.35)}}
</style></head><body>
<header>{LOGO.replace('var(--acc)', '#5844C8')}<h1>VietKB · Cài đặt</h1><p>8 màn hình × sáng/tối, đúng kích thước 1040 × 720. Đặc tả: DESIGN-SPEC.md</p></header>
<section class="spec"><h2>Thang chữ — kiểm tra dấu chồng <span>nền tím nhạt = hộp dòng; không ký tự nào được tràn ra ngoài</span></h2>
{SPECIMEN}
</section>
{''.join(frames)}
</body></html>'''
    (ROOT / 'index.html').write_text(idx, encoding='utf-8')

if __name__ == '__main__':
    build()
    print('ok')
