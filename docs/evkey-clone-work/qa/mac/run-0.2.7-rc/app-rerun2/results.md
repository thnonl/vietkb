| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|---|
| app | - | APP-70 | MAC-UPD-01 | `update badge in the settings window (fake update feed)` | `see item` | `SKIP: the app exposes no update indicator in the settings window yet (labels read: 33); the menu-bar icon badge is a template image and is not AX-readable` | SKIP |  |
| app | - | APP-DOCK-update | MAC-DCK-03 | `after phase update: no new Dock tile and no running instance of the dev bundle remain (guardrail)` | `see item` | `clean (tiles 1, baseline 1)` | PASS |  |

| target | PASS | FAIL | other |
|---|---|---|---|
| app | 1 | 0 | 1 |
