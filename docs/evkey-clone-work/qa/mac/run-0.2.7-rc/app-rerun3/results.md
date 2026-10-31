| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|---|
| app | - | APP-70 | MAC-UPD-01 | `update badge in the settings window (fake update feed)` | `see item` | `labels: ["Update available — open About"]` | PASS | shots/theme-toggle-5-update-button.png |
| app | - | APP-DOCK-update | MAC-DCK-03 | `after phase update: no new Dock tile and no running instance of the dev bundle remain (guardrail)` | `see item` | `clean (tiles 0, baseline 0)` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| app | 2 | 0 | 0 |
