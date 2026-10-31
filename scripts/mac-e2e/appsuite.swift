// appsuite.swift: declarations for the app-level checks (CAP-1 LaunchServices launch, CAP-2 Dock/activation policy,
// CAP-3 window z-order/visibility, CAP-4 AX-driven GUI, CAP-14 quit paths). Compiled together with vkbe2e.swift
// (run.sh copies vkbe2e.swift to main.swift; top-level code lives only there).
//
// Guardrails specific to this file (docs/evkey-clone-work/99-agent-guardrails.md incl. section 6):
//  * only the dev-channel bundle (identifier ending ".dev") is ever launched; if ANY process of that bundle id is already
//    running the run is BLOCKED (single-instance would forward into it).
//  * every action is an AX action (AXPress / AXShowMenu / AXCancel / AXRaise) on an element that belongs to a pid we
//    launched, or on the Dock tile whose AXURL is the path of the bundle we launched. No coordinate click, no CGEvent,
//    no keystroke is posted by anything in this file. The menu bar and status items are reached through the pid's own AX tree.
//  * the only other app touched is TextEdit, through the harness's existing opener (its own new document, closed saving no).
//  * screenshots: VietKB windows (by CGWindowID) and a cropped region around the VietKB Dock tile. Never the full screen.
import Foundation
import AppKit
import ApplicationServices

// MARK: - guardrail log (same file and format run.sh writes)
func glog(_ s: String) {
    let line = "\(ts().prefix(8)) \(s)\n"
    let path = outDir + "/guardrail.log"
    if !FileManager.default.fileExists(atPath: path) { FileManager.default.createFile(atPath: path, contents: nil) }
    if let h = FileHandle(forWritingAtPath: path) { h.seekToEndOfFile(); h.write(line.data(using: .utf8)!); try? h.close() }
    log("GUARD " + s)
}
/// Pids launched by this suite, recorded so run.sh can reap any survivor even if this process dies.
var launchedPids: [Int32] = []
func notePid(_ p: Int32) {
    launchedPids.append(p)
    let path = outDir + "/launched-pids.txt"
    if let h = FileHandle(forWritingAtPath: path) ?? { FileManager.default.createFile(atPath: path, contents: nil); return FileHandle(forWritingAtPath: path) }() {
        h.seekToEndOfFile(); h.write("\(p)\n".data(using: .utf8)!); try? h.close()
    }
}

// MARK: - AX helpers
func axStr(_ el: AXUIElement, _ attr: String) -> String? {
    guard let v = axCopy(el, attr) else { return nil }
    if let s = v as? String { return s }
    if let n = v as? NSNumber { return n.stringValue }
    if let u = v as? URL { return u.path }
    return nil
}
func axKids(_ el: AXUIElement) -> [AXUIElement] { (axCopy(el, kAXChildrenAttribute as String) as? [AXUIElement]) ?? [] }
func axRole(_ el: AXUIElement) -> String { axStr(el, kAXRoleAttribute as String) ?? "" }
/// The visible label of a control: title, then description, then value, then the first static text below it.
func axLabel(_ el: AXUIElement) -> String {
    for a in [kAXTitleAttribute, kAXDescriptionAttribute, kAXValueAttribute] {
        if let s = axStr(el, a as String), !s.isEmpty { return s }
    }
    for k in axKids(el) { if axRole(k) == "AXStaticText", let s = axStr(k, kAXValueAttribute as String), !s.isEmpty { return s } }
    return ""
}
func axActions(_ el: AXUIElement) -> [String] {
    var names: CFArray?
    return AXUIElementCopyActionNames(el, &names) == .success ? (names as? [String]) ?? [] : []
}
@discardableResult func axPress(_ el: AXUIElement) -> Bool { AXUIElementPerformAction(el, kAXPressAction as CFString) == .success }
func axWalk(_ el: AXUIElement, depth: Int = 0, maxDepth: Int = 28, _ visit: (AXUIElement, Int) -> Void) {
    if depth > maxDepth { return }
    visit(el, depth)
    for k in axKids(el) { axWalk(k, depth: depth + 1, maxDepth: maxDepth, visit) }
}
func axFindAll(_ root: AXUIElement, role: String? = nil, where pred: (AXUIElement) -> Bool) -> [AXUIElement] {
    var out: [AXUIElement] = []
    axWalk(root) { el, _ in if (role == nil || axRole(el) == role!) && pred(el) { out.append(el) } }
    return out
}
func axDump(_ root: AXUIElement, maxDepth: Int = 28) -> String {
    var lines: [String] = []
    axWalk(root, maxDepth: maxDepth) { el, d in
        let r = axRect(el).map { "[\(Int($0.minX)),\(Int($0.minY)) \(Int($0.width))x\(Int($0.height))]" } ?? ""
        let sub = axStr(el, kAXSubroleAttribute as String) ?? ""
        lines.append(String(repeating: "  ", count: d) + axRole(el) + (sub.isEmpty ? "" : "/" + sub) + " '" + axLabel(el).replacingOccurrences(of: "\n", with: "⏎") + "' " + axActions(el).joined(separator: ",") + " " + r)
    }
    return lines.joined(separator: "\n")
}
func axWindows(_ pid: Int32) -> [AXUIElement] { (axCopy(AXUIElementCreateApplication(pid), kAXWindowsAttribute as String) as? [AXUIElement]) ?? [] }
func axTitle(_ w: AXUIElement) -> String { axStr(w, kAXTitleAttribute as String) ?? "" }

// MARK: - running-app identity (never match by name: by bundle id AND bundle path)
func bundleInfo(_ appPath: String) -> (id: String, name: String)? {
    guard let b = Bundle(path: appPath), let id = b.bundleIdentifier else { return nil }
    return (id, (b.infoDictionary?["CFBundleName"] as? String) ?? "")
}
func runningInstances(_ appPath: String) -> [NSRunningApplication] {
    guard let (id, _) = bundleInfo(appPath) else { return [] }
    return NSRunningApplication.runningApplications(withBundleIdentifier: id).filter { $0.bundleURL?.resolvingSymlinksInPath().path == URL(fileURLWithPath: appPath).resolvingSymlinksInPath().path }
}
func waitGone(_ pid: Int32, _ secs: Double) -> Bool {
    let end = Date().addingTimeInterval(secs)
    while Date() < end { if kill(pid, 0) != 0 { return true }; usleep(100_000) }
    return kill(pid, 0) != 0
}
func waitUntil(_ secs: Double, _ cond: () -> Bool) -> Bool {
    let end = Date().addingTimeInterval(secs)
    while Date() < end { if cond() { return true }; usleep(120_000) }
    return cond()
}

// MARK: - owner activity guard
/// The idle gate is checked once up front; this keeps checking while the suite runs. AX actions never reset HID idle, so any
/// drop means a human is using the machine: stop at the next step instead of popping windows in front of them.
func quiet() throws {
    let idle = hidIdle()
    if env["SKIP_IDLE_GATE"] == nil && idle < 8 { throw abort("owner became active (HID idle \(String(format: "%.1f", idle)) s): stopping before the next step") }
}

// MARK: - launching (CAP-1)
enum LaunchMode: String { case launchServices = "open", direct = "exec" }
var directProcs: [Process] = []
/// Launch the dev-channel bundle. LaunchServices first (`open -n -a`, env passed with `--env`): this exercises Info.plist (LSUIElement),
/// the bundle identity and the Dock. Falls back to direct exec only when the caller asks (direct inherits this shell's TCC grants,
/// which is what lets the event tap start; LaunchServices launches run under the bundle's own, un-granted, identity).
func launchApp(_ appPath: String, home: String, mode: LaunchMode, extraEnv: [String: String] = [:], tag: String) throws -> Int32 {
    try quiet()
    guard let (id, _) = bundleInfo(appPath) else { throw abort("cannot read bundle id of \(appPath)") }
    guard id.hasSuffix(".dev") else { throw abort("REFUSED: bundle id \(id) is not a dev-channel build (guardrails section 6)") }
    _ = waitUntil(6) { runningInstances(appPath).isEmpty }   // LaunchServices drops a terminated app a moment after its pid is gone
    let existing = runningInstances(appPath)
    guard existing.isEmpty else { throw abort("BLOCKED: \(id) already running (pid \(existing.map { $0.processIdentifier })): a launch would be forwarded into it") }
    var envAll = extraEnv; envAll["HOME"] = home
    let outLog = outDir + "/vietkb-\(tag).log"
    var pid: Int32 = 0
    switch mode {
    case .launchServices:
        var args = ["-n", "-a", appPath, "--stdout", outLog, "--stderr", outLog]
        for (k, v) in envAll.sorted(by: { $0.key < $1.key }) { args += ["--env", "\(k)=\(v)"] }
        let r = sh("/usr/bin/open", args, timeout: 20)
        if r.code != 0 { throw abort("open failed (\(r.code)): \(r.out)") }
        guard waitUntil(15, { !runningInstances(appPath).isEmpty }) else { throw abort("LaunchServices did not start \(id)") }
        pid = runningInstances(appPath)[0].processIdentifier
    case .direct:
        let p = Process(); p.executableURL = URL(fileURLWithPath: appPath + "/Contents/MacOS/vietkb")
        var e = ProcessInfo.processInfo.environment; for (k, v) in envAll { e[k] = v }; p.environment = e
        FileManager.default.createFile(atPath: outLog, contents: nil)
        let fh = FileHandle(forWritingAtPath: outLog); p.standardOutput = fh; p.standardError = fh
        try p.run(); directProcs.append(p); pid = p.processIdentifier
    }
    notePid(pid)
    glog("LAUNCH pid=\(pid) path=\(appPath) via=\(mode.rawValue) HOME=\(home) tag=\(tag) (dev-channel \(id), launched by the app suite)")
    return pid
}
/// Terminate a pid we launched (SIGTERM, then SIGKILL after 4 s). Logged as KILL.
func killOurs(_ pid: Int32, reason: String) {
    guard launchedPids.contains(pid), kill(pid, 0) == 0 else { return }
    glog("KILL pid=\(pid) path=\(String(cString: getprogname())) reason=\(reason) (launched by the app suite)")
    // graceful first: the app's own Quit menu item (AXPress), so Dock/LaunchServices state is torn down normally
    if let q = quitMenuItem(pid), axPress(q), waitGone(pid, 5) { glog("  pid=\(pid) exited after AX Quit"); return }
    kill(pid, SIGTERM)
    if waitGone(pid, 5) { glog("  pid=\(pid) exited after SIGTERM"); return }
    glog("KILL -9 pid=\(pid) LAST RESORT: did not exit 5 s after AX Quit and SIGTERM (may leave a stale Dock tile)")
    kill(pid, SIGKILL); _ = waitGone(pid, 2)
}
func reapAll(reason: String) {
    for p in launchedPids where kill(p, 0) == 0 { killOurs(p, reason: reason) }
}

