| target | method | id | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|
| gui | - | GUI-1 | `settings window opens` | `present` | `PASS` | PASS | shots/gui-settings-window.png |
| gui | - | GUI-2 | `Accessibility banner NOT shown (permission granted)` | `present` | `PASS` | PASS | gui-settings-ax.txt + shots/gui-settings-window.png |
| gui | - | GUI-3 | `menu-bar status item present` | `present` | `PASS` | PASS | shots/gui-statusitem-on.png |
| gui | - | GUI-4 | `menu-bar menu opens` | `present` | `PASS` | PASS | shots/gui-statusitem-menu.png |
| secure | telex | SEC-1 | `dduwowcj Vieejt` | `dduwowcj Vieejt` | `dduwowcj Vieejt` | PASS | shots/secure-dialog-telex.png |
| gui | - | GUI-5 | `{CTRL+SHIFT} x2 with 'tieengs' typed while off` | `icon image changes V -> E -> V; text raw while off` | `icon differs: true, restored: true; text while off: tieengs` | PASS | shots/gui-statusitem-V-before-toggle.png / shots/gui-statusitem-E-after-toggle.png / shots/gui-statusitem-V-after-second-toggle.png |
| textedit-plain | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-telex-TX-B1.png |
| textedit-plain | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| textedit-plain | telex | TX-B4 | `hoaf` | `hòa` | `hòa` | PASS |  |
| textedit-plain | telex | TX-B5 | `quaan` | `quân` | `quân` | PASS |  |
| textedit-plain | telex | TX-B6 | `tieengs` | `tiếng` | `tiếng` | PASS |  |
| textedit-plain | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/textedit-plain-telex-TX-S1.png |
| textedit-plain | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | telex | TX-S3 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/textedit-plain-telex-TX-S3.png |
| textedit-plain | telex | TX-S4 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | telex | TX-E1 | `ddeei` | `ddeei` | `ddeei` | PASS |  |
| textedit-plain | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E3a | `ddeei{BS}{BS}{BS}ddeem` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E3b | `ddeei{BS}{BS}{BS}ddeens` | `đến` | `đến` | PASS |  |
| textedit-plain | telex | TX-E4 | `ddeei{BS}{BS}eem` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E5 | `pass{BS}{BS}{BS}dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-E6 | `dduwowcjk{BS}` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-E7 | `gifk{BS}` | `gì` | `gì` | PASS |  |
| textedit-plain | telex | TX-E8 | `test{BS}` | `té` | `té` | PASS |  |
| textedit-plain | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| textedit-plain | telex | TX-E9b | `3giowf` | `3giờ` | `3giờ` | PASS |  |
| textedit-plain | telex | TX-E10 | `Tha {BS}s` | `Thá` | `Thá` | PASS |  |
| textedit-plain | telex | TX-E11 | `ddeei {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | telex | TX-E12 | `tieengs{BS}{BS}{BS}` | `ti` | `ti` | PASS |  |
| textedit-plain | telex | TX-E13 | `tieengs1` | `tiếng1` | `tiếng1` | PASS |  |
| textedit-plain | telex | TX-E14 | `tieeng{BS}{BS}{BS}eeng` | `tiêng` | `tiêng` | PASS |  |
| textedit-plain | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C2 | `{SHIFT}vieetj{/SHIFT}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C3 | `DDUWOWCJ` | `ĐƯỢC` | `ĐƯỢC` | PASS |  |
| textedit-plain | telex | TX-C4 | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-H1 | `{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs` | `tieengs tiếng` | `tieengs tiếng` | PASS |  |
| textedit-plain | telex | TX-H2 | `vieejt{CTRL+SHIFT+Z}` | `vieejt` | `vieejt` | PASS |  |
| textedit-plain | telex | TX-H3 | `vie{LEFT}{RIGHT}et` | `vieet` | `vieet` | PASS |  |
| textedit-rich | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-rich-telex-TX-B1.png |
| textedit-rich | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| textedit-rich | telex | TX-B4 | `hoaf` | `hòa` | `hòa` | PASS |  |
| textedit-rich | telex | TX-B5 | `quaan` | `quân` | `quân` | PASS |  |
| textedit-rich | telex | TX-B6 | `tieengs` | `tiếng` | `tiếng` | PASS |  |
| textedit-rich | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/textedit-rich-telex-TX-S1.png |
| textedit-rich | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-rich | telex | TX-S3 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/textedit-rich-telex-TX-S3.png |
| textedit-rich | telex | TX-S4 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-rich | telex | TX-E1 | `ddeei` | `ddeei` | `ddeei` | PASS |  |
| textedit-rich | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E3a | `ddeei{BS}{BS}{BS}ddeem` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E3b | `ddeei{BS}{BS}{BS}ddeens` | `đến` | `đến` | PASS |  |
| textedit-rich | telex | TX-E4 | `ddeei{BS}{BS}eem` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E5 | `pass{BS}{BS}{BS}dduwowcj` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-E6 | `dduwowcjk{BS}` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-E7 | `gifk{BS}` | `gì` | `gì` | PASS |  |
| textedit-rich | telex | TX-E8 | `test{BS}` | `té` | `té` | PASS |  |
| textedit-rich | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| textedit-rich | telex | TX-E9b | `3giowf` | `3giờ` | `3giờ` | PASS |  |
| textedit-rich | telex | TX-E10 | `Tha {BS}s` | `Thá` | `Thá` | PASS |  |
| textedit-rich | telex | TX-E11 | `ddeei {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-rich | telex | TX-E12 | `tieengs{BS}{BS}{BS}` | `ti` | `ti` | PASS |  |
| textedit-rich | telex | TX-E13 | `tieengs1` | `tiếng1` | `tiếng1` | PASS |  |
| textedit-rich | telex | TX-E14 | `tieeng{BS}{BS}{BS}eeng` | `tiêng` | `tiêng` | PASS |  |
| textedit-rich | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-C2 | `{SHIFT}vieetj{/SHIFT}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-C3 | `DDUWOWCJ` | `ĐƯỢC` | `ĐƯỢC` | PASS |  |
| textedit-rich | telex | TX-C4 | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-H1 | `{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs` | `tieengs tiếng` | `tieengs tiếng` | PASS |  |
| textedit-rich | telex | TX-H2 | `vieejt{CTRL+SHIFT+Z}` | `vieejt` | `vieejt` | PASS |  |
| textedit-rich | telex | TX-H3 | `vie{LEFT}{RIGHT}et` | `vieet` | `vieet` | PASS |  |
| notes | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/notes-telex-TX-B1.png |
| notes | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| notes | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| notes | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/notes-telex-TX-S1.png |
| notes | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| notes | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| notes | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| notes | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| stickies | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/stickies-telex-TX-B1.png |
| stickies | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| stickies | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| stickies | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/stickies-telex-TX-S1.png |
| stickies | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| stickies | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| stickies | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| stickies | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| chrome-textarea | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/chrome-textarea-telex-TX-B1.png |
| chrome-textarea | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| chrome-textarea | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| chrome-textarea | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/chrome-textarea-telex-TX-S1.png |
| chrome-textarea | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| chrome-textarea | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| chrome-textarea | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| chrome-textarea | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| chrome-omnibox | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/chrome-omnibox-telex-TX-B1.png |
| chrome-omnibox | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| chrome-omnibox | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| vscode | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/vscode-telex-TX-B1.png |
| vscode | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| vscode | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| vscode | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/vscode-telex-TX-S1.png |
| vscode | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj dduwowcj` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| vscode | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| vscode | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| vscode | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |
| terminal | telex | TX-B1 | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/terminal-telex-TX-B1.png |
| terminal | telex | TX-B2 | `dduwowcj` | `được` | `được` | PASS |  |
| terminal | telex | TX-B3 | `nguowif` | `người` | `người` | PASS |  |
| terminal | telex | TX-S1 | `Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj. Tieesng Vieetj raast ddepj.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS | shots/terminal-telex-TX-S1.png |
| terminal | telex | TX-S2 | `dduwowcj dduwowcj dduwowcj` | `được được được` | `được được được` | PASS |  |
| terminal | telex | TX-E2 | `ddeei{BS}m` | `đêm` | `đêm` | PASS |  |
| terminal | telex | TX-E9a | `1dduwowcj` | `1được` | `1được` | PASS |  |
| terminal | telex | TX-C1 | `VIEETJ` | `VIỆT` | `VIỆT` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| chrome-omnibox | 3 | 0 | 0 |
| chrome-textarea | 8 | 0 | 0 |
| gui | 5 | 0 | 0 |
| notes | 8 | 0 | 0 |
| secure | 1 | 0 | 0 |
| stickies | 8 | 0 | 0 |
| terminal | 8 | 0 | 0 |
| textedit-plain | 33 | 0 | 0 |
| textedit-rich | 33 | 0 | 0 |
| vscode | 8 | 0 | 0 |
