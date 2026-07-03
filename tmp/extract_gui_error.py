from pathlib import Path

data = Path('target/gui-json.txt').read_text(encoding='utf-16', errors='replace')
for line in data.splitlines()[-160:]:
    lower = line.lower()
    if any(token in lower for token in ['error', 'failed', 'stderr', 'panic', 'tauri']):
        print(line[:1600])