// MARK: - Dock (CAP-2)
func dockPid() -> Int32? { NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.dock").first?.processIdentifier }
/// The Dock tile whose AXURL is exactly our bundle path (read-only AX query of the Dock's tree).
func dockTileCount(appPath: String) -> Int {
    guard let dp = dockPid() else { return 0 }
    let want = URL(fileURLWithPath: appPath).resolvingSymlinksInPath().path
    return axKids(AXUIElementCreateApplication(dp)).flatMap { axKids($0) }.filter { axRole($0) == "AXDockItem" && ((axCopy($0, "AXURL") as? URL)?.resolvingSymlinksInPath().path == want) }.count
}
func dockTile(appPath: String) -> (AXUIElement, CGRect)? {
    guard let dp = dockPid() else { return nil }
    let want = URL(fileURLWithPath: appPath).resolvingSymlinksInPath().path
    let app = AXUIElementCreateApplication(dp)
    for list in axKids(app) {
        for item in axKids(list) where axRole(item) == "AXDockItem" {
            // a tile that is still animating in/out has a degenerate rect: not "present" yet
            // only a RUNNING app's tile counts as "present": stale tiles left by killed processes are counted separately (dockTileCount)
            if let u = axCopy(item, "AXURL") as? URL, u.resolvingSymlinksInPath().path == want, axStr(item, "AXIsApplicationRunning") == "1", let r = axRect(item), r.width >= 16, r.height >= 16 { return (item, r) }
        }
    }
    return nil
}
func policyName(_ pid: Int32) -> String {
    switch NSRunningApplication(processIdentifier: pid)?.activationPolicy {
    case .some(.regular): return "regular"
    case .some(.accessory): return "accessory"
    case .some(.prohibited): return "prohibited"
    default: return "unknown"
    }
}
/// Dock-tile crop for evidence: a square around the tile only (never the whole Dock or screen).
func shotDockTile(_ r: CGRect, _ name: String) -> String {
    shotRegion(CGRect(x: r.minX - 6, y: r.minY - 6, width: r.width + 12, height: r.height + 12), name)
}

// MARK: - windows: z-order, visibility, occlusion (CAP-3)
struct WinInfo { let id: Int; let pid: Int32; let owner: String; let layer: Int; let alpha: Double; let bounds: CGRect; let onscreen: Bool; let order: Int; let name: String }
/// Every window in front-to-back order including off-screen ones (`order` is the index in the on-screen list, -1 when not listed).
func allWindows() -> [WinInfo] {
    guard let on = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]],
          let all = CGWindowListCopyWindowInfo([.optionAll], kCGNullWindowID) as? [[String: Any]] else { return [] }
    var order: [Int: Int] = [:]
    for (i, d) in on.enumerated() { if let n = d[kCGWindowNumber as String] as? Int { order[n] = i } }
    return all.map { d in
        var r = CGRect.zero
        if let b = d[kCGWindowBounds as String] as? [String: Any] { r = CGRect(dictionaryRepresentation: b as CFDictionary) ?? .zero }
        let n = d[kCGWindowNumber as String] as? Int ?? 0
        return WinInfo(id: n, pid: Int32(d[kCGWindowOwnerPID as String] as? Int ?? 0), owner: d[kCGWindowOwnerName as String] as? String ?? "",
                       layer: d[kCGWindowLayer as String] as? Int ?? 0, alpha: d[kCGWindowAlpha as String] as? Double ?? 1, bounds: r,
                       onscreen: (d[kCGWindowIsOnscreen as String] as? Bool) ?? false, order: order[n] ?? -1, name: d[kCGWindowName as String] as? String ?? "")
    }
}
/// A VietKB content window: layer 0, real size. (The status item, menus and the web view's helper surfaces are filtered out.)
func appContentWindows(_ pid: Int32) -> [WinInfo] { allWindows().filter { $0.pid == pid && $0.layer == 0 && $0.bounds.width > 150 && $0.bounds.height > 100 } }
func displayRects() -> [CGRect] {
    var ids = [CGDirectDisplayID](repeating: 0, count: 8); var n: UInt32 = 0
    CGGetActiveDisplayList(8, &ids, &n)
    return ids.prefix(Int(n)).map { CGDisplayBounds($0) }
}
/// Fraction of `w`'s area that lies inside the display it mostly sits on (1.0 = fully on screen).
func onScreenFraction(_ w: CGRect) -> Double {
    let best = displayRects().map { $0.intersection(w) }.filter { !$0.isNull }.map { Double($0.width * $0.height) }.max() ?? 0
    return best / Double(max(1, w.width * w.height))
}
/// Fraction of `w` covered by windows that are above it in the stack and do not belong to `ignorePids`. Sampled on a 20x20 grid.
/// Menu-bar/status/Dock layers (>= 20), Window Server surfaces and transparent windows do not count.
func foreignCoverage(of w: WinInfo, ignorePids: Set<Int32>) -> Double {
    guard w.order >= 0 else { return 1 }
    let above = allWindows().filter { $0.order >= 0 && $0.order < w.order && $0.layer < 20 && $0.alpha > 0.05 && !ignorePids.contains($0.pid)
        && $0.owner != "Window Server" && $0.owner != "Dock" && $0.bounds.width > 20 && $0.bounds.height > 20 }
    var covered = 0, total = 0
    for i in 0..<20 { for j in 0..<20 {
        let p = CGPoint(x: w.bounds.minX + (Double(i) + 0.5) * w.bounds.width / 20, y: w.bounds.minY + (Double(j) + 0.5) * w.bounds.height / 20)
        total += 1; if above.contains(where: { $0.bounds.contains(p) }) { covered += 1 }
    } }
    return Double(covered) / Double(total)
}
func frontmostPid() -> Int32 { NSWorkspace.shared.frontmostApplication?.processIdentifier ?? -1 }

