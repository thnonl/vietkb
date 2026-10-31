| target | method | id | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|
| textedit-plain | vni | VN-B1 | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-vni-VN-B1.png |
| textedit-plain | vni | VN-B2 | `d9u7o7c5` | `được` | `được` | PASS |  |
| textedit-plain | vni | VN-B3 | `ngu7o7i2` | `người` | `người` | PASS |  |
| textedit-plain | vni | VN-B4 | `hoa2` | `hòa` | `hòa` | PASS |  |
| textedit-plain | vni | VN-B5 | `qua6n` | `quân` | `quân` | PASS |  |
| textedit-plain | vni | VN-B6 | `tie6ng1` | `tiếng` | `tiếng` | PASS |  |
| textedit-plain | vni | VN-S1 | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | vni | VN-S2 | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | vni | VN-S3 | `Tie6ng1 Vie6t5 ra6t1 d9e5p. Tie6ng1 Vie6t5 ra6t1 d9e5p.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | `Tiếng Việt rất đẹp. Tiếng Việt rất đẹp.` | PASS |  |
| textedit-plain | vni | VN-S4 | `d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5 d9u7o7c5` | `được được được được được được được được được được` | `được được được được được được được được được được` | PASS |  |
| textedit-plain | vni | VN-E1 | `d9e6i` | `d9e6i` | `d9e6i` | PASS |  |
| textedit-plain | vni | VN-E2 | `d9e6i{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E3a | `d9e6i{BS}{BS}{BS}d9e6m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E3b | `d9e6i{BS}{BS}{BS}d9e6n1` | `đến` | `đến` | PASS |  |
| textedit-plain | vni | VN-E4 | `d9e6i{BS}{BS}e6m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E8 | `d9e6i{BS}` | `đê` | `đê` | PASS |  |
| textedit-plain | vni | VN-E9 | `2d9e6m` | `2đêm` | `2đêm` | PASS |  |
| textedit-plain | vni | VN-E10 | `Tha {BS}1` | `Thá` | `Thá` | PASS |  |
| textedit-plain | vni | VN-E11 | `d9e6i {BS}{BS}m` | `đêm` | `đêm` | PASS |  |
| textedit-plain | vni | VN-E12 | `nghie6ng2{BS}` | `nghiền` | `nghiền` | PASS |  |
| textedit-plain | vni | VN-C1 | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-X1 | `VIE6T` | `VIÊT` | `VIÊT` | PASS |  |
| textedit-plain | vni | VN-X2 | `VIE6` | `VIÊ` | `VIÊ` | PASS |  |
| textedit-plain | vni | VN-X3 | `VIE5` | `VIẸ` | `VỊE` | FAIL | shots/textedit-plain-vni-VN-X3.png |
| textedit-plain | vni | VN-X4 | `Vie6T` | `ViêT` | `ViêT` | PASS |  |
| textedit-plain | vni | VN-C4 | `{CAPS}vie6t5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-H1 | `{CTRL+SHIFT}tie6ng1 {CTRL+SHIFT}tie6ng1` | `tie6ng1 tiếng` | `tie6ng1 tiếng` | PASS |  |
| textedit-plain | vni | VN-H2 | `vie6t5{CTRL+SHIFT+Z}` | `vie6t5` | `vie6t5` | PASS |  |
| textedit-plain | vni | VN-H3 | `vie{LEFT}{RIGHT}6t5` | `vie6t5` | `vie6t5` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| textedit-plain | 28 | 1 | 0 |
