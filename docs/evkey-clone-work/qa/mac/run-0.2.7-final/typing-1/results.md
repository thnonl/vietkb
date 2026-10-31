| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|---|
| gui | - | GUI-1 |  | `settings window opens` | `present` | `PASS` | PASS | shots/gui-settings-window.png |
| gui | - | GUI-2 |  | `Accessibility banner NOT shown (permission granted)` | `present` | `PASS` | PASS | gui-settings-ax.txt + shots/gui-settings-window.png |
| gui | - | GUI-3 |  | `menu-bar status item present` | `present` | `PASS` | PASS | shots/gui-statusitem-on.png |
| gui | - | GUI-4 |  | `menu-bar menu opens` | `present` | `PASS` | PASS | shots/gui-statusitem-menu.png |
| gui | - | GUI-4b |  | `left click on the status item toggles V/E and a second left click toggles back (M10)` | `present` | `PASS` | PASS | shots/gui-statusitem-left-0-start.png / -1-after-click.png / -2-after-second-click.png |
| gui | - | GUI-4c |  | `original V/E state restored after the left-click checks (asserted before GUI-5)` | `present` | `PASS` | PASS |  |
| secure | telex | SEC-1 |  | `dduwowcj Vieejt` | `dduwowcj Vieejt` | `dduwowcj Vieejt` | PASS | shots/secure-dialog-telex.png |
| gui | - | GUI-5 |  | `{CTRL+SHIFT} x2 with 'tieengs' typed while off` | `icon image changes V -> E -> V; text raw while off` | `icon differs: true, restored: true; text while off: tieengs` | PASS | shots/gui-statusitem-V-before-toggle.png / shots/gui-statusitem-E-after-toggle.png / shots/gui-statusitem-V-after-second-toggle.png |
| textedit-plain | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-telex-TX-B1.png |
| textedit-plain | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| textedit-plain | telex | TX-B4 |  | `hoaf` | `hòa` | `hòa` | PASS |  |
| textedit-plain | telex | TX-B5 |  | `quaan` | `quân` | `quân` | PASS |  |
| textedit-plain | telex | TX-B6 |  | `tieengs` | `tiếng` | `tiếng` | PASS |  |
| textedit-plain | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | telex | TX-S3 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | telex | TX-S4 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | telex | TX-E1 |  | `ddeei` | `ddeei` | `ddeei` | PASS |  |
| textedit-plain | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E3a |  | `ddeei{BS}{BS}{BS}ddeem` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E3b |  | `ddeei{BS}{BS}{BS}ddeens` | `đến` | `đến` | PASS |  |
| textedit-plain | telex | TX-E4 |  | `ddeei{BS}{BS}eem` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E5 |  | `pass{BS}{BS}{BS}dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-E6 |  | `dduwowcjk{BS}` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-E7 |  | `gifk{BS}` | `gì` | `gì` | PASS |  |
| textedit-plain | telex | TX-E8 |  | `test{BS}` | `té` | `té` | PASS |  |
| textedit-plain | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| textedit-plain | telex | TX-E9b |  | `3giowf` | `3giờ` | `3giờ` | PASS |  |
| textedit-plain | telex | TX-E10 |  | `Tha {BS}s` | `Thá` | `Thá` | PASS |  |
| textedit-plain | telex | TX-E11 |  | `ddeei {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E12 |  | `tieengs{BS}{BS}{BS}` | `ti` | `ti` | PASS |  |
| textedit-plain | telex | TX-E13 |  | `tieengs1` | `tiếng1` | `tiếng1` | PASS |  |
| textedit-plain | telex | TX-E14 |  | `tieeng{BS}{BS}{BS}eeng` | `tiêng` | `tiêng` | PASS |  |
| textedit-plain | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C2 |  | `{SHIFT}vieetj{/SHIFT}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C3 |  | `DDUWOWCJ` | `ĐƯỢC` | `ĐƯỢC` | PASS |  |
| textedit-plain | telex | TX-C4 |  | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-H1 |  | `{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs` | `tieengs tiếng` | `tieengs tiếng` | PASS |  |
| textedit-plain | telex | TX-H2 |  | `vieejt{CTRL+SHIFT+Z}` | `vieejt` | `vieejt` | PASS |  |
| textedit-plain | telex | TX-H3 |  | `vie{LEFT}{RIGHT}et` | `vieet` | `vieet` | PASS |  |
| textedit-rich | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-rich-telex-TX-B1.png |
| textedit-rich | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| textedit-rich | telex | TX-B4 |  | `hoaf` | `hòa` | `hòa` | PASS |  |
| textedit-rich | telex | TX-B5 |  | `quaan` | `quân` | `quân` | PASS |  |
| textedit-rich | telex | TX-B6 |  | `tieengs` | `tiếng` | `tiếng` | PASS |  |
| textedit-rich | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-rich | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-rich | telex | TX-S3 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-rich | telex | TX-S4 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-rich | telex | TX-E1 |  | `ddeei` | `ddeei` | `ddeei` | PASS |  |
| textedit-rich | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E3a |  | `ddeei{BS}{BS}{BS}ddeem` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E3b |  | `ddeei{BS}{BS}{BS}ddeens` | `đến` | `đến` | PASS |  |
| textedit-rich | telex | TX-E4 |  | `ddeei{BS}{BS}eem` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E5 |  | `pass{BS}{BS}{BS}dduwowcj` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-E6 |  | `dduwowcjk{BS}` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-E7 |  | `gifk{BS}` | `gì` | `gì` | PASS |  |
| textedit-rich | telex | TX-E8 |  | `test{BS}` | `té` | `té` | PASS |  |
| textedit-rich | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| textedit-rich | telex | TX-E9b |  | `3giowf` | `3giờ` | `3giờ` | PASS |  |
| textedit-rich | telex | TX-E10 |  | `Tha {BS}s` | `Thá` | `Thá` | PASS |  |
| textedit-rich | telex | TX-E11 |  | `ddeei {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E12 |  | `tieengs{BS}{BS}{BS}` | `ti` | `ti` | PASS |  |
| textedit-rich | telex | TX-E13 |  | `tieengs1` | `tiếng1` | `tiếng1` | PASS |  |
| textedit-rich | telex | TX-E14 |  | `tieeng{BS}{BS}{BS}eeng` | `tiêng` | `tiêng` | PASS |  |
| textedit-rich | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-C2 |  | `{SHIFT}vieetj{/SHIFT}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-C3 |  | `DDUWOWCJ` | `ĐƯỢC` | `ĐƯỢC` | PASS |  |
| textedit-rich | telex | TX-C4 |  | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-H1 |  | `{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs` | `tieengs tiếng` | `tieengs tiếng` | PASS |  |
| textedit-rich | telex | TX-H2 |  | `vieejt{CTRL+SHIFT+Z}` | `vieejt` | `vieejt` | PASS |  |
| textedit-rich | telex | TX-H3 |  | `vie{LEFT}{RIGHT}et` | `vieet` | `vieet` | PASS |  |
| notes | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS |  |
| notes | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| notes | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| notes | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| notes | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| notes | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| notes | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| notes | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| stickies | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/stickies-telex-TX-B1.png |
| stickies | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| stickies | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| stickies | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| stickies | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| stickies | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| stickies | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| stickies | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| chrome-textarea | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/chrome-textarea-telex-TX-B1.png |
| chrome-textarea | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| chrome-textarea | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| chrome-textarea | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| chrome-textarea | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| chrome-textarea | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| chrome-textarea | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| chrome-textarea | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| chrome-omnibox | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/chrome-omnibox-telex-TX-B1.png |
| chrome-omnibox | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| chrome-omnibox | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| vscode | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/vscode-telex-TX-B1.png |
| vscode | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| vscode | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| vscode | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| vscode | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| vscode | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| vscode | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| vscode | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| terminal | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/terminal-telex-TX-B1.png |
| terminal | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| terminal | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| terminal | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| terminal | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj` | `được được được` | `được được được` | PASS |  |
| terminal | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| terminal | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| terminal | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| secure | vni | SEC-1 |  | `d9u7o7c5 Vie65t` | `d9u7o7c5 Vie65t` | `d9u7o7c5 Vie65t` | PASS | shots/secure-dialog-vni.png |
| textedit-plain | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-vni-VN-B1.png |
| textedit-plain | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| textedit-plain | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| textedit-plain | vni | VN-B4 |  | `hoa2` | `hòa` | `hòa` | PASS |  |
| textedit-plain | vni | VN-B5 |  | `qua6n` | `quân` | `quân` | PASS |  |
| textedit-plain | vni | VN-B6 |  | `tie6ng1` | `tiếng` | `tiếng` | PASS |  |
| textedit-plain | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | vni | VN-S3 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | vni | VN-S4 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | vni | VN-E1 |  | `d9e6i` | `d9e6i` | `d9e6i` | PASS |  |
| textedit-plain | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E3a |  | `d9e6i{BS}{BS}{BS}d9e6m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E3b |  | `d9e6i{BS}{BS}{BS}d9e6n1` | `đến` | `đến` | PASS |  |
| textedit-plain | vni | VN-E4 |  | `d9e6i{BS}{BS}e6m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E8 |  | `d9e6i{BS}` | `đê` | `đê` | PASS |  |
| textedit-plain | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| textedit-plain | vni | VN-E10 |  | `Tha {BS}1` | `Thá` | `Thá` | PASS |  |
| textedit-plain | vni | VN-E11 |  | `d9e6i {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E12 |  | `nghie6ng2{BS}` | `nghiền` | `nghiền` | PASS |  |
| textedit-plain | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-X1 |  | `VIE6T` | `VIÊT` | `VIÊT` | PASS |  |
| textedit-plain | vni | VN-X2 |  | `VIE6` | `VIÊ` | `VIÊ` | PASS |  |
| textedit-plain | vni | VN-X3 |  | `VIE5` | `VỊE` | `VỊE` | PASS |  |
| textedit-plain | vni | VN-X4 |  | `Vie6T` | `ViêT` | `ViêT` | PASS |  |
| textedit-plain | vni | VN-C4 |  | `{CAPS}vie6t5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-H1 |  | `{CTRL+SHIFT}tie6ng1 {CTRL+SHIFT}tie6ng1` | `tie6ng1 tiếng` | `tie6ng1 tiếng` | PASS |  |
| textedit-plain | vni | VN-H2 |  | `vie6t5{CTRL+SHIFT+Z}` | `vie6t5` | `vie6t5` | PASS |  |
| textedit-plain | vni | VN-H3 |  | `vie{LEFT}{RIGHT}6t5` | `vie6t5` | `vie6t5` | PASS |  |
| textedit-rich | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-rich-vni-VN-B1.png |
| textedit-rich | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| textedit-rich | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| textedit-rich | vni | VN-B4 |  | `hoa2` | `hòa` | `hòa` | PASS |  |
| textedit-rich | vni | VN-B5 |  | `qua6n` | `quân` | `quân` | PASS |  |
| textedit-rich | vni | VN-B6 |  | `tie6ng1` | `tiếng` | `tiếng` | PASS |  |
| textedit-rich | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-rich | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-rich | vni | VN-S3 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-rich | vni | VN-S4 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-rich | vni | VN-E1 |  | `d9e6i` | `d9e6i` | `d9e6i` | PASS |  |
| textedit-rich | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | vni | VN-E3a |  | `d9e6i{BS}{BS}{BS}d9e6m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | vni | VN-E3b |  | `d9e6i{BS}{BS}{BS}d9e6n1` | `đến` | `đến` | PASS |  |
| textedit-rich | vni | VN-E4 |  | `d9e6i{BS}{BS}e6m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | vni | VN-E8 |  | `d9e6i{BS}` | `đê` | `đê` | PASS |  |
| textedit-rich | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| textedit-rich | vni | VN-E10 |  | `Tha {BS}1` | `Thá` | `Thá` | PASS |  |
| textedit-rich | vni | VN-E11 |  | `d9e6i {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | vni | VN-E12 |  | `nghie6ng2{BS}` | `nghiền` | `nghiền` | PASS |  |
| textedit-rich | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | vni | VN-X1 |  | `VIE6T` | `VIÊT` | `VIÊT` | PASS |  |
| textedit-rich | vni | VN-X2 |  | `VIE6` | `VIÊ` | `VIÊ` | PASS |  |
| textedit-rich | vni | VN-X3 |  | `VIE5` | `VỊE` | `VỊE` | PASS |  |
| textedit-rich | vni | VN-X4 |  | `Vie6T` | `ViêT` | `ViêT` | PASS |  |
| textedit-rich | vni | VN-C4 |  | `{CAPS}vie6t5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | vni | VN-H1 |  | `{CTRL+SHIFT}tie6ng1 {CTRL+SHIFT}tie6ng1` | `tie6ng1 tiếng` | `tie6ng1 tiếng` | PASS |  |
| textedit-rich | vni | VN-H2 |  | `vie6t5{CTRL+SHIFT+Z}` | `vie6t5` | `vie6t5` | PASS |  |
| textedit-rich | vni | VN-H3 |  | `vie{LEFT}{RIGHT}6t5` | `vie6t5` | `vie6t5` | PASS |  |
| notes | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS |  |
| notes | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| notes | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| notes | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| notes | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| notes | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| notes | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| notes | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| stickies | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/stickies-vni-VN-B1.png |
| stickies | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| stickies | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| stickies | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| stickies | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| stickies | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| stickies | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| stickies | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| chrome-textarea | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/chrome-textarea-vni-VN-B1.png |
| chrome-textarea | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| chrome-textarea | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| chrome-textarea | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| chrome-textarea | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| chrome-textarea | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| chrome-textarea | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| chrome-textarea | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| vscode | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/vscode-vni-VN-B1.png |
| vscode | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| vscode | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| vscode | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| vscode | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| vscode | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| vscode | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| vscode | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| terminal | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/terminal-vni-VN-B1.png |
| terminal | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| terminal | vni | VN-B3 |  | `ngu7o7i2` | `người` | `người` | PASS |  |
| terminal | vni | VN-S1 |  | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| terminal | vni | VN-S2 |  | `d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được` | `được được được` | PASS |  |
| terminal | vni | VN-E2 |  | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| terminal | vni | VN-E9 |  | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| terminal | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-telex-TX-B1.png |
| textedit-plain | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-B3 |  | `nguowif` | `người` | `người` | PASS |  |
| textedit-plain | telex | TX-B4 |  | `hoaf` | `hòa` | `hòa` | PASS |  |
| textedit-plain | telex | TX-B5 |  | `quaan` | `quân` | `quân` | PASS |  |
| textedit-plain | telex | TX-B6 |  | `tieengs` | `tiếng` | `tiếng` | PASS |  |
| textedit-plain | telex | TX-S1 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | telex | TX-S2 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | telex | TX-S3 |  | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | telex | TX-S4 |  | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | telex | TX-E1 |  | `ddeei` | `ddeei` | `ddeei` | PASS |  |
| textedit-plain | telex | TX-E2 |  | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E3a |  | `ddeei{BS}{BS}{BS}ddeem` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E3b |  | `ddeei{BS}{BS}{BS}ddeens` | `đến` | `đến` | PASS |  |
| textedit-plain | telex | TX-E4 |  | `ddeei{BS}{BS}eem` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E5 |  | `pass{BS}{BS}{BS}dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-E6 |  | `dduwowcjk{BS}` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-E7 |  | `gifk{BS}` | `gì` | `gì` | PASS |  |
| textedit-plain | telex | TX-E8 |  | `test{BS}` | `té` | `té` | PASS |  |
| textedit-plain | telex | TX-E9a |  | `1dduwowcj` | `1được` | `1được` | PASS |  |
| textedit-plain | telex | TX-E9b |  | `3giowf` | `3giờ` | `3giờ` | PASS |  |
| textedit-plain | telex | TX-E10 |  | `Tha {BS}s` | `Thá` | `Thá` | PASS |  |
| textedit-plain | telex | TX-E11 |  | `ddeei {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E12 |  | `tieengs{BS}{BS}{BS}` | `ti` | `ti` | PASS |  |
| textedit-plain | telex | TX-E13 |  | `tieengs1` | `tiếng1` | `tiếng1` | PASS |  |
| textedit-plain | telex | TX-E14 |  | `tieeng{BS}{BS}{BS}eeng` | `tiêng` | `tiêng` | PASS |  |
| textedit-plain | telex | TX-C1 |  | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C2 |  | `{SHIFT}vieetj{/SHIFT}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C3 |  | `DDUWOWCJ` | `ĐƯỢC` | `ĐƯỢC` | PASS |  |
| textedit-plain | telex | TX-C4 |  | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-H1 |  | `{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs` | `tieengs tiếng` | `tieengs tiếng` | PASS |  |
| textedit-plain | telex | TX-H2 |  | `vieejt{CTRL+SHIFT+Z}` | `vieejt` | `vieejt` | PASS |  |
| textedit-plain | telex | TX-H3 |  | `vie{LEFT}{RIGHT}et` | `vieet` | `vieet` | PASS |  |
| stickies | telex | PA-1 |  | `dduwowcj` | `dduwowcj` | `dduwowcj` | PASS | shots/stickies-telex-PA-1.png |
| stickies | telex | PA-2 |  | `Vieejt Nam` | `Vieejt Nam` | `Vieejt Nam` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| chrome-omnibox | 3 | 0 | 0 |
| chrome-textarea | 16 | 0 | 0 |
| gui | 7 | 0 | 0 |
| notes | 16 | 0 | 0 |
| secure | 2 | 0 | 0 |
| stickies | 18 | 0 | 0 |
| terminal | 16 | 0 | 0 |
| textedit-plain | 95 | 0 | 0 |
| textedit-rich | 62 | 0 | 0 |
| vscode | 16 | 0 | 0 |