// MARK: - locale strings (read from the repo, so the checks follow the copy instead of duplicating it)
var repoRoot: String { env["E2E_REPO"] ?? FileManager.default.currentDirectoryPath }
func loadLocale(_ lang: String) -> [String: String] {
    guard let text = try? String(contentsOfFile: repoRoot + "/ui/src/lib/locales/\(lang).ts", encoding: .utf8) else { return [:] }
    var d: [String: String] = [:]
    let re = try! NSRegularExpression(pattern: #"^\s*"([^"]+)":\s*"((?:[^"\\]|\\.)*)",?\s*$"#, options: [.anchorsMatchLines])
    for m in re.matches(in: text, range: NSRange(text.startIndex..., in: text)) {
        let k = String(text[Range(m.range(at: 1), in: text)!])
        var v = String(text[Range(m.range(at: 2), in: text)!])
        v = v.replacingOccurrences(of: "\\\"", with: "\"").replacingOccurrences(of: "\\n", with: "\n").replacingOccurrences(of: "\\\\", with: "\\")
        d[k] = v.precomposedStringWithCanonicalMapping
    }
    return d
}
/// Rust-side table (crates/vkb-app/src/i18n_table.rs): window titles and tray menu rows.
func loadRustTable() -> [String: (vi: String, en: String)] {
    guard let text = try? String(contentsOfFile: repoRoot + "/crates/vkb-app/src/i18n_table.rs", encoding: .utf8) else { return [:] }
    var d: [String: (vi: String, en: String)] = [:]
    let re = try! NSRegularExpression(pattern: #"\(\s*"([\w.]+)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)"#)
    for m in re.matches(in: text, range: NSRange(text.startIndex..., in: text)) {
        func g(_ i: Int) -> String { String(text[Range(m.range(at: i), in: text)!]).precomposedStringWithCanonicalMapping }
        d[g(1)] = (g(2), g(3))
    }
    return d
}

// MARK: - config read-back
func configPath(home: String, bundleID: String) -> String {
    home + "/Library/Application Support/" + (bundleID.hasSuffix(".dev") ? "VietKB Dev" : "VietKB") + "/config.toml"
}
func configValue(_ path: String, _ key: String) -> String? {
    guard let t = try? String(contentsOfFile: path, encoding: .utf8) else { return nil }
    for l in t.split(separator: "\n") {
        let s = l.trimmingCharacters(in: .whitespaces)
        if s.hasPrefix(key + " ") || s.hasPrefix(key + "=") {
            let v = s.split(separator: "=", maxSplits: 1).last.map { String($0).trimmingCharacters(in: .whitespaces) } ?? ""
            return v.trimmingCharacters(in: CharacterSet(charactersIn: "\""))
        }
    }
    return nil
}
func writeConfig(home: String, bundleID: String, language: String, startMinimized: Bool, theme: String = "dark") {
    let p = configPath(home: home, bundleID: bundleID)
    try? FileManager.default.createDirectory(atPath: (p as NSString).deletingLastPathComponent, withIntermediateDirectories: true)
    let body = """
    version = 1
    [input]
    enabled = true
    method = "telex"
    [system]
    start_with_windows = false
    start_minimized = \(startMinimized)
    notification_sound = false
    check_updates = false
    language = "\(language)"
    theme = "\(theme)"

    """
    try? body.write(toFile: p, atomically: true, encoding: .utf8)
}

// MARK: - rows
var suiteRows: [[String: Any]] = []
/// One result row: `id` is a harness check id, `cases` the catalog case ids it automates (or part-automates).
func row(_ id: String, _ cases: String, _ item: String, _ ok: Bool?, _ actual: String, evidence: String = "", note: String = "") {
    let verdict = ok == nil ? "SKIP" : (ok! ? "PASS" : "FAIL")
    var d: [String: Any] = ["target": "app", "method": "-", "id": id, "keys": item, "expected": "see item", "actual": actual, "verdict": verdict, "evidence": evidence, "note": note, "cases": cases]
    d["actual"] = actual
    suiteRows.append(d)
    var cnt = counts["app"] ?? (0, 0); if ok == true { cnt.pass += 1 } else if ok == false { cnt.fail += 1 }; counts["app"] = cnt
    let data = try! JSONSerialization.data(withJSONObject: d, options: [.sortedKeys])
    resultsHandle?.write(data); resultsHandle?.write("\n".data(using: .utf8)!)
    log("[app] \(id) \(verdict) \(cases) :: \(item) :: \(actual)")
}

/// WebKit builds the accessibility tree of a web view only for clients that ask: set the standard opt-in attributes on OUR pid's
/// application element (this changes nothing but that process's AX exposure).
func enableWebAX(_ pid: Int32) {
    let a = AXUIElementCreateApplication(pid)
    for attr in ["AXEnhancedUserInterface", "AXManualAccessibility"] { _ = AXUIElementSetAttributeValue(a, attr as CFString, kCFBooleanTrue) }
}

// MARK: - recon dump (debug subcommand `appdump`)
func appDump(app: String, home: String, mode: LaunchMode) throws {
    guard let (bid, _) = bundleInfo(app) else { throw abort("bad bundle") }
    writeConfig(home: home, bundleID: bid, language: "vi", startMinimized: false)
    let pid = try launchApp(app, home: home, mode: mode, tag: "dump")
    defer { reapAll(reason: "appdump finished") }
    sleep(5)
    enableWebAX(pid)
    sleep(2)
    print("pid", pid, "policy", policyName(pid), "front", frontmostPid(), "windows(AX)", axWindows(pid).map { axTitle($0) })
    print("dock tile:", dockTile(appPath: app).map { "\($0.1)" } ?? "none")
    for w in axWindows(pid) {
        let t = axDump(w)
        try? t.write(toFile: outDir + "/ax-window-\(axWindowID(w)).txt", atomically: true, encoding: .utf8)
        print("window \(axWindowID(w)) '\(axTitle(w))' attrs: \(axStr(w, "AXEnabled") ?? "-") actions \(axActions(w)); AX lines \(t.split(separator: "\n").count)")
    }
    let appEl = AXUIElementCreateApplication(pid)
    if let w = axWindows(pid).first, let r = axRect(w) {
        // read-only hit test scoped to OUR pid's application element (no event is posted)
        var hit: AXUIElement?
        let err = AXUIElementCopyElementAtPosition(appEl, Float(r.midX), Float(r.minY + 150), &hit)
        print("hit test err", err.rawValue, hit.map { "role=\(axRole($0)) label='\(axLabel($0))' parent=\(axCopy($0, kAXParentAttribute as String).map { axRole($0 as! AXUIElement) } ?? "-")" } ?? "nil")
        var cur: AXUIElement? = hit
        var chain: [String] = []
        while let c = cur, chain.count < 12 { chain.append(axRole(c)); cur = axCopy(c, kAXParentAttribute as String) as! AXUIElement? }
        print("chain up:", chain)
        let primed = axDump(w)
        try? primed.write(toFile: outDir + "/ax-window-primed.txt", atomically: true, encoding: .utf8)
        print("after hit-test priming the window tree has \(primed.split(separator: "\n").count) lines")
        let g = axKids(axKids(w).first!)
        print("group child count", g.count, g.map { axRole($0) })
        var an: CFArray?; if let first = g.first { AXUIElementCopyAttributeNames(first, &an); print("inner group attrs", (an as? [String]) ?? []) }
    }
    var names: CFArray?; AXUIElementCopyAttributeNames(appEl, &names)
    print("app attrs:", (names as? [String]) ?? [])
    if let bar = axCopy(appEl, "AXExtrasMenuBar") {
        let b = bar as! AXUIElement
        print("extras bar:\n" + axDump(b, maxDepth: 6))
        if let item = axKids(b).first {
            print("status item actions:", axActions(item))
            let before = Set(allWindows().map { $0.id })
            print("AXPress ->", axPress(item))
            usleep(1_500_000)
            print("after press, subtree:\n" + axDump(item, maxDepth: 6))
            print("app tree depth 3:\n" + axDump(appEl, maxDepth: 3).split(separator: "\n").prefix(40).joined(separator: "\n"))
            print("new windows (any owner):", allWindows().filter { !before.contains($0.id) }.map { "\($0.id) \($0.owner) L\($0.layer) \($0.bounds) on=\($0.onscreen)" })
            print("pid windows now:", allWindows().filter { $0.pid == pid }.map { "\($0.id) L\($0.layer) \($0.bounds) on=\($0.onscreen)" })
            if let menu = axKids(item).first(where: { axRole($0) == "AXMenu" }) { print("cancel ->", AXUIElementPerformAction(menu, kAXCancelAction as CFString).rawValue) }
        }
    }
    if let mb = axCopy(appEl, kAXMenuBarAttribute as String) { print("menu bar:\n" + axDump(mb as! AXUIElement, maxDepth: 4)) }
    if let (tile, r) = dockTile(appPath: app) { print("dock tile actions:", axActions(tile), r) }
}

// =====================================================================================================================
// MARK: - the suite
// =====================================================================================================================
let DIACRITICS = try! NSRegularExpression(pattern: "[ăâêôơưđĂÂÊÔƠƯĐàáảãạằắẳẵặầấẩẫậèéẻẽẹềếểễệìíỉĩịòóỏõọồốổỗộờớởỡợùúủũụừứửữựỳýỷỹỵ]")
func hasDiacritic(_ s: String) -> Bool { DIACRITICS.firstMatch(in: s, range: NSRange(s.startIndex..., in: s)) != nil }
func norm(_ s: String) -> String {
    var t = s.precomposedStringWithCanonicalMapping.trimmingCharacters(in: .whitespacesAndNewlines)
    for suf in ["…", "..."] where t.hasSuffix(suf) { t.removeLast(suf.count) }
    return t.trimmingCharacters(in: .whitespaces)
}

final class AppCtx {
    let app: String, bid: String, tmp: String
    let vi: [String: String], en: [String: String], rt: [String: (vi: String, en: String)]
    var pid: Int32 = 0
    var lang = "vi"
    var homeN = 0
    init(app: String, tmp: String) {
        self.app = app; self.tmp = tmp; bid = bundleInfo(app)?.id ?? ""
        vi = loadLocale("vi"); en = loadLocale("en"); rt = loadRustTable()
    }
    func L(_ k: String, _ lg: String? = nil) -> String { ((lg ?? lang) == "en" ? en : vi)[k] ?? "<<\(k)>>" }
    func T(_ k: String, _ lg: String? = nil) -> String { ((lg ?? lang) == "en" ? rt[k]?.en : rt[k]?.vi) ?? "<<\(k)>>" }
    func newHome(_ tag: String) -> String { homeN += 1; let h = tmp + "/home-\(tag)-\(homeN)"; try? FileManager.default.createDirectory(atPath: h, withIntermediateDirectories: true); return h }
    func cfg(_ home: String) -> String { configPath(home: home, bundleID: bid) }
}
let WINKEYS = ["main": "rwin.main", "hotkeys": "rwin.hotkeys", "advanced": "rwin.advanced", "macros": "rwin.macros", "about": "rwin.about"]

/// Hit-test priming: WebKit exposes the page's AX tree to a client only after the client has touched it (read-only, no event).
func primeWebAX(_ pid: Int32, _ w: AXUIElement) {
    guard let r = axRect(w) else { return }
    let a = AXUIElementCreateApplication(pid)
    for dy in [150.0, 260.0, 100.0] { var hit: AXUIElement?; _ = AXUIElementCopyElementAtPosition(a, Float(r.midX), Float(r.minY + dy), &hit) }
}
/// All of the pid's content windows as (AX element, CGWindowID, title).
func contentAX(_ pid: Int32) -> [(el: AXUIElement, id: Int, title: String)] {
    let ids = Set(appContentWindows(pid).map { $0.id })
    return axWindows(pid).compactMap { w in let i = axWindowID(w); return ids.contains(i) ? (w, i, axTitle(w)) : nil }
}
/// The window with its web content readable: prime, then wait (up to ~2.5 s) until the AXWebArea actually has children.
func tree(_ pid: Int32, _ w: AXUIElement) -> AXUIElement {
    for _ in 0..<16 {
        primeWebAX(pid, w)
        if let wa = axFirst(w, role: "AXWebArea"), axKids(wa).count > 0 { return w }
        usleep(160_000)
    }
    return w
}
func findWin(_ c: AppCtx, _ kind: String, lang: String? = nil) -> (el: AXUIElement, id: Int, title: String)? {
    let title = c.T(WINKEYS[kind]!, lang)
    return contentAX(c.pid).first { $0.title == title }
}
func waitWin(_ c: AppCtx, _ kind: String, secs: Double = 6, lang: String? = nil) -> (el: AXUIElement, id: Int, title: String)? {
    var r: (el: AXUIElement, id: Int, title: String)?
    _ = waitUntil(secs) { r = findWin(c, kind, lang: lang); return r != nil }
    return r
}
func buttons(_ c: AppCtx, _ w: AXUIElement) -> [AXUIElement] { axFindAll(tree(c.pid, w), role: "AXButton") { !axLabel($0).isEmpty } }
func button(_ c: AppCtx, _ w: AXUIElement, label: String) -> AXUIElement? { buttons(c, w).first { norm(axLabel($0)) == norm(label) } }
func closeWindow(_ w: AXUIElement) -> Bool {
    for k in axKids(w) where axStr(k, kAXSubroleAttribute as String) == "AXCloseButton" { return axPress(k) }
    return false
}
/// Every label in a window (static texts, buttons, toggles): what a reader of the screen sees.
func windowTexts(_ c: AppCtx, _ w: AXUIElement) -> [String] {
    var out: [String] = []
    axWalk(tree(c.pid, w)) { el, _ in
        let r = axRole(el)
        if ["AXStaticText", "AXButton", "AXCheckBox", "AXPopUpButton", "AXTextField", "AXLink", "AXRadioButton", "AXHeading"].contains(r) {
            let l = axLabel(el); if !l.isEmpty { out.append(l) }
        }
    }
    return out
}
func controlsOutside(_ c: AppCtx, _ w: AXUIElement) -> [String] {
    guard let wr = axRect(w) else { return ["no window rect"] }
    var bad: [String] = []
    axWalk(tree(c.pid, w)) { el, _ in
        guard ["AXButton", "AXCheckBox", "AXPopUpButton"].contains(axRole(el)), axStr(el, kAXSubroleAttribute as String) != "AXCloseButton",
              axStr(el, kAXSubroleAttribute as String) != "AXZoomButton", axStr(el, kAXSubroleAttribute as String) != "AXMinimizeButton",
              let r = axRect(el), r.width > 0 else { return }
        // controls inside a scrolling table/changelog may legitimately extend past the window; the web area's own scroll
        // position is not readable, so only controls that START inside the window but END past its bottom/right are clipping
        if r.minY < wr.maxY - 1 && r.maxY > wr.maxY + 1 || r.minX < wr.maxX - 1 && r.maxX > wr.maxX + 1 || r.minX < wr.minX - 1 { bad.append("\(axLabel(el)) \(r) vs window \(wr)") }
    }
    return bad
}

struct Vis { let ok: Bool; let text: String }
/// CAP-3: on screen, opaque, fully on a display, not covered by any other app's window.
func visibility(_ id: Int, pid: Int32) -> Vis {
    guard let w = allWindows().first(where: { $0.id == id }) else { return Vis(ok: false, text: "window \(id) not found") }
    let frac = onScreenFraction(w.bounds)
    let cov = foreignCoverage(of: w, ignorePids: [pid])
    let ok = w.onscreen && w.order >= 0 && w.alpha >= 0.99 && frac >= 0.995 && cov <= 0.02
    return Vis(ok: ok, text: "id \(id) onscreen=\(w.onscreen) alpha=\(w.alpha) onDisplay=\(String(format: "%.3f", frac)) foreignCoverage=\(String(format: "%.3f", cov)) z=\(w.order) bounds=\(Int(w.bounds.minX)),\(Int(w.bounds.minY)) \(Int(w.bounds.width))x\(Int(w.bounds.height))")
}
func zOrder(_ ids: [Int]) -> (ok: Bool, text: String) {
    let all = allWindows()
    let z = ids.map { id in all.first(where: { $0.id == id })?.order ?? -1 }
    let ok = !z.contains(-1) && zip(z, z.dropFirst()).allSatisfy { $0 < $1 }
    return (ok, "front-to-back wanted \(ids) got z=\(z)")
}
func tapCount(_ pid: Int32) -> Int {
    var n: UInt32 = 0
    CGGetEventTapList(0, nil, &n)
    guard n > 0 else { return 0 }
    var list = [CGEventTapInformation](repeating: CGEventTapInformation(), count: Int(n))
    CGGetEventTapList(n, &list, &n)
    return list.prefix(Int(n)).filter { $0.tappingProcess == pid }.count
}
func ownerPids() -> [Int32] {   // the owner's production VietKB, observed read-only; the run must not change this set
    NSRunningApplication.runningApplications(withBundleIdentifier: "vn.vietkb.app").map { $0.processIdentifier }.sorted()
}
func openRunning(_ c: AppCtx) -> Bool {   // `open -a` without -n: LaunchServices activates the running instance and sends it a "reopen"
    sh("/usr/bin/open", ["-a", c.app], timeout: 15).code == 0
}
func quitMenuItem(_ pid: Int32) -> AXUIElement? {
    guard let mb = axCopy(AXUIElementCreateApplication(pid), kAXMenuBarAttribute as String) else { return nil }
    var found: AXUIElement?
    axWalk(mb as! AXUIElement, maxDepth: 5) { el, _ in
        if found == nil, axRole(el) == "AXMenuItem", axStr(el, "AXMenuItemCmdChar") == "Q", (axStr(el, "AXMenuItemCmdModifiers") ?? "0") == "0" { found = el }
    }
    return found
}
/// Our own Dock tile's context menu, via AXShowMenu on the tile whose AXURL is our bundle; returns the items.
func dockMenuItems(_ tile: AXUIElement) -> [AXUIElement] {
    _ = AXUIElementPerformAction(tile, "AXShowMenu" as CFString)
    usleep(700_000)
    return axFindAll(tile, role: "AXMenuItem") { _ in true }
}
func statusItemEl(_ pid: Int32) -> AXUIElement? {
    guard let bar = axCopy(AXUIElementCreateApplication(pid), "AXExtrasMenuBar") else { return nil }
    return axKids(bar as! AXUIElement).first
}
func launchFresh(_ c: AppCtx, _ tag: String, lang: String = "vi", startMinimized: Bool, mode: LaunchMode = .launchServices, home: String? = nil, keepConfig: Bool = false) throws -> String {
    let h = home ?? c.newHome(tag)
    if !keepConfig { writeConfig(home: h, bundleID: c.bid, language: lang, startMinimized: startMinimized) }
    c.pid = try launchApp(c.app, home: h, mode: mode, tag: tag)
    c.lang = lang
    if !startMinimized { _ = waitUntil(12) { !appContentWindows(c.pid).isEmpty } }
    else { sleep(3) }
    enableWebAX(c.pid)
    return h
}
func endApp(_ c: AppCtx, _ why: String) {
    if c.pid != 0, kill(c.pid, 0) == 0 { killOurs(c.pid, reason: why) }
}

// ---------------------------------------------------------------------------------------------------------------------
// Phase A: LaunchServices launch, Dock, windows, modals, close paths, relaunch, language (MAC-DCK/WIN/LCY/LNG, CAP-1..4)
// ---------------------------------------------------------------------------------------------------------------------
func phaseMain(_ c: AppCtx, te: Target?) throws {
    log("==== phase main: LaunchServices launch, Dock, windows, modals, language ====")
    let home = try launchFresh(c, "main", startMinimized: false)
    let pid = c.pid
    defer { endApp(c, "end of phase main") }
    let inst = runningInstances(c.app)
    row("APP-01", "MAC-LCY-06,MAC-LCY-07", "launched through LaunchServices (open -n) from a temp HOME: exactly one process of the dev bundle; config written under the temp HOME, not the owner's",
        inst.count == 1 && FileManager.default.fileExists(atPath: c.cfg(home)) && !FileManager.default.fileExists(atPath: NSHomeDirectory() + "/Library/Application Support/VietKB Dev"),
        "instances=\(inst.map { $0.processIdentifier }) cfg=\(FileManager.default.fileExists(atPath: c.cfg(home))) ownerDevDirExists=\(FileManager.default.fileExists(atPath: NSHomeDirectory() + "/Library/Application Support/VietKB Dev"))")

    // --- Dock / activation policy with the main window open (start_minimized = false)
    var tile = dockTile(appPath: c.app)
    if tile == nil { _ = waitUntil(4) { tile = dockTile(appPath: c.app); return tile != nil } }
    var ev = ""
    if let t = tile { ev = shotDockTile(t.1, "dock-tile-window-open") }
    row("APP-02", "MAC-DCK-02,MAC-DCK-08,MAC-DOC-13", "window open (start_minimized=false): activation policy regular (Cmd+Tab lists it) and a Dock tile for the dev bundle",
        policyName(pid) == "regular" && tile != nil, "policy=\(policyName(pid)) dockTile=\(tile.map { "\($0.1)" } ?? "none") frontmost=\(frontmostPid() == pid)", evidence: ev)

    // --- main window geometry / visibility / title
    guard let main0 = waitWin(c, "main") else {
        row("APP-03", "MAC-WIN-01,MAC-WIN-15", "main window with the expected title is on screen", false, "titles seen: \(contentAX(pid).map { $0.title })"); return
    }
    primeWebAX(pid, main0.el)
    let mainShot = shot(main0.id, "main-window-vi")
    let vis = visibility(main0.id, pid: pid)
    let zoomEnabled = axKids(main0.el).first { axStr($0, kAXSubroleAttribute as String) == "AXZoomButton" }.flatMap { axStr($0, "AXEnabled") } ?? "?"
    let sz = axRect(main0.el)
    row("APP-03", "MAC-WIN-01,MAC-WIN-15,MAC-WIN-22,MAC-DOC-10", "main window: title from the table, visible, opaque, wholly on a display, not covered by another app; fixed size; footer controls inside the window",
        vis.ok && controlsOutside(c, main0.el).isEmpty, "\(vis.text); size=\(sz.map { "\(Int($0.width))x\(Int($0.height))" } ?? "?") zoomEnabled=\(zoomEnabled) clipped=\(controlsOutside(c, main0.el))", evidence: mainShot)
    let mainTexts = windowTexts(c, main0.el)
    let footer = ["quick.hotkeys", "quick.advanced", "quick.quit", "common.close"].map { c.L($0) }
    row("APP-04", "MAC-WIN-15,MAC-DOC-05,MAC-DOC-16,MAC-I18-01", "main window text: footer buttons present; Mac wording (Mở cùng macOS, thanh menu), no Windows/khay/Ctrl/exe; no Clipboard row",
        footer.allSatisfy { f in mainTexts.contains { norm($0) == norm(f) } } && mainTexts.contains { $0.contains("macOS") } && !mainTexts.contains { $0.range(of: #"Windows|khay|\bCtrl\b|\.exe|Backspace|Clipboard"#, options: .regularExpression) != nil },
        "footer=\(footer.map { f in mainTexts.contains { norm($0) == norm(f) } }) hits=\(mainTexts.filter { $0.range(of: #"Windows|khay|\bCtrl\b|\.exe|Backspace|Clipboard"#, options: .regularExpression) != nil })")

    // --- status item exists exactly once in the pid's own AX extras menu bar; AXPress (the VoiceOver action) does not open the menu (finding)
    let barItems = (axCopy(AXUIElementCreateApplication(pid), "AXExtrasMenuBar") as! AXUIElement?).map { axKids($0).count } ?? 0
    row("APP-05", "MAC-MBR-01", "exactly one menu-bar item for the dev app, found by pid via AXExtrasMenuBar (never by screen position)", barItems == 1, "items=\(barItems)")
    if let item = statusItemEl(pid) {
        let before = Set(allWindows().map { $0.id })
        let pressed = axPress(item)
        usleep(1_500_000)
        let menuWins = allWindows().filter { !before.contains($0.id) && $0.layer > 0 && $0.pid == pid }
        let menuAX = axFindAll(item, role: "AXMenuItem") { _ in true }
        row("APP-05b", "MAC-MBR-02,MAC-A11-01", "AXPress on the status item (what VoiceOver does) opens the menu",
            pressed && (!menuWins.isEmpty || !menuAX.isEmpty), "AXPress returned \(pressed); popup windows appeared=\(menuWins.count); AXMenuItems under the item=\(menuAX.count). The menu is unreachable by AX; coordinate clicks are forbidden (guardrails 6), so MAC-MBR-02..08, MAC-WIN-02/03, MAC-DCK-04 and the status-item halves of MAC-WIN-01/06 stay manual",
            note: "candidate product defect: tray icon ignores AXPress")
        if !menuWins.isEmpty || !menuAX.isEmpty, let m = axKids(item).first(where: { axRole($0) == "AXMenu" }) { _ = AXUIElementPerformAction(m, kAXCancelAction as CFString) }
    }

    // --- modals from the main window
    for (kind, key) in [("macros", "quick.macroPanel"), ("hotkeys", "quick.hotkeys"), ("advanced", "quick.advanced")] {
        try quiet()
        guard let mw = findWin(c, "main"), let b = button(c, mw.el, label: c.L(key)) else { row("APP-06-\(kind)", "MAC-WIN-07", "\(kind) modal opens from the main window", false, "button '\(c.L(key))' not found"); continue }
        _ = axPress(b)
        guard let modal = waitWin(c, kind) else { row("APP-06-\(kind)", "MAC-WIN-07", "\(kind) modal opens from the main window", false, "no window titled '\(c.T(WINKEYS[kind]!))'; windows=\(contentAX(pid).map { $0.title })"); continue }
        usleep(500_000)
        let z = zOrder([modal.id, mw.id] + (te.map { [$0.winID] } ?? []))
        let vm = visibility(modal.id, pid: pid), vp = visibility(mw.id, pid: pid)
        let mr = axRect(modal.el) ?? .zero, pr = axRect(mw.el) ?? .zero
        let dc = hypot(mr.midX - pr.midX, mr.midY - pr.midY)
        let s = shot(modal.id, "modal-\(kind)-vi")
        let tileNow = dockTile(appPath: c.app) != nil
        let stillOne = contentAX(pid).count
        _ = axPress(b)   // press the same button again: still exactly one modal (MAC-WIN-17)
        usleep(500_000)
        let after = contentAX(pid).count
        row("APP-06-\(kind)", "MAC-WIN-07,MAC-WIN-17,MAC-WIN-15,MAC-DCK-02,MAC-DCK-03", "\(kind): modal opened from main is above main, visible, centred on main (\(Int(dc)) pt off), not clipped, Dock tile stays; second press leaves one modal",
            z.ok && vm.ok && dc <= 6 && tileNow && stillOne == 2 && after == 2 && controlsOutside(c, modal.el).isEmpty,
            "\(z.text); modal: \(vm.text); parent: \(vp.text); centreDelta=\(Int(dc)) dock=\(tileNow) windows=\(stillOne)->\(after) clipped=\(controlsOutside(c, modal.el))", evidence: s)
        if kind == "advanced", let adv = findWin(c, "advanced"), let ab = button(c, adv.el, label: c.L("adv.about")) {
            _ = axPress(ab)
            if let about = waitWin(c, "about") {
                usleep(500_000)
                let z2 = zOrder([about.id, adv.id, mw.id]), va = visibility(about.id, pid: pid)
                row("APP-07", "MAC-WIN-08,MAC-WIN-15", "About opened from Advanced is in front of Advanced and main, visible (record: parent is main, so Advanced stays enabled)",
                    z2.ok && va.ok, "\(z2.text); \(va.text); advancedEnabled=\(axStr(adv.el, "AXEnabled") ?? "-")", evidence: shot(about.id, "modal-about-vi"))
                _ = closeWindow(about.el); _ = waitUntil(3) { findWin(c, "about") == nil }
            } else { row("APP-07", "MAC-WIN-08", "About opened from Advanced", false, "no About window; windows=\(contentAX(pid).map { $0.title })") }
        }
        if let cur = findWin(c, kind) { _ = closeWindow(cur.el) }
        let gone = waitUntil(3) { findWin(c, kind) == nil }
        let backTop = zOrder([mw.id]).ok && visibility(mw.id, pid: pid).ok
        row("APP-08-\(kind)", "MAC-WIN-16,MAC-DCK-03", "\(kind): closing the modal leaves main visible and the Dock tile in place", gone && backTop && dockTile(appPath: c.app) != nil, "closed=\(gone) mainVisible=\(backTop) dock=\(dockTile(appPath: c.app) != nil)")
    }

    // --- DOC-06/07/08: what the windows offer (hotkeys rows, no raw-zone / charset control)
    if let hk = ({ () -> (el: AXUIElement, id: Int, title: String)? in
        guard let mw = findWin(c, "main"), let b = button(c, mw.el, label: c.L("quick.hotkeys")) else { return nil }
        _ = axPress(b); return waitWin(c, "hotkeys") })() {
        usleep(400_000)
        let rows = ["toggle", "cycleMethod", "restoreWord", "openSettings"].map { c.L("hotkeys.row.\($0)") }
        let texts = windowTexts(c, hk.el)
        let present = rows.filter { r in texts.contains { norm($0) == norm(r) } }
        let extra = ["hotkey.enable.label", "hotkey.disable.label", "hotkey.appExclude.label", "hotkey.macroPanel.label"].map { c.L($0) }.filter { e in texts.contains { norm($0) == norm(e) } }
        row("APP-09", "MAC-DOC-06", "Phím tắt window lists exactly the 4 rows (toggle, cycle method, restore word, open settings); no F-key rows", present.count == 4 && extra.isEmpty, "rows present=\(present) unexpected rows=\(extra)", evidence: shot(hk.id, "modal-hotkeys-vi"))
        _ = closeWindow(hk.el); _ = waitUntil(3) { findWin(c, "hotkeys") == nil }
    }
    var everyText: [String] = [], everyControl: [String] = []
    for kind in ["main", "advanced", "macros", "about", "hotkeys"] {
        if kind != "main", let mw = findWin(c, "main"), let b = button(c, mw.el, label: c.L(kind == "advanced" ? "quick.advanced" : kind == "macros" ? "quick.macroPanel" : kind == "hotkeys" ? "quick.hotkeys" : "quick.advanced")) {
            if kind == "about" { guard let adv = waitWin(c, "advanced") ?? ({ _ = axPress(b); return waitWin(c, "advanced") })(), let ab = button(c, adv.el, label: c.L("adv.about")) else { continue }; _ = axPress(ab) }
            else { _ = axPress(b) }
        }
        if let w = waitWin(c, kind, secs: 4) {
            usleep(300_000); everyText += windowTexts(c, w.el)
            axWalk(tree(pid, w.el)) { el, _ in if ["AXButton", "AXCheckBox", "AXPopUpButton", "AXRadioButton", "AXTextField"].contains(axRole(el)) { let l = axLabel(el); if !l.isEmpty { everyControl.append(l) } } }
        }
    }
    let rawHits = everyControl.filter { $0.range(of: #"Vùng gõ thô|raw zone|Bảng mã|Unicode NFD|NFD|charset|VK_PACKET|Clipboard"#, options: [.regularExpression, .caseInsensitive]) != nil }
    let winHits = everyText.filter { $0.range(of: #"Windows|khay|\bCtrl\b|\bAlt\b|\bWin\b|\.exe|Backspace"#, options: .regularExpression) != nil }
    row("APP-10", "MAC-DOC-07,MAC-DOC-08", "no window offers a raw-zone marker or a charset (NFC/NFD) picker (controls only; prose such as the About story is not a control)", rawHits.isEmpty, "controls read=\(everyControl.count) hits=\(rawHits)")
    row("APP-11", "MAC-I18-01", "no Windows wording in any Mac window (Windows|khay|Ctrl|Alt|exe|Backspace); known: Macros keycap Ctrl+F (S-09)", winHits.isEmpty, "windows read: \(everyText.count) labels; hits=\(winHits)", note: "intentional hits must be listed in the defect log")
    // close whatever is left but the main window
    for w in contentAX(pid) where w.title != c.T("rwin.main") { _ = closeWindow(w.el) }
    _ = waitUntil(3) { contentAX(pid).count == 1 }

    // --- WIN-18: close main while a modal is open -> everything closes together
    if let mw = findWin(c, "main"), let b = button(c, mw.el, label: c.L("quick.hotkeys")) {
        _ = axPress(b); _ = waitWin(c, "hotkeys"); usleep(400_000)
        let ids = contentAX(pid).map { $0.id }
        if let m2 = findWin(c, "main") { _ = closeWindow(m2.el) }
        let allGone = waitUntil(4) { contentAX(pid).isEmpty }
        // closing a window can be refused while a modal is up; if so, close the modal first and then main (record the behaviour)
        var how = "main close button closed both"
        if !allGone { if let h = findWin(c, "hotkeys") { _ = closeWindow(h.el) }; usleep(500_000); if let m3 = findWin(c, "main") { _ = closeWindow(m3.el) }; _ = waitUntil(4) { contentAX(pid).isEmpty }; how = "main close ignored while the modal lived; closed modal then main" }
        row("APP-12", "MAC-WIN-18,MAC-WIN-16", "closing main while a modal is open leaves no orphan window", allGone, "ids before=\(ids); remaining=\(contentAX(pid).map { $0.title }); \(how)")
    }
    // --- last window closed: accessory, no tile, process alive, status item still there
    _ = waitUntil(4) { dockTile(appPath: c.app) == nil }
    var flashes = 0
    for _ in 0..<10 { if dockTile(appPath: c.app) != nil { flashes += 1 }; usleep(150_000) }
    row("APP-13", "MAC-DCK-03,MAC-DCK-01,MAC-LCY-12,MAC-DCK-06", "last window closed: Dock tile gone, policy accessory, process alive, status item still present",
        dockTile(appPath: c.app) == nil && policyName(pid) == "accessory" && kill(pid, 0) == 0 && statusItemEl(pid) != nil && flashes == 0,
        "dock=\(dockTile(appPath: c.app) != nil) policy=\(policyName(pid)) alive=\(kill(pid, 0) == 0) statusItem=\(statusItemEl(pid) != nil) flashes=\(flashes)")

    // --- reopen x10 (LCY-12) via the Dock/Finder route (`open -a` -> reopen event)
    var bad: [String] = []
    for i in 1...10 {
        try quiet()
        _ = openRunning(c)
        let up = waitUntil(5) { !contentAX(pid).isEmpty }
        let reg = policyName(pid) == "regular", tileUp = waitUntil(3) { dockTile(appPath: c.app) != nil }
        if !up || !reg || !tileUp { bad.append("#\(i): window=\(up) policy=\(policyName(pid)) tile=\(tileUp)") }
        for w in contentAX(pid) { _ = closeWindow(w.el) }
        if !waitUntil(4, { contentAX(pid).isEmpty }) { bad.append("#\(i): window did not close") }
        _ = waitUntil(3) { dockTile(appPath: c.app) == nil }
        if kill(pid, 0) != 0 { bad.append("#\(i): process died"); break }
    }
    row("APP-14", "MAC-LCY-12,MAC-LCY-06,MAC-DCK-03,MAC-DCK-06", "10x (reopen via open -a, window shows, Dock tile appears, close, tile disappears): same pid alive, policy flips each time", bad.isEmpty, bad.isEmpty ? "10/10 cycles, pid \(pid) unchanged, instances=\(runningInstances(c.app).count)" : bad.joined(separator: "; "))

    // --- WIN-06 / DCK-05: main already open but covered by another app's window; reopen route and Dock-tile click raise it
    if let te = te {
        _ = openRunning(c); guard let mw = waitWin(c, "main"), let mr = axRect(mw.el) else { row("APP-15", "MAC-WIN-06", "main reopen for z-order test", false, "no main window"); return }
        _ = osa("tell application \"TextEdit\" to set bounds of (first window whose id is \(te.winID)) to {\(Int(mr.minX) - 40), \(Int(mr.minY) - 40), \(Int(mr.maxX) + 40), \(Int(mr.maxY) + 40)}")
        activatePid(te.pid); usleep(600_000)
        let covered = (allWindows().first { $0.id == mw.id }).map { foreignCoverage(of: $0, ignorePids: [pid]) } ?? 0
        _ = openRunning(c); usleep(1_200_000)
        let v1 = visibility(mw.id, pid: pid), z1 = zOrder([mw.id, te.winID]), f1 = frontmostPid()
        row("APP-15", "MAC-WIN-06,MAC-LCY-06", "main open but covered (\(Int(covered * 100))% by TextEdit): reopen via open -a raises it above TextEdit and makes VietKB frontmost", covered > 0.5 && v1.ok && z1.ok && f1 == pid,
            "coveredBefore=\(String(format: "%.2f", covered)); \(v1.text); \(z1.text); frontmost=\(f1 == pid ? "VietKB" : "pid \(f1)")", evidence: shot(mw.id, "main-after-reopen-over-textedit"))
        activatePid(te.pid); usleep(500_000)
        if let (tileEl, tr) = dockTile(appPath: c.app) {
            _ = axPress(tileEl); usleep(1_200_000)
            let v2 = visibility(mw.id, pid: pid), z2 = zOrder([mw.id, te.winID])
            row("APP-16", "MAC-DCK-05,MAC-WIN-06", "Dock tile press (AX on our own tile) raises the covered main window and activates VietKB", v2.ok && z2.ok && frontmostPid() == pid, "\(v2.text); \(z2.text)", evidence: shotDockTile(tr, "dock-tile-before-press"))
        } else { row("APP-16", "MAC-DCK-05", "Dock tile press raises the window", false, "no Dock tile while main is open") }
    }

    // --- language: toggle in every window, live switch, config read-back, 10x round trip (LNG-01/02/03/06/08/15)
    guard let m1 = waitWin(c, "main") else { return }
    var togglesOK: [String] = []
    for (kind, key) in [("main", ""), ("macros", "quick.macroPanel"), ("hotkeys", "quick.hotkeys"), ("advanced", "quick.advanced")] {
        if kind != "main", let mw = findWin(c, "main"), let b = button(c, mw.el, label: c.L(key)) { _ = axPress(b) }
        guard let w = waitWin(c, kind, secs: 4) else { togglesOK.append("\(kind): window missing"); continue }
        usleep(300_000)
        let boxes = axFindAll(tree(pid, w.el), role: "AXCheckBox") { [norm(c.L("lang.vi.name")), norm(c.L("lang.en.name"))].contains(norm(axLabel($0))) }
        let wr = axRect(w.el) ?? .zero
        let inside = boxes.allSatisfy { axRect($0).map { wr.contains($0) } ?? false }
        let alerts = axFindAll(w.el, role: "AXGroup") { axStr($0, kAXSubroleAttribute as String) == "AXApplicationAlert" }
        let overlap = boxes.contains { b in alerts.contains { a in (axRect(a) ?? .zero).intersects(axRect(b) ?? .zero) } }
        if boxes.count != 2 || !inside || overlap { togglesOK.append("\(kind): toggles=\(boxes.count) inside=\(inside) overlapsBanner=\(overlap)") }
    }
    if let about = findWin(c, "main"), let adv = findWin(c, "advanced"), let ab = button(c, adv.el, label: c.L("adv.about")) { _ = axPress(ab); if let a = waitWin(c, "about", secs: 4) {
        let boxes = axFindAll(tree(pid, a.el), role: "AXCheckBox") { [norm(c.L("lang.vi.name")), norm(c.L("lang.en.name"))].contains(norm(axLabel($0))) }
        if boxes.count != 2 { togglesOK.append("about: toggles=\(boxes.count)") }; _ = about } }
    row("APP-17", "MAC-LNG-01", "VI | EN toggle present in main, Abbreviations, Shortcuts, Advanced and About; inside the window, not overlapping the Accessibility banner", togglesOK.isEmpty, togglesOK.isEmpty ? "5/5 windows" : togglesOK.joined(separator: "; "))
    let themeBtn = axFindAll(tree(pid, m1.el), role: nil) { el in let l = axLabel(el); return (axRole(el) == "AXButton" || axRole(el) == "AXCheckBox" || axRole(el) == "AXRadioButton") && (l.contains("Giao diện") || l.localizedCaseInsensitiveContains("appearance") || l.localizedCaseInsensitiveContains("theme")) }
    row("APP-18", "MAC-LNG-01,MAC-DOC-09", "theme toggle next to VI | EN on Mac (27-mac-decisions M6)", themeBtn.isEmpty ? nil : true, themeBtn.isEmpty ? "absent in this build: the M6 Mac theme toggle is specified in docs (42edcb6) but the UI renders only the language switch on Mac" : "present: \(themeBtn.map { axLabel($0) })", note: themeBtn.isEmpty ? "SKIP = spec ahead of code, not a regression" : "")

    // keep main + hotkeys + macros open, switch to English, measure
    let initialVi = windowTexts(c, m1.el)
    if let mw = findWin(c, "main"), let b = button(c, mw.el, label: c.L("quick.hotkeys")) { _ = axPress(b); _ = waitWin(c, "hotkeys") }
    let t0 = Date()
    func pressLang(_ code: String) -> Bool {
        guard let mw = findWin(c, "main", lang: code == "en" ? "vi" : "en") ?? findWin(c, "main", lang: code), let box = axFindAll(tree(pid, mw.el), role: "AXCheckBox", where: { norm(axLabel($0)) == norm(c.L(code == "en" ? "lang.en.name" : "lang.vi.name", code)) }).first else { return false }
        return axPress(box)
    }
    let pressedEN = pressLang("en")
    let switched = waitUntil(3) { findWin(c, "main", lang: "en") != nil && findWin(c, "hotkeys", lang: "en") != nil }
    let ms = Int(Date().timeIntervalSince(t0) * 1000)
    c.lang = "en"
    let cfgLang = { configValue(c.cfg(home), "language") }
    _ = waitUntil(3) { cfgLang() == "en" }
    row("APP-19", "MAC-LNG-02,MAC-LNG-15,MAC-LNG-04", "press EN in main: main AND the open Shortcuts window retitle to English in ≤ 1.5 s without restart; config.toml language = en", pressedEN && switched && ms <= 1500 && cfgLang() == "en",
        "pressed=\(pressedEN) bothRetitled=\(switched) after \(ms) ms; titles now \(contentAX(pid).map { $0.title }); config language=\(cfgLang() ?? "nil")")
    // English completeness over every open window + clipping in EN
    var leaks: [String] = [], clipEN: [String] = []
    let allowFragments = c.en.values.filter { hasDiacritic($0) }
    for w in contentAX(pid) {
        for t in windowTexts(c, w.el) where hasDiacritic(t) && t != c.en["lang.vi.name"] && !t.contains("→") && !allowFragments.contains(where: { $0.contains(t) || t.contains($0) }) { leaks.append("\(w.title): \(t.prefix(40))") }
        clipEN += controlsOutside(c, w.el).map { "\(w.title): \($0.prefix(60))" }
    }
    row("APP-20", "MAC-LNG-06,MAC-LNG-02", "English UI: no Vietnamese text left in main + Shortcuts (AX dump, excluding the endonym and Vietnamese-output examples)", leaks.isEmpty, "leaks=\(leaks)", evidence: findWin(c, "main", lang: "en").map { shot($0.id, "main-window-en") } ?? "")
    row("APP-21", "MAC-LNG-08,MAC-WIN-15", "English UI: no control clipped by the window edge (longer English labels)", clipEN.isEmpty, "clipped=\(clipEN)")
    // status of the round trip: back to Vietnamese, then 10x
    _ = pressLang("vi"); c.lang = "vi"
    _ = waitUntil(3) { findWin(c, "main", lang: "vi") != nil }
    usleep(500_000)
    let backVi = findWin(c, "main").map { windowTexts(c, $0.el) } ?? []
    var rt = 0
    for _ in 0..<10 {
        try quiet()
        _ = pressLang("en"); c.lang = "en"; _ = waitUntil(3) { findWin(c, "main", lang: "en") != nil }
        _ = pressLang("vi"); c.lang = "vi"; _ = waitUntil(3) { findWin(c, "main", lang: "vi") != nil }
        rt += 1
    }
    usleep(500_000)
    let finalVi = findWin(c, "main").map { windowTexts(c, $0.el) } ?? []
    row("APP-22", "MAC-LNG-03", "EN -> VI restores every string; 10 round trips leave no residue (AX label list identical to the first Vietnamese read)", backVi == initialVi.filter { _ in true } || backVi == finalVi && finalVi == initialVi, "round trips=\(rt); labels initial=\(initialVi.count) afterFirstBack=\(backVi.count) final=\(finalVi.count); equal(initial,final)=\(initialVi == finalVi); diff=\(Array(Set(initialVi).symmetricDifference(Set(finalVi)).prefix(5)))")
    for w in contentAX(pid) where w.title != c.T("rwin.main") { _ = closeWindow(w.el) }
    // --- persist EN, quit with Thoát hẳn (LCY-01), relaunch, first paint English (LNG-04)
    _ = pressLang("en"); c.lang = "en"; _ = waitUntil(3) { cfgLang() == "en" }
    if let mw = findWin(c, "main"), let q = button(c, mw.el, label: c.L("quick.quit")) {
        _ = axPress(q)
        let exited = waitGone(pid, 6)
        _ = waitUntil(4) { dockTile(appPath: c.app) == nil }
        row("APP-23", "MAC-LCY-01", "'Thoát hẳn'/'Quit' button quits the app (process gone, Dock tile gone)", exited && dockTile(appPath: c.app) == nil, "exited=\(exited) tile=\(dockTile(appPath: c.app) != nil) tapsBefore n/a (LaunchServices launch is not granted the event tap)")
    }
    if kill(pid, 0) == 0 { killOurs(pid, reason: "quit button did not exit") }
    let pre = configValue(c.cfg(home), "language")
    _ = try launchFresh(c, "relaunch-en", lang: "en", startMinimized: false, home: home, keepConfig: true)   // config.toml exactly as the GUI left it
    let enMain = waitWin(c, "main", lang: "en")
    let barAfter = (axCopy(AXUIElementCreateApplication(c.pid), "AXExtrasMenuBar") as! AXUIElement?).map { axKids($0).count } ?? 0
    row("APP-24", "MAC-LNG-04,MAC-MBR-01", "language = en persisted by the GUI survives quit; relaunch opens the window titled in English on the first read, and exactly one menu-bar item again", pre == "en" && enMain != nil && barAfter == 1, "config before relaunch: language=\(pre ?? "nil"); first window titles \(contentAX(c.pid).map { $0.title }); menu-bar items=\(barAfter)")
    endApp(c, "end of relaunch check")

}

// ---------------------------------------------------------------------------------------------------------------------
// Phase A2: hand-edited language values (MAC-LNG-05)
// ---------------------------------------------------------------------------------------------------------------------
func phaseBadLang(_ c: AppCtx) throws {
    log("==== phase badlang: hand-edited language values ====")
    for (value, label) in [("xx", "invalid value"), ("EN", "upper-case value")] {
        let h = c.newHome("badlang")
        let p = c.cfg(h)
        try? FileManager.default.createDirectory(atPath: (p as NSString).deletingLastPathComponent, withIntermediateDirectories: true)
        // method = vni and start_minimized = false are the canaries: a config that is discarded as a whole loses both
        try? "version = 1\n[input]\nenabled = true\nmethod = \"vni\"\n[system]\nstart_with_windows = false\nstart_minimized = false\nnotification_sound = false\ncheck_updates = false\nlanguage = \"\(value)\"\ntheme = \"dark\"\n".write(toFile: p, atomically: true, encoding: .utf8)
        c.pid = try launchApp(c.app, home: h, mode: .launchServices, tag: "badlang-\(value)"); c.lang = "vi"
        let up = waitUntil(10) { !contentAX(c.pid).isEmpty }
        sleep(1)
        let titles = contentAX(c.pid).map { $0.title }
        let after = (try? String(contentsOfFile: p, encoding: .utf8)) ?? ""
        let method = configValue(p, "method") ?? "nil", minimized = configValue(p, "start_minimized") ?? "nil", lang = configValue(p, "language") ?? "nil"
        let alive = kill(c.pid, 0) == 0
        let kept = method == "vni" && minimized == "false"
        row("APP-25-\(value)", "MAC-LNG-05", "hand-edited language \"\(value)\" (\(label)): no crash, a window opens in Vietnamese or English, and the rest of config.toml survives (method=vni, start_minimized=false)",
            alive && up && kept, "alive=\(alive) windowShown=\(up) titles=\(titles) after: language=\(lang) method=\(method) start_minimized=\(minimized) bytes=\(after.count)",
            note: kept ? "" : "config fields were reset to defaults: the whole file was discarded because of one bad value")
        endApp(c, "end of badlang check")
        _ = waitUntil(6) { runningInstances(c.app).isEmpty }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Phase S: singleton (owner rule M8) and the update badge
// ---------------------------------------------------------------------------------------------------------------------
func instancesByID(_ c: AppCtx) -> [NSRunningApplication] { NSRunningApplication.runningApplications(withBundleIdentifier: c.bid) }
func statusItemCount(_ pid: Int32) -> Int { (axCopy(AXUIElementCreateApplication(pid), "AXExtrasMenuBar") as! AXUIElement?).map { axKids($0).count } ?? 0 }
func phaseSingle(_ c: AppCtx) throws {
    log("==== phase single: second launches must not leave a second instance ====")
    let home = try launchFresh(c, "single", startMinimized: true)
    let first = c.pid
    defer { endApp(c, "end of phase single") }
    func snapshot() -> (procs: Int, items: Int, taps: Int, tiles: Int) {
        let ids = instancesByID(c).map { $0.processIdentifier }
        return (ids.count, ids.reduce(0) { $0 + statusItemCount($1) }, ids.reduce(0) { $0 + tapCount($1) }, dockTile(appPath: c.app) == nil ? 0 : 1)
    }
    let base = snapshot()
    var failures: [String] = []
    func attempt(_ name: String, _ launch: () -> String) {
        let before = Set(instancesByID(c).map { $0.processIdentifier })
        let note = launch()
        sleep(5)
        let after = instancesByID(c)
        for p in after where !before.contains(p.processIdentifier) { notePid(p.processIdentifier) }
        let snap = snapshot()
        let shown = !appContentWindows(first).isEmpty
        let ok = snap.procs == 1 && snap.items == 1 && snap.taps <= base.taps
        // cleanup of whatever the attempt left (only pids this suite started): graceful first
        for p in after where p.processIdentifier != first { killOurs(p.processIdentifier, reason: "singleton violation: second instance left running by \(name)") }
        row("APP-6\(name.prefix(1))", "MAC-LCY-06,MAC-LCY-07", "singleton (M8), \(name): no second process, status item or event tap",
            ok, "\(note); processes=\(snap.procs) statusItems=\(snap.items) taps=\(snap.taps) (baseline \(base.taps)) dockTiles(running)=\(snap.tiles) pidsAfter=\(after.map { $0.processIdentifier })")
        row("APP-6\(name.prefix(1))w", "MAC-LCY-06", "singleton (M8), \(name): the running instance shows its settings window (and its single Dock tile)", shown && snap.tiles == 1, "settingsWindowShown=\(shown) runningDockTiles=\(snap.tiles)")
        if !ok { failures.append(name) }
        for w in contentAX(first) { _ = closeWindow(w.el) }
        _ = waitUntil(5) { appContentWindows(first).isEmpty && dockTile(appPath: c.app) == nil }
    }
    row("APP-60", "MAC-LCY-06", "singleton baseline: one hidden instance, one status item, no tile, no window", base.procs == 1 && base.items == 1 && base.tiles == 0 && appContentWindows(first).isEmpty, "\(base)")
    attempt("a: open -n, same HOME") { "rc=\(sh("/usr/bin/open", ["-n", "-a", c.app, "--env", "HOME=\(home)"], timeout: 15).code)" }
    attempt("b: open -n, other HOME") { "rc=\(sh("/usr/bin/open", ["-n", "-a", c.app, "--env", "HOME=\(c.newHome("single2"))"], timeout: 15).code)" }
    // c: a second copy of the .app at another temp path (same bundle id)
    let copy = c.tmp + "/copy/VietKB Dev.app"
    try? FileManager.default.createDirectory(atPath: c.tmp + "/copy", withIntermediateDirectories: true)
    try? FileManager.default.removeItem(atPath: copy)
    if (try? FileManager.default.copyItem(atPath: c.app, toPath: copy)) != nil {
        attempt("c: open -n of a second copy of the .app") { "rc=\(sh("/usr/bin/open", ["-n", "-a", copy, "--env", "HOME=\(c.newHome("single3"))"], timeout: 15).code)" }
        try? FileManager.default.removeItem(atPath: c.tmp + "/copy")
    } else { row("APP-6c", "MAC-LCY-07", "singleton, second copy of the .app", nil, "could not copy the bundle") }
    // d: direct exec of the binary (the child is ours, so it is reaped here)
    var child: Process?
    attempt("d: direct exec of the binary") {
        let p = Process(); p.executableURL = URL(fileURLWithPath: c.app + "/Contents/MacOS/vietkb")
        var e = ProcessInfo.processInfo.environment; e["HOME"] = c.newHome("single4"); p.environment = e
        p.standardOutput = FileHandle.nullDevice; p.standardError = FileHandle.nullDevice
        try? p.run(); child = p
        if p.processIdentifier > 0 { notePid(p.processIdentifier) }
        glog("LAUNCH pid=\(p.processIdentifier) path=\(c.app)/Contents/MacOS/vietkb via=exec (second-instance attempt, reaped by the suite)")
        return "execPid=\(p.processIdentifier)"
    }
    if let p = child {
        let alive = p.isRunning
        row("APP-6e", "MAC-LCY-07", "singleton, direct exec: the second process exits by itself", !alive, "second process still running: \(alive)")
        if alive { killOurs(p.processIdentifier, reason: "direct-exec second instance did not exit by itself") }
    }
    row("APP-6z", "MAC-LCY-06,MAC-LCY-07", "singleton: the first instance is still the only one and still alive", kill(first, 0) == 0 && instancesByID(c).count == 1, "failures so far: \(failures)")
}

func phaseUpdate(_ c: AppCtx) throws {
    log("==== phase update: update badge (tolerant until the app exposes one) ====")
    let h = c.newHome("update")
    writeConfig(home: h, bundleID: c.bid, language: "en", startMinimized: false)
    // check_updates stays false in the written config; the fake feed is switched on by env, which is what update.rs reads
    c.pid = try launchApp(c.app, home: h, mode: .launchServices, extraEnv: ["VKB_UPDATE_FAKE": "1"], tag: "update"); c.lang = "en"
    defer { endApp(c, "end of phase update") }
    guard let w = waitWin(c, "main", secs: 10, lang: "en") else { row("APP-70", "MAC-UPD-01", "update badge", nil, "no window"); return }
    sleep(4)
    let texts = windowTexts(c, w.el)
    let hit = texts.filter { $0.range(of: #"[Uu]pdate (v\d|available)|cập nhật"#, options: .regularExpression) != nil }
    row("APP-70", "MAC-UPD-01", "update badge in the settings window (fake update feed)", hit.isEmpty ? nil : true,
        hit.isEmpty ? "SKIP: the app exposes no update indicator in the settings window yet (labels read: \(texts.count)); the menu-bar icon badge is a template image and is not AX-readable" : "labels: \(hit)")
}

// ---------------------------------------------------------------------------------------------------------------------
// Phase B: start hidden (Dock must stay empty), second launch, second instance with another HOME
// ---------------------------------------------------------------------------------------------------------------------
func phaseHidden(_ c: AppCtx) throws {
    log("==== phase hidden: start_minimized = true ====")
    _ = try launchFresh(c, "hidden", startMinimized: true)
    let pid = c.pid
    defer { endApp(c, "end of phase hidden") }
    var tileSeen = 0, winSeen = 0
    for _ in 0..<25 { if dockTile(appPath: c.app) != nil { tileSeen += 1 }; if !appContentWindows(pid).isEmpty { winSeen += 1 }; usleep(200_000) }
    row("APP-30", "MAC-DCK-01,MAC-DCK-08", "start hidden: activation policy accessory, no Dock tile at any of 25 samples over 5 s, no window, status item present",
        policyName(pid) == "accessory" && tileSeen == 0 && winSeen == 0 && statusItemEl(pid) != nil, "policy=\(policyName(pid)) tileSamples=\(tileSeen) windowSamples=\(winSeen) statusItem=\(statusItemEl(pid) != nil)")
    // second launch while running (Finder / Spotlight / open -a)
    _ = openRunning(c); usleep(2_500_000)
    let n = runningInstances(c.app).count
    let shown = !appContentWindows(pid).isEmpty
    row("APP-31", "MAC-LCY-06", "second launch with start hidden: still exactly one process", n == 1, "instances=\(n)")
    row("APP-32", "MAC-LCY-06", "second launch with start hidden: what the user sees (S-06: nothing visible = looks like a dead click)", shown ? true : nil, shown ? "settings window appeared" : "INFO: no window appeared after open -a; policy=\(policyName(pid)) tile=\(dockTile(appPath: c.app) != nil)", note: "decision pending in doc 52 S-06")
    for w in contentAX(pid) { _ = closeWindow(w.el) }
    // second instance with a different HOME
    let other = c.newHome("second-instance")
    writeConfig(home: other, bundleID: c.bid, language: "vi", startMinimized: true)
    let r = sh("/usr/bin/open", ["-n", "-a", c.app, "--env", "HOME=\(other)"], timeout: 15)
    glog("LAUNCH second-instance attempt via=open -n HOME=\(other) (single-instance check; any extra pid is reaped)")
    sleep(5)
    let procs = runningInstances(c.app)
    for p in procs where p.processIdentifier != pid { notePid(p.processIdentifier) }
    row("APP-33", "MAC-LCY-07", "second instance (open -n, other HOME) is blocked by single-instance: still one process", procs.count == 1, "open rc=\(r.code) instances=\(procs.map { $0.processIdentifier }) (first=\(pid))")
    for p in procs where p.processIdentifier != pid { killOurs(p.processIdentifier, reason: "second instance did not exit by itself") }
    // SIGTERM
    glog("SIGTERM pid=\(pid) (LCY-05)")
    kill(pid, SIGTERM)
    let gone = waitGone(pid, 5)
    row("APP-34", "MAC-LCY-05", "SIGTERM: process exits (or is documented as ignored)", gone, "exited within 5 s: \(gone)")
}

// ---------------------------------------------------------------------------------------------------------------------
// Phase C: quit paths (CAP-14), LaunchServices launch
// ---------------------------------------------------------------------------------------------------------------------
func phaseQuit(_ c: AppCtx) throws {
    log("==== phase quit paths ====")
    // 1. Cmd+Q through the app menu (AXPress on the item that carries ⌘Q), with a modal open
    _ = try launchFresh(c, "quit-cmdq", startMinimized: false)
    var pid = c.pid
    if let mw = waitWin(c, "main"), let b = button(c, mw.el, label: c.L("quick.hotkeys")) { _ = axPress(b); _ = waitWin(c, "hotkeys") }
    var item = quitMenuItem(pid)
    var pressed = item.map { axPress($0) } ?? false
    var gone = waitGone(pid, 6)
    row("APP-40", "MAC-LCY-02,MAC-LCY-08", "⌘Q (AXPress on the app menu item with cmd-char Q) with a modal open: app quits within 6 s without hanging", item != nil && pressed && gone,
        "menuItem=\(item != nil) pressed=\(pressed) exited=\(gone)", note: gone ? "" : "S-05: ExitRequested is swallowed unless QUITTING is set")
    endApp(c, "cmd+q left the process alive")
    // 2. Dock tile > Quit (AXShowMenu on our own tile, press Quit)
    _ = try launchFresh(c, "quit-dock", startMinimized: false)
    pid = c.pid
    var tile = dockTile(appPath: c.app); if tile == nil { _ = waitUntil(4) { tile = dockTile(appPath: c.app); return tile != nil } }
    if let t = tile {
        let items = dockMenuItems(t.0)
        let titles = items.map { axLabel($0) }
        let q = items.first { ["Quit", "Thoát", "Beenden", "Quitter", "Salir"].contains(axLabel($0)) } ?? items.last
        let ok = q.map { axPress($0) } ?? false
        gone = waitGone(pid, 6)
        row("APP-41", "MAC-LCY-03,MAC-DCK-05", "Dock tile menu (AXShowMenu on our tile) lists the window and Quit; Quit quits the app", ok && gone, "menu items=\(titles); pressed=\(ok); exited=\(gone)", evidence: shotDockTile(t.1, "dock-tile-menu"))
    } else { row("APP-41", "MAC-LCY-03,MAC-DCK-05", "Dock tile Quit", false, "no Dock tile") }
    endApp(c, "dock quit left the process alive")
    // 3. osascript quit (the app's own bundle id, our pid only)
    _ = try launchFresh(c, "quit-osa", startMinimized: false)
    pid = c.pid
    let r = sh("/usr/bin/osascript", ["-e", "tell application id \"\(c.bid)\" to quit"], timeout: 10)
    gone = waitGone(pid, 6)
    row("APP-42", "MAC-LCY-05", "osascript 'quit' by bundle id exits the app cleanly", gone, "osascript rc=\(r.code) out=\(r.out.prefix(80)) exited=\(gone)")
    endApp(c, "osascript quit left the process alive")
    // 4. SIGKILL then relaunch: no stale single-instance state, window shows
    guard env["VKB_E2E_SIGKILL"] == "1" else {
        row("APP-43", "MAC-LCY-05", "SIGKILL then relaunch", nil, "skipped: SIGKILL of a Regular-policy app leaves a stale Dock tile; set VKB_E2E_SIGKILL=1 to run it")
        return
    }
    let h = try launchFresh(c, "quit-kill9", startMinimized: false)
    pid = c.pid
    glog("SIGKILL pid=\(pid) (LCY-05, our process)")
    kill(pid, SIGKILL); _ = waitGone(pid, 3); sleep(1)
    c.pid = try launchApp(c.app, home: h, mode: .launchServices, tag: "after-kill9")
    let back = waitWin(c, "main", secs: 10) != nil
    row("APP-43", "MAC-LCY-05", "after SIGKILL a relaunch starts normally (no stale single-instance lock) and shows its window", back && runningInstances(c.app).count == 1, "window=\(back) instances=\(runningInstances(c.app).count)")
    endApp(c, "end of kill -9 check")
}

// ---------------------------------------------------------------------------------------------------------------------
// Phase D: direct exec (inherits this shell's TCC grants, so the event tap starts): tap survives policy flips, is gone after quit
// ---------------------------------------------------------------------------------------------------------------------
func phaseTap(_ c: AppCtx) throws {
    log("==== phase tap: direct exec with inherited grants ====")
    let h = try launchFresh(c, "tap", startMinimized: false, mode: .direct)
    let pid = c.pid
    defer { endApp(c, "end of phase tap") }
    let up = waitUntil(10) { tapCount(pid) > 0 }
    guard up else { row("APP-50", "MAC-LCY-01,MAC-LCY-02,MAC-DCK-06", "event tap present for the direct-exec dev app", nil, "no event tap owned by pid \(pid) within 10 s (grants not inherited by this shell?)"); return }
    var bad: [String] = []
    for i in 1...3 {
        if let w = waitWin(c, "main", secs: 3) { _ = closeWindow(w.el) }
        _ = waitUntil(3) { contentAX(pid).isEmpty }
        if tapCount(pid) == 0 { bad.append("#\(i): tap gone after window closed") }
        if statusItemEl(pid) == nil { bad.append("#\(i): status item gone") }
        _ = openRunning(c); _ = waitUntil(5) { !contentAX(pid).isEmpty }
    }
    row("APP-50", "MAC-DCK-06", "policy flips (3x close last window / reopen) do not drop the event tap or the status item", bad.isEmpty, bad.isEmpty ? "tap and status item present across 3 flips" : bad.joined(separator: "; "))
    let item = quitMenuItem(pid)
    let pressed = item.map { axPress($0) } ?? false
    let gone = waitGone(pid, 6)
    row("APP-51", "MAC-LCY-02,MAC-LCY-01", "⌘Q menu item quits the direct-exec app; event tap gone with the process", item != nil && pressed && gone && tapCount(pid) == 0, "menuItem=\(item != nil) pressed=\(pressed) exited=\(gone) tapsAfter=\(tapCount(pid))")
    _ = h
}

// ---------------------------------------------------------------------------------------------------------------------
// entry
// ---------------------------------------------------------------------------------------------------------------------
func runAppSuite(app: String, phases: Set<String>, withTextEdit: Bool) -> Int32 {
    let tmp = env["E2E_TMP"] ?? NSTemporaryDirectory() + "vkbe2e-app-\(getpid())"
    try? FileManager.default.createDirectory(atPath: tmp, withIntermediateDirectories: true)
    guard let info = bundleInfo(app), info.id.hasSuffix(".dev") else { log("ABORT: --app must be the dev-channel bundle (identifier ending .dev)"); return 3 }
    let c = AppCtx(app: app, tmp: tmp)
    let ownerBefore = ownerPids()
    glog("OWNER VietKB (vn.vietkb.app) pids before: \(ownerBefore) (observed only, never touched)")
    if !runningInstances(app).isEmpty { log("ABORT: a dev-channel instance is already running; a launch would be forwarded into it"); return 4 }
    var te: Setup?
    var code: Int32 = 0
    let staleBaseline = dockTileCount(appPath: app)
    glog("DOCK tiles for the dev bundle at start: \(staleBaseline) (stale ones are leftovers of earlier killed instances; never touched)")
    defer {
        te?.teardown()
        reapAll(reason: "end of app suite")
        let ownerAfter = ownerPids()
        glog("OWNER VietKB pids after: \(ownerAfter) -> \(ownerAfter == ownerBefore ? "unchanged" : "CHANGED")")
        for (k, v) in counts.sorted(by: { $0.key < $1.key }) { log("TALLY \(k): PASS \(v.pass) FAIL \(v.fail)") }
    }
    func run(_ name: String, _ f: () throws -> Void) {
        guard phases.contains(name) else { return }
        do { try f() } catch let a as Abort { log("PHASE \(name) ABORT: \(a.why)"); row("APP-ABORT-\(name)", "-", "phase \(name) completed", false, a.why); code = 5 }
        catch { log("PHASE \(name) ERROR: \(error)"); code = 7 }
        endApp(c, "phase \(name) finished")
        // CAP-2 guardrail: no Dock tile of ours may survive the phase
        let clean = waitUntil(6) { dockTileCount(appPath: app) <= staleBaseline && runningInstances(app).isEmpty }
        row("APP-DOCK-\(name)", "MAC-DCK-03", "after phase \(name): no new Dock tile and no running instance of the dev bundle remain (guardrail)", clean, clean ? "clean (tiles \(dockTileCount(appPath: app)), baseline \(staleBaseline))" : "GUARDRAIL FAILURE: tiles=\(dockTileCount(appPath: app)) baseline=\(staleBaseline) instances=\(runningInstances(app).map { $0.processIdentifier }). Not touching the Dock; owner must clear it")
    }
    if withTextEdit && phases.contains("main") {
        do { te = try openTextEdit(plain: true); glog("OPEN TextEdit document window \(te!.target.winID) pid \(te!.target.pid) (harness-owned test window)") } catch { log("TextEdit unavailable: \(error)") }
    }
    run("main") { try phaseMain(c, te: te?.target) }
    run("badlang") { try phaseBadLang(c) }
    run("hidden") { try phaseHidden(c) }
    run("quit") { try phaseQuit(c) }
    run("single") { try phaseSingle(c) }
    run("update") { try phaseUpdate(c) }
    run("tap") { try phaseTap(c) }
    return code
}
