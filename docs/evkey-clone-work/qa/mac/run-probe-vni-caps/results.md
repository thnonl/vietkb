| target | method | id | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|
| textedit-plain | vni | VN-C1 | `VIE6T5` | `VIỆT` | `VIE6T5` | FAIL | shots/textedit-plain-vni-VN-C1.png |
| textedit-plain | vni | VN-X1 | `VIE6T` | `VIÊT` | `VIE6T` | FAIL | shots/textedit-plain-vni-VN-X1.png |
| textedit-plain | vni | VN-X2 | `VIE6` | `VIÊ` | `VIE6` | FAIL | shots/textedit-plain-vni-VN-X2.png |
| textedit-plain | vni | VN-X3 | `VIE5` | `VIẸ` | `VIE5` | FAIL | shots/textedit-plain-vni-VN-X3.png |
| textedit-plain | vni | VN-X4 | `Vie6T` | `ViêT` | `ViêT` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| textedit-plain | 1 | 4 | 0 |
