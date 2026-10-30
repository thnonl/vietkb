### notepad

| id | keystrokes | expected | actual | read via | verdict |
|---|---|---|---|---|---|
| A1-bat-slash-tat | ba6t5/ta8t1 | bật/tắt | bật/ttt | TextPattern/ControlType.Document | FAIL |
| A2-bat-dot-tat | ba6t5.ta8t1 | bật.tắt | bật.ttt | TextPattern/ControlType.Document | FAIL |
| A3-bat-question-tat | ba6t5?ta8t1 | bật?tắt | bậttt | TextPattern/ControlType.Document | FAIL |
| A4-bat-underscore-tat | ba6t5_ta8t1 | bật_tắt | bật_tắt | TextPattern/ControlType.Document | PASS |
| A5-windows-path | C:\Users\nam | C:\Users\nam | C:\Users\nam | TextPattern/ControlType.Document | PASS |
| A6-bat-space-tat | ba6t5 ta8t1 | bật tắt | bật tắt | TextPattern/ControlType.Document | PASS |
| A7-nguyen-thu-ha | Nguye6n4 Thu Ha2 | Nguyễn Thu Hà | Nguyễn Thu Hà | TextPattern/ControlType.Document | PASS |
| A8-url-tai | https://vietkb.dev/ta3i | https://vietkb.dev/tải | https://vietkb.dev/ta3i | TextPattern/ControlType.Document | FAIL |
| A9-a-slash-b | a/b | a/b | a/b | TextPattern/ControlType.Document | PASS |
| A10-2-slash-3 | 2/3 | 2/3 | 2/3 | TextPattern/ControlType.Document | PASS |
| A11-percent | 100%/50% | 100%/50% | 100%/50% | TextPattern/ControlType.Document | PASS |
| B1-bs-tone | ba6t5 | bậ | bậ | TextPattern/ControlType.Document | PASS |
| B2-bs-horn | ba6t | bâ | bâ | TextPattern/ControlType.Document | PASS |
| C1-restore-word | ba6t5 Ctrl+Shift+Z | ba6t5 |  | TextPattern/ControlType.Document | FAIL |
| C2-macro-m01 | m01  | mục 1  | mục    | TextPattern/ControlType.Document | FAIL |
| D1-enter | ba6t5 Enter ba6t5 | bật\nbật | bật\nttt | TextPattern/ControlType.Document | FAIL |
| D2-tab | ba6t5 Tab ba6t5 | bật\tbật | bật\tttt | TextPattern/ControlType.Document | FAIL |
| E1-off-f2 | ba6t5 after F2 | ba6t5 | ba6t5 | TextPattern/ControlType.Document | PASS |
| E2-on-f1 | ba6t5 after F1 | bật | bật | TextPattern/ControlType.Document | PASS |

### chrome

