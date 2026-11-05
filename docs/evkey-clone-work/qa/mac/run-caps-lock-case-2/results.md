| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|---|
| textedit-plain | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-telex-TX-B1.png |
| textedit-plain | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-plain | telex | TX-C4 |  | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | telex | TX-C5 |  | `{CAPS}VIEETJ{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-B1 |  | `Vieejt Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-rich-telex-TX-B1.png |
| textedit-rich | telex | TX-B2 |  | `dduwowcj` | `được` | `được` | PASS |  |
| textedit-rich | telex | TX-C4 |  | `{CAPS}vieejt{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | telex | TX-C5 |  | `{CAPS}VIEETJ{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-plain-vni-VN-B1.png |
| textedit-plain | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| textedit-plain | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-C4 |  | `{CAPS}vie6t5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-plain | vni | VN-C5 |  | `{CAPS}VIE6T5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | vni | VN-B1 |  | `Vie65t Nam` | `Việt Nam` | `Việt Nam` | PASS | shots/textedit-rich-vni-VN-B1.png |
| textedit-rich | vni | VN-B2 |  | `d9u7o7c5` | `được` | `được` | PASS |  |
| textedit-rich | vni | VN-C1 |  | `VIE6T5` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | vni | VN-C4 |  | `{CAPS}vie6t5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |
| textedit-rich | vni | VN-C5 |  | `{CAPS}VIE6T5{CAPS}` | `VIỆT` | `VIỆT` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| textedit-plain | 9 | 0 | 0 |
| textedit-rich | 9 | 0 | 0 |
