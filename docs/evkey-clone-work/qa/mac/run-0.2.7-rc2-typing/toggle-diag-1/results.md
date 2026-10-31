| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|---|
| gui | - | GUI-5 |  | `{CTRL+SHIFT} x2 with 'tieengs' typed while off` | `icon image changes V -> E -> V; text raw while off` | `icon differs: true, restored: true; text while off: tieengs` | PASS | shots/gui-statusitem-V-before-toggle.png / shots/gui-statusitem-E-after-toggle.png / shots/gui-statusitem-V-after-second-toggle.png |

| target | PASS | FAIL | other |
|---|---|---|---|
| gui | 1 | 0 | 0 |
