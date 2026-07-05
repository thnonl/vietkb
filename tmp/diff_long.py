import pathlib, re
s = pathlib.Path('target/long-fail.txt').read_text(encoding='utf-8', errors='replace')
l = re.search(r'left: "(.*)"', s).group(1)
r = re.search(r'right: "(.*)"', s).group(1)
for i, (a, b) in enumerate(zip(l, r)):
    if a != b:
        print(i)
        print(l[max(0, i - 80):i + 120])
        print('---')
        print(r[max(0, i - 80):i + 120])
        break
print(len(l), len(r))
