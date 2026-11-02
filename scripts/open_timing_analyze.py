import sys, statistics
from collections import defaultdict, OrderedDict

# opens[(run, label, n)] = OrderedDict phase -> list[(at, dur)]
opens = OrderedDict()
for run, path in enumerate(sys.argv[1:], 1):
    for line in open(path, errors="replace"):
        if not line.startswith("VKBT\t"):
            continue
        _, label, n, phase, at, dur = line.rstrip("\n").split("\t")
        key = (run, label, int(n))
        d = opens.setdefault(key, OrderedDict())
        d.setdefault(phase, []).append((float(at), None if dur == "-" else float(dur)))

def fmt(xs):
    if not xs:
        return "-"
    return f"{statistics.median(xs):7.1f} / {max(xs):7.1f}"

def table(label, select, title):
    keys = [k for k in opens if k[1] == label and select(k)]
    if not keys:
        return
    print(f"\n## {title}  ({len(keys)} opens: {', '.join(f'r{k[0]}#{k[2]}' for k in keys)})")
    phases = OrderedDict()
    for k in keys:
        for p in opens[k]:
            phases.setdefault(p, None)
    print(f"{'phase':45s} {'at ms med / max':>18s}   {'dur ms med / max':>18s}")
    rows = []
    for p in phases:
        ats, durs = [], []
        for k in keys:
            v = opens[k].get(p)
            if v:
                ats.append(v[0][0])
                ds = [x[1] for x in v if x[1] is not None]
                if ds:
                    durs.append(sum(ds) if p.startswith("page:ipc:") is False else ds[0])
        rows.append((statistics.median(ats), p, ats, durs, len(ats)))
    rows.sort()
    for _, p, ats, durs, cnt in rows:
        extra = "" if cnt == len(keys) else f"  (n={cnt})"
        print(f"{p:45s} {fmt(ats):>18s}   {fmt(durs):>18s}{extra}")

for label in ["main", "about", "hotkeys"]:
    table(label, lambda k: k[2] == 1, f"{label}: first open in process (cold process for main)")
    table(label, lambda k: k[2] > 1, f"{label}: later opens")