| id | keystrokes | expected | actual | read via | verdict |
|---|---|---|---|---|---|
| A1-bat-slash-tat | ba6t5/ta8t1 | bật/tắt | bật/tắt | TextPattern/ControlType.Document | PASS |
| A2-bat-dot-tat | ba6t5.ta8t1 | bật.tắt | bật.tắt | TextPattern/ControlType.Document | PASS |
| A3-bat-question-tat | ba6t5?ta8t1 | bật?tắt | bật?tắt | TextPattern/ControlType.Document | PASS |
| A4-bat-underscore-tat | ba6t5_ta8t1 | bật_tắt | bật_tắt | TextPattern/ControlType.Document | PASS |
| A5-windows-path | C:\Users\nam | C:\Users\nam | C:\Users\nam | TextPattern/ControlType.Document | PASS |
| A6-bat-space-tat | ba6t5 ta8t1 | bật tắt | bật tắt | TextPattern/ControlType.Document | PASS |
| A7-nguyen-thu-ha | Nguye6n4 Thu Ha2 | Nguyễn Thu Hà | Nguyễn Thu Hà | TextPattern/ControlType.Document | PASS |
| A8-url-tai | https://vietkb.dev/ta3i | https://vietkb.dev/tải | https://vietkb.dev/ta3i | TextPattern/ControlType.Document | FAIL |
| A9-a-slash-b | a/b | a/b | a/b | TextPattern/ControlType.Document | PASS |
| A10-2-slash-3 | 2/3 | 2/3 | 2/3 | TextPattern/ControlType.Document | PASS |
| A11-percent | 100%/50% | 100%/50% | 100%/50% | TextPattern/ControlType.Document | PASS |
| B1-bs-tone | ba6t5 | bậ | bậ | TextPattern/ControlType.Document | PASS |
| B2-bs-horn | ba6t | bâ | bâ | TextPattern/ControlType.Document | PASS |
| C1-restore-word | ba6t5 Ctrl+Shift+Z | ba6t5 | bật | TextPattern/ControlType.Document | FAIL |
| C2-macro-m01 | m01  | mục 1  | mục 1  | TextPattern/ControlType.Document | PASS |
| D1-enter | ba6t5 Enter ba6t5 | bật\nbật | bật\nbật | TextPattern/ControlType.Document | PASS |
| D2-tab | ba6t5 Tab ba6t5 | bậtbật | bậtbật | TextPattern/ControlType.Document | PASS |
| E1-off-f2 | ba6t5 after F2 | ba6t5 | ba6t5 | TextPattern/ControlType.Document | PASS |
| E2-on-f1 | ba6t5 after F1 | bật | bật | TextPattern/ControlType.Document | PASS |

### vscode

| id | keystrokes | expected | actual | read via | verdict |
|---|---|---|---|---|---|
| A1-bat-slash-tat | ba6t5/ta8t1 | bật/tắt | bật/tắt | saved-file | PASS |
| A2-bat-dot-tat | ba6t5.ta8t1 | bật.tắt | bật.tắt | saved-file | PASS |
| A3-bat-question-tat | ba6t5?ta8t1 | bật?tắt | bật?tắt | saved-file | PASS |
| A4-bat-underscore-tat | ba6t5_ta8t1 | bật_tắt | bật_tắt | saved-file | PASS |
| A5-windows-path | C:\Users\nam | C:\Users\nam | C:\Users\nam | saved-file | PASS |
| A6-bat-space-tat | ba6t5 ta8t1 | bật tắt | bật tắt | saved-file | PASS |
| A7-nguyen-thu-ha | Nguye6n4 Thu Ha2 | Nguyễn Thu Hà | Nguyễn Thu Hà | saved-file | PASS |
| A8-url-tai | https://vietkb.dev/ta3i | https://vietkb.dev/tải | https://vietkb.dev/ta3i | saved-file | FAIL |
| A9-a-slash-b | a/b | a/b | a/b | saved-file | PASS |
| A10-2-slash-3 | 2/3 | 2/3 | 2/3 | saved-file | PASS |
| A11-percent | 100%/50% | 100%/50% | 100%/50% | saved-file | PASS |
| B1-bs-tone | ba6t5 | bậ | bậ | saved-file | PASS |
| B2-bs-horn | ba6t | bâ | bâ | saved-file | PASS |
| C1-restore-word | ba6t5 Ctrl+Shift+Z | ba6t5 | bật | saved-file | FAIL |
| C2-macro-m01 | m01  | mục 1  | mục 1  | saved-file | PASS |
| D1-enter | ba6t5 Enter ba6t5 | bật\nbật | bật\nbật | saved-file | PASS |
| D2-tab | ba6t5 Tab ba6t5 | bật\tbật | bật bật | saved-file | FAIL |
| E1-off-f2 | ba6t5 after F2 | ba6t5 | ba6t5 | saved-file | PASS |
| E2-on-f1 | ba6t5 after F1 | bật | ba6t5 | saved-file | FAIL |


