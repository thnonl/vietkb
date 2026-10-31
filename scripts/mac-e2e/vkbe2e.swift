// vkbe2e: real-app E2E harness for the VietKB macOS port.
//
// Build:  swiftc -O -swift-version 5 -o vkbe2e vkbe2e.swift
// Usage:  see README.md (normally driven by run.sh, which owns the process/config guardrails).
//
// Guardrails built in (docs/evkey-clone-work/99-agent-guardrails.md, mac-handoff.md):
//  * physical-input watcher: a listen-only CGEventTap flags any key/mouse event that has no source pid
//    AND neither our marker nor VietKB's injection marker in kCGEventSourceUserData -> abort at once.
//  * before and after EVERY batch (and every ~12 keys inside a batch): the AX-focused application and the
//    topmost normal window must be exactly the window this harness opened. Mismatch -> nothing more is sent.
//  * idle gate: HID idle >= 60 s before the first synthetic event.
//  * never presses Return/Enter outside the hidden-answer dialog we own; never touches the clipboard.
import Foundation
import AppKit
import ApplicationServices
import Carbon.HIToolbox

setvbuf(stdout, nil, _IOLBF, 0)

let MARKER: Int64 = 0x564B_4532_4545        // "VKE2EE": ours
let VK_MARKER: Int64 = 0x0000_564B_4D41_4331 // vkb-macos MARKER (its own injections)

// MARK: - logging / errors
let env = ProcessInfo.processInfo.environment
let outDir = env["E2E_OUT"] ?? FileManager.default.currentDirectoryPath
let t0 = Date()
func ts() -> String { let f = DateFormatter(); f.dateFormat = "HH:mm:ss.SSS"; return f.string(from: Date()) }
var logHandle: FileHandle? = {
    FileManager.default.createFile(atPath: outDir + "/harness.log", contents: nil, attributes: nil)
    return FileHandle(forWritingAtPath: outDir + "/harness.log")
}()
func log(_ s: String) {
    let line = "\(ts()) \(s)\n"
    FileHandle.standardOutput.write(line.data(using: .utf8)!)
    logHandle?.seekToEndOfFile(); logHandle?.write(line.data(using: .utf8)!)
}
struct Abort: Error { let why: String; let physical: Bool }
func abort(_ why: String, physical: Bool = false) -> Abort { Abort(why: why, physical: physical) }

// MARK: - process helpers
@discardableResult
func sh(_ path: String, _ args: [String], timeout: Double = 20) -> (code: Int32, out: String) {
    let p = Process(); p.executableURL = URL(fileURLWithPath: path); p.arguments = args
    let pipe = Pipe(); p.standardOutput = pipe; p.standardError = pipe
    do { try p.run() } catch { return (-1, "run failed: \(error)") }
    let deadline = Date().addingTimeInterval(timeout)
    while p.isRunning && Date() < deadline { usleep(20_000) }
    if p.isRunning { p.terminate(); usleep(100_000) }
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    return (p.terminationStatus, String(data: data, encoding: .utf8)?.trimmingCharacters(in: .newlines) ?? "")
}
func osa(_ script: String, timeout: Double = 15) -> String {
    sh("/usr/bin/osascript", ["-e", script], timeout: timeout).out
}

// MARK: - physical-input watcher
var wFlag: String? = nil
var wSeen = 0, wOurs = 0, wVK = 0
var wVerbose = false
var wPhys: [String: Int] = [:]
var wOthers: [String: Int] = [:]   // "type/pid" of events with a source pid that are not ours (other software, e.g. other tools posting events)
var wTap: CFMachPort?
let wCallback: CGEventTapCallBack = { _, type, event, _ in
    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let t = wTap { CGEvent.tapEnable(tap: t, enable: true) }
        return Unmanaged.passUnretained(event)
    }
    let pid = event.getIntegerValueField(.eventSourceUnixProcessID)
    let ud = event.getIntegerValueField(.eventSourceUserData)
    wSeen += 1
    if ud == MARKER { wOurs += 1 }
    else if ud == VK_MARKER { wVK += 1 }
    else if pid != 0 { wOthers["type\(type.rawValue)/pid\(pid)", default: 0] += 1 }
    else if pid == 0 {
        if wVerbose { wPhys["type\(type.rawValue)", default: 0] += 1 }
        if wFlag == nil {
            wFlag = "physical event type=\(type.rawValue) keycode=\(event.getIntegerValueField(.keyboardEventKeycode)) at \(ts())"
        }
    }
    return Unmanaged.passUnretained(event)
}
func startWatcher() -> Bool {
    let types: [CGEventType] = [.keyDown, .keyUp, .flagsChanged, .leftMouseDown, .rightMouseDown, .otherMouseDown, .scrollWheel, .mouseMoved, .leftMouseDragged]
    var mask: CGEventMask = 0
    for t in types { mask |= CGEventMask(1) << CGEventMask(t.rawValue) }
    guard let tap = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap, options: .listenOnly,
                                      eventsOfInterest: mask, callback: wCallback, userInfo: nil) else { return false }
    wTap = tap
    let th = Thread {
        let src = CFMachPortCreateRunLoopSource(nil, tap, 0)
        CFRunLoopAddSource(CFRunLoopGetCurrent(), src, .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
        CFRunLoopRun()
    }
    th.start()
    return true
}

func hidIdle() -> Double {
    CGEventSource.secondsSinceLastEventType(.hidSystemState, eventType: CGEventType(rawValue: UInt32.max)!)
}

// MARK: - windows / AX
struct Win { let id: Int; let pid: Int32; let layer: Int; let owner: String; let name: String; let bounds: CGRect; let alpha: Double }
func windows() -> [Win] {
    guard let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] else { return [] }
    return list.map { d in
        var r = CGRect.zero
        if let b = d[kCGWindowBounds as String] as? [String: Any] { r = CGRect(dictionaryRepresentation: b as CFDictionary) ?? .zero }
        return Win(id: d[kCGWindowNumber as String] as? Int ?? 0, pid: Int32(d[kCGWindowOwnerPID as String] as? Int ?? 0),
                   layer: d[kCGWindowLayer as String] as? Int ?? 0, owner: d[kCGWindowOwnerName as String] as? String ?? "",
                   name: d[kCGWindowName as String] as? String ?? "", bounds: r, alpha: d[kCGWindowAlpha as String] as? Double ?? 1)
    }
}
func topNormalWindow() -> Win? {
    windows().first { $0.layer == 0 && $0.alpha > 0 && $0.bounds.width > 40 && $0.bounds.height > 20 }
}
func axCopy(_ el: AXUIElement, _ attr: String) -> AnyObject? {
    var v: AnyObject?
    return AXUIElementCopyAttributeValue(el, attr as CFString, &v) == .success ? v : nil
}
func focusedAppPid() -> Int32 {
    for _ in 0..<90 {   // the first AX query of a fresh process can fail for ~2 s
        let sys = AXUIElementCreateSystemWide()   // a fresh element per try: a failed one can stay broken for this process
        if let a = axCopy(sys, kAXFocusedApplicationAttribute as String) {
            var pid: pid_t = 0
            AXUIElementGetPid(a as! AXUIElement, &pid)
            return pid
        }
        usleep(60_000)
    }
    return -1
}
func axFocusedValue(pid: Int32) -> String? {
    let app = AXUIElementCreateApplication(pid)
    guard let f = axCopy(app, kAXFocusedUIElementAttribute as String) else { return nil }
    if let s = axCopy(f as! AXUIElement, kAXValueAttribute as String) as? String { return s }
    return nil
}
func axFocusedDesc(pid: Int32) -> String {
    let app = AXUIElementCreateApplication(pid)
    guard let f = axCopy(app, kAXFocusedUIElementAttribute as String) else { return "none" }
    let role = axCopy(f as! AXUIElement, kAXRoleAttribute as String) as? String ?? "?"
    let sub = axCopy(f as! AXUIElement, kAXSubroleAttribute as String) as? String ?? ""
    return role + (sub.isEmpty ? "" : "/" + sub)
}
func axTexts(pid: Int32, window: Int? = nil, limit: Int = 400) -> [String] {
    var out: [String] = []
    func walk(_ el: AXUIElement, _ depth: Int) {
        if out.count >= limit || depth > 14 { return }
        let role = axCopy(el, kAXRoleAttribute as String) as? String ?? ""
        for a in [kAXTitleAttribute, kAXValueAttribute, kAXDescriptionAttribute] {
            if let s = axCopy(el, a as String) as? String, !s.isEmpty { out.append("\(role): \(s)") ; break }
        }
        if let kids = axCopy(el, kAXChildrenAttribute as String) as? [AXUIElement] { for k in kids { walk(k, depth + 1) } }
    }
    // windows only: the app's menu bar would pull the owner's "Recent Items" into the evidence
    for w in (axCopy(AXUIElementCreateApplication(pid), kAXWindowsAttribute as String) as? [AXUIElement]) ?? [] { walk(w, 0) }
    return out
}

func axRect(_ el: AXUIElement) -> CGRect? {
    guard let pv = axCopy(el, kAXPositionAttribute as String), let sv = axCopy(el, kAXSizeAttribute as String) else { return nil }
    var p = CGPoint.zero, z = CGSize.zero
    guard AXValueGetValue(pv as! AXValue, .cgPoint, &p), AXValueGetValue(sv as! AXValue, .cgSize, &z) else { return nil }
    return CGRect(origin: p, size: z)
}
/// The app's menu-bar extra (status item). On recent macOS it is not a CGWindow of the app, so go through AX.
func statusItem(pid: Int32) -> (AXUIElement, CGRect)? {
    let app = AXUIElementCreateApplication(pid)
    guard let bar = axCopy(app, "AXExtrasMenuBar") else { return nil }
    guard let kids = axCopy(bar as! AXUIElement, kAXChildrenAttribute as String) as? [AXUIElement], let k = kids.first, let r = axRect(k) else { return nil }
    return (k, r)
}
/// Click the status item with a synthetic mouse event (AXPress does not open the Tauri menu, AXShowMenu is unsupported).
/// Hit-test guard: the AX element under the pointer must belong to `pid`, else nothing is sent.
/// Returns the on-screen windows that appeared (the menu is not in the AX tree of the item).
func clickStatusItem(pid: Int32, frame: CGRect, right: Bool) -> (String, [Win]) {
    let c = CGPoint(x: frame.midX, y: frame.midY)
    var hit: AXUIElement?
    AXUIElementCopyElementAtPosition(AXUIElementCreateSystemWide(), Float(c.x), Float(c.y), &hit)
    var hp: pid_t = 0
    if let h = hit { AXUIElementGetPid(h, &hp) }
    if hp != pid { return ("hit-test pid \(hp) != \(pid): click NOT sent", []) }
    let before = Set(windows().map { $0.id })
    let orig = CGEvent(source: nil)?.location ?? .zero
    let (dn, up, btn): (CGEventType, CGEventType, CGMouseButton) = right ? (.rightMouseDown, .rightMouseUp, .right) : (.leftMouseDown, .leftMouseUp, .left)
    for typ in [CGEventType.mouseMoved, dn, up] {
        if let e = CGEvent(mouseEventSource: evSource, mouseType: typ, mouseCursorPosition: c, mouseButton: btn) {
            e.setIntegerValueField(.eventSourceUserData, value: MARKER); e.post(tap: postTap); usleep(90_000)
        }
    }
    var fresh: [Win] = []
    let end = Date().addingTimeInterval(2)
    while Date() < end && fresh.isEmpty {
        usleep(200_000)
        fresh = windows().filter { !before.contains($0.id) && $0.bounds.width > 60 && $0.bounds.height > 40 }
    }
    // put the pointer back
    if let e = CGEvent(mouseEventSource: evSource, mouseType: .mouseMoved, mouseCursorPosition: orig, mouseButton: .left) {
        e.setIntegerValueField(.eventSourceUserData, value: MARKER); e.post(tap: postTap)
    }
    return ("clicked \(right ? "right" : "left") at \(c)", fresh)
}
func shotRegion(_ r: CGRect, _ name: String) -> String {
    let rel = "shots/\(name).png"
    try? FileManager.default.createDirectory(atPath: outDir + "/shots", withIntermediateDirectories: true)
    sh("/usr/sbin/screencapture", ["-x", "-R", "\(Int(r.minX)),\(Int(r.minY)),\(Int(r.width.rounded(.up))),\(Int(r.height.rounded(.up)))", outDir + "/" + rel])
    return rel
}

@_silgen_name("_AXUIElementGetWindow") func _AXUIElementGetWindow(_ el: AXUIElement, _ wid: UnsafeMutablePointer<CGWindowID>) -> AXError
func axWindowID(_ w: AXUIElement) -> Int { var id: CGWindowID = 0; return _AXUIElementGetWindow(w, &id) == .success ? Int(id) : 0 }
func axFirst(_ el: AXUIElement, role: String, depth: Int = 0) -> AXUIElement? {
    if depth > 8 { return nil }
    if (axCopy(el, kAXRoleAttribute as String) as? String) == role { return el }
    for k in (axCopy(el, kAXChildrenAttribute as String) as? [AXUIElement]) ?? [] { if let r = axFirst(k, role: role, depth: depth + 1) { return r } }
    return nil
}
/// Press a button of a sheet/dialog attached to `win` whose title is in `titles` (our own note's close prompt). Returns the title pressed.
func pressSheetButton(_ win: AXUIElement, titles: [String]) -> String? {
    guard let sheet = axFirst(win, role: "AXSheet") else { return nil }
    func buttons(_ el: AXUIElement, _ d: Int, _ out: inout [AXUIElement]) {
        if d > 6 { return }
        if (axCopy(el, kAXRoleAttribute as String) as? String) == "AXButton" { out.append(el) }
        for k in (axCopy(el, kAXChildrenAttribute as String) as? [AXUIElement]) ?? [] { buttons(k, d + 1, &out) }
    }
    var bs: [AXUIElement] = []; buttons(sheet, 0, &bs)
    for t in titles { for b in bs where (axCopy(b, kAXTitleAttribute as String) as? String) == t { AXUIElementPerformAction(b, kAXPressAction as CFString); return t } }
    log("sheet present, buttons: \(bs.map { axCopy($0, kAXTitleAttribute as String) as? String ?? "?" })")
    return nil
}

// MARK: - keyboard synthesis
let codes: [Character: CGKeyCode] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9, "b": 11, "q": 12, "w": 13, "e": 14, "r": 15,
    "y": 16, "t": 17, "o": 31, "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46,
    "1": 18, "2": 19, "3": 20, "4": 21, "5": 23, "6": 22, "7": 26, "8": 28, "9": 25, "0": 29,
    "-": 27, "=": 24, "[": 33, "]": 30, ";": 41, "'": 39, ",": 43, ".": 47, "/": 44, "\\": 42, "`": 50, " ": 49,
]
let shifted: [Character: Character] = ["!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6", "&": "7", "*": "8", "(": "9", ")": "0",
                                       "_": "-", "+": "=", "?": "/", ":": ";", "\"": "'", "<": ",", ">": "."]
let named: [String: CGKeyCode] = ["BS": 51, "TAB": 48, "LEFT": 123, "RIGHT": 124, "DOWN": 125, "UP": 126, "HOME": 115, "END": 119,
                                   "ESC": 53, "SP": 49, "F19": 80, "RETURN": 36]
var allowReturn = false   // Return is only ever sent into the hidden-answer dialog this harness owns
let modCode: [String: CGKeyCode] = ["SHIFT": 56, "CTRL": 59, "ALT": 58, "OPT": 58, "CMD": 55]
let modFlag: [String: CGEventFlags] = ["SHIFT": .maskShift, "CTRL": .maskControl, "ALT": .maskAlternate, "OPT": .maskAlternate, "CMD": .maskCommand]
let modDev: [String: UInt64] = ["SHIFT": 0x2, "CTRL": 0x1, "ALT": 0x20, "OPT": 0x20, "CMD": 0x8]

var curFlags: UInt64 = 0         // raw flag bits currently "held"
var capsOn = false               // synthetic Caps Lock state (flag only; see README)
var postTap: CGEventTapLocation = .cgSessionEventTap
let evSource = CGEventSource(stateID: .privateState)

func flagsNow() -> CGEventFlags { CGEventFlags(rawValue: curFlags | (capsOn ? CGEventFlags.maskAlphaShift.rawValue : 0)) }
func postKey(_ kc: CGKeyCode, down: Bool) {
    guard let ev = CGEvent(keyboardEventSource: evSource, virtualKey: kc, keyDown: down) else { return }
    ev.flags = flagsNow()
    ev.setIntegerValueField(.eventSourceUserData, value: MARKER)
    ev.post(tap: postTap)
}
func postMod(_ m: String, down: Bool) {
    if down { curFlags |= modFlag[m]!.rawValue | modDev[m]! } else { curFlags &= ~(modFlag[m]!.rawValue | modDev[m]!) }
    guard let ev = CGEvent(source: evSource) else { return }
    ev.type = .flagsChanged
    ev.setIntegerValueField(.keyboardEventKeycode, value: Int64(modCode[m]!))
    ev.flags = flagsNow()
    ev.setIntegerValueField(.eventSourceUserData, value: MARKER)
    ev.post(tap: postTap)
}
func releaseAllMods() { for m in ["SHIFT", "CTRL", "ALT", "CMD"] where curFlags & modFlag[m]!.rawValue != 0 { postMod(m, down: false) } }

enum Speed: String { case fast, slow, burst
    func gap() -> UInt32 {
        switch self { case .fast: return UInt32.random(in: 8_000...15_000)
        case .slow: return UInt32.random(in: 150_000...250_000)
        case .burst: return UInt32.random(in: 2_000...4_000) }
    }
}
func tapKey(_ kc: CGKeyCode, speed: Speed) {
    postKey(kc, down: true); usleep(UInt32.random(in: 1_500...4_000)); postKey(kc, down: false); usleep(speed.gap())
}

/// Script tokens: plain characters; `{BS}` `{TAB}` `{LEFT}` `{RIGHT}` `{SP}` `{ESC}`...; `{WAIT:ms}`;
/// `{CAPS}` toggles the synthetic Caps Lock flag; `{SHIFT}` / `{/SHIFT}` hold / release Shift;
/// chords `{CTRL+SHIFT}` (pure modifier tap), `{CTRL+SHIFT+Z}`, `{CMD+A}`.
/// Upper-case letters and shifted symbols press Shift around the key (per-letter Shift).
func typeScript(_ s: String, speed: Speed, check: () throws -> Void) throws {
    defer { releaseAllMods() }
    var n = 0
    let chars = Array(s); var i = 0
    while i < chars.count {
        let c = chars[i]; i += 1
        n += 1
        if n % 12 == 0 || c == " " { try check() }
        if c == "{" {
            var name = ""
            while i < chars.count && chars[i] != "}" { name.append(chars[i]); i += 1 }
            i += 1
            try token(name, speed: speed)
            continue
        }
        if let kc = codes[Character(c.lowercased())], c.isLetter || codes[c] != nil {
            let up = c.isUppercase
            if up && curFlags & modFlag["SHIFT"]!.rawValue == 0 { postMod("SHIFT", down: true); usleep(3_000) }
            tapKey(kc, speed: speed)
            if up && curFlags & modFlag["SHIFT"]!.rawValue != 0 && !shiftHeld { postMod("SHIFT", down: false) }
        } else if let base = shifted[c], let kc = codes[base] {
            postMod("SHIFT", down: true); usleep(3_000); tapKey(kc, speed: speed); if !shiftHeld { postMod("SHIFT", down: false) }
        } else {
            throw abort("script: unsupported char \(c)")
        }
    }
}
var shiftHeld = false
func token(_ name: String, speed: Speed) throws {
    if name.hasPrefix("WAIT:") { usleep(UInt32(Int(name.dropFirst(5)) ?? 100) * 1000); return }
    if name == "CAPS" { capsOn.toggle(); return }
    if name == "SHIFT" { shiftHeld = true; postMod("SHIFT", down: true); usleep(5_000); return }
    if name == "/SHIFT" { shiftHeld = false; postMod("SHIFT", down: false); usleep(5_000); return }
    let parts = name.split(separator: "+").map(String.init)
    if parts.count == 1, let kc = named[parts[0]] {
        if parts[0] == "RETURN" && !allowReturn { throw abort("script: RETURN not allowed here") }
        tapKey(kc, speed: speed); return
    }
    guard !parts.isEmpty else { throw abort("script: empty token") }
    // chord
    var mods: [String] = []; var key: CGKeyCode? = nil
    for p in parts {
        if modCode[p] != nil { mods.append(p) }
        else if let k = named[p] { key = k }
        else if p.count == 1, let k = codes[Character(p.lowercased())] { key = k }
        else { throw abort("script: bad chord token \(name)") }
    }
    for m in mods { postMod(m, down: true); usleep(25_000) }
    if let k = key { tapKey(k, speed: .fast) } else { usleep(60_000) }
    for m in mods.reversed() { postMod(m, down: false); usleep(25_000) }
    usleep(speed.gap())
}

// MARK: - target model
final class Target {
    let name: String
    var pid: Int32
    var winID: Int
    var allowSamePid = false         // accept any top window of the same (harness-launched) process
    var trimNL = false               // AX value carries a trailing newline (VS Code)
    var modal = false                // window sits at layer 1...10 (system dialog); accept it as "topmost"
    var readFn: () -> String?
    var clearKeys = "{CMD+A}{BS}"
    var prep: [String] = []          // keys sent after activation (e.g. {CMD+N})
    init(name: String, pid: Int32, winID: Int, read: @escaping () -> String?) { self.name = name; self.pid = pid; self.winID = winID; self.readFn = read }
    func read() -> String {
        var v = readFn() ?? "<unreadable>"
        if trimNL { while v.hasSuffix("\n") { v.removeLast() } }
        return v
    }
}
var idle0 = 0.0, idleStart = Date(), idleDetector = false
func guardFront(_ t: Target, _ what: String) throws {
    if let w = wFlag { throw abort("PHYSICAL INPUT: \(w) (\(what))", physical: true) }
    let fp = focusedAppPid()
    if fp != t.pid { throw abort("front pid \(fp) != target \(t.name) pid \(t.pid) (\(what))") }
    let topOpt = t.modal ? windows().first(where: { $0.layer <= 10 && $0.alpha > 0 && $0.bounds.width > 40 && $0.bounds.height > 20 }) : topNormalWindow()
    guard let top = topOpt else { throw abort("no normal window on screen (\(what))") }
    if top.pid != t.pid { throw abort("topmost window owner \(top.owner)#\(top.pid) != \(t.name) (\(what))") }
    if t.winID != 0 && top.id != t.winID && !t.allowSamePid { throw abort("topmost window id \(top.id) != test window \(t.winID) (\(what))") }
    if idleDetector {
        let expect = idle0 + Date().timeIntervalSince(idleStart) - 1.5
        if hidIdle() < expect { throw abort("PHYSICAL INPUT: HID idle \(hidIdle()) < expected \(expect) (\(what))", physical: true) }
    }
}
func settleRead(_ t: Target, minWait: Double = 0.35, maxWait: Double = 3.0) -> String {
    usleep(UInt32(minWait * 1_000_000))
    var last = t.read(); var stable = 0; let start = Date()
    while stable < 2 && Date().timeIntervalSince(start) < maxWait {
        usleep(120_000)
        let cur = t.read()
        if cur == last { stable += 1 } else { stable = 0; last = cur }
    }
    return last
}
func nfc(_ s: String) -> String { s.precomposedStringWithCanonicalMapping }

// MARK: - cases
struct Case {
    var id: String; var keys: String; var expect: String
    var reps = 1; var sep = " "; var speed: Speed = .fast; var smoke = false; var note = ""
}
func telexCases() -> [Case] {
    let sent = "Tieesng Vieetj raast ddepj."
    return [
        Case(id: "TX-B1", keys: "Vieejt Nam", expect: "Việt Nam", smoke: true),
        Case(id: "TX-B2", keys: "dduwowcj", expect: "được", smoke: true),
        Case(id: "TX-B3", keys: "nguowif", expect: "người", smoke: true),
        Case(id: "TX-B4", keys: "hoaf", expect: "hòa"),
        Case(id: "TX-B5", keys: "quaan", expect: "quân"),
        Case(id: "TX-B6", keys: "tieengs", expect: "tiếng"),
        Case(id: "TX-S1", keys: sent, expect: "Tiếng Việt rất đẹp.", reps: 10, smoke: true),
        Case(id: "TX-S2", keys: "dduwowcj", expect: "được", reps: 10, smoke: true, note: "D-E2E-01 pattern"),
        Case(id: "TX-S3", keys: sent, expect: "Tiếng Việt rất đẹp.", reps: 2, speed: .slow, note: "slow 150-250 ms/key"),
        Case(id: "TX-S4", keys: "dduwowcj", expect: "được", reps: 10, speed: .burst, note: "burst 2-4 ms/key stress"),
        Case(id: "TX-E1", keys: "ddeei", expect: "ddeei", note: "wrong word stays raw when typed fresh"),
        Case(id: "TX-E2", keys: "ddeei{BS}m", expect: "đêm", smoke: true),
        Case(id: "TX-E3a", keys: "ddeei{BS}{BS}{BS}ddeem", expect: "đêm", note: "delete fully, retype"),
        Case(id: "TX-E3b", keys: "ddeei{BS}{BS}{BS}ddeens", expect: "đến", note: "delete fully, retype"),
        Case(id: "TX-E4", keys: "ddeei{BS}{BS}eem", expect: "đêm", note: "delete back to first consonant, continue"),
        Case(id: "TX-E5", keys: "pass{BS}{BS}{BS}dduwowcj", expect: "được", note: "wrong English word, delete, Vietnamese"),
        Case(id: "TX-E6", keys: "dduwowcjk{BS}", expect: "được", note: "D-BS-LIT-02"),
        Case(id: "TX-E7", keys: "gifk{BS}", expect: "gì", note: "D-BS-LIT-02"),
        Case(id: "TX-E8", keys: "test{BS}", expect: "té", note: "accepted re-judge consequence"),
        Case(id: "TX-E9a", keys: "1dduwowcj", expect: "1được", smoke: true, note: "digit before a word"),
        Case(id: "TX-E9b", keys: "3giowf", expect: "3giờ", note: "digit before a word"),
        Case(id: "TX-E10", keys: "Tha {BS}s", expect: "Thá", note: "reopen across a space"),
        Case(id: "TX-E11", keys: "ddeei {BS}{BS}m", expect: "đêm", note: "through the space"),
        Case(id: "TX-E12", keys: "tieengs{BS}{BS}{BS}", expect: "ti", note: "Backspace mid-word"),
        Case(id: "TX-E13", keys: "tieengs1", expect: "tiếng1", note: "digit commits the word"),
        Case(id: "TX-E14", keys: "tieeng{BS}{BS}{BS}eeng", expect: "tiêng", note: "Backspace into the middle then retype"),
        Case(id: "TX-C1", keys: "VIEETJ", expect: "VIỆT", smoke: true, note: "Shift per letter"),
        Case(id: "TX-C2", keys: "{SHIFT}vieetj{/SHIFT}", expect: "VIỆT", note: "Shift held"),
        Case(id: "TX-C3", keys: "DDUWOWCJ", expect: "ĐƯỢC"),
        Case(id: "TX-C4", keys: "{CAPS}vieejt{CAPS}", expect: "VIỆT", note: "Caps Lock (synthetic flag)"),
        Case(id: "TX-H1", keys: "{CTRL+SHIFT}tieengs {CTRL+SHIFT}tieengs", expect: "tieengs tiếng", note: "toggle hotkey off/on (Release mode)"),
        Case(id: "TX-H2", keys: "vieejt{CTRL+SHIFT+Z}", expect: "vieejt", note: "restore word"),
        Case(id: "TX-H3", keys: "vie{LEFT}{RIGHT}et", expect: "vieet", note: "arrow-key caret move resets the word"),
    ]
}
func vniCases() -> [Case] {
    let sent = "Tie6ng1 Vie6t5 ra6t1 d9e5p."
    return [
        Case(id: "VN-B1", keys: "Vie65t Nam", expect: "Việt Nam", smoke: true),
        Case(id: "VN-B2", keys: "d9u7o7c5", expect: "được", smoke: true),
        Case(id: "VN-B3", keys: "ngu7o7i2", expect: "người", smoke: true),
        Case(id: "VN-B4", keys: "hoa2", expect: "hòa"),
        Case(id: "VN-B5", keys: "qua6n", expect: "quân"),
        Case(id: "VN-B6", keys: "tie6ng1", expect: "tiếng"),
        Case(id: "VN-S1", keys: sent, expect: "Tiếng Việt rất đẹp.", reps: 10, smoke: true),
        Case(id: "VN-S2", keys: "d9u7o7c5", expect: "được", reps: 10, smoke: true),
        Case(id: "VN-S3", keys: sent, expect: "Tiếng Việt rất đẹp.", reps: 2, speed: .slow, note: "slow 150-250 ms/key"),
        Case(id: "VN-S4", keys: "d9u7o7c5", expect: "được", reps: 10, speed: .burst, note: "burst 2-4 ms/key stress"),
        Case(id: "VN-E1", keys: "d9e6i", expect: "d9e6i", note: "wrong word stays raw when typed fresh"),
        Case(id: "VN-E2", keys: "d9e6i{BS}m", expect: "đêm", smoke: true),
        Case(id: "VN-E3a", keys: "d9e6i{BS}{BS}{BS}d9e6m", expect: "đêm"),
        Case(id: "VN-E3b", keys: "d9e6i{BS}{BS}{BS}d9e6n1", expect: "đến"),
        Case(id: "VN-E4", keys: "d9e6i{BS}{BS}e6m", expect: "đêm", note: "delete back to first consonant, continue"),
        Case(id: "VN-E8", keys: "d9e6i{BS}", expect: "đê", note: "accepted re-judge consequence"),
        Case(id: "VN-E9", keys: "2d9e6m", expect: "2đêm", smoke: true, note: "digit before a word"),
        Case(id: "VN-E10", keys: "Tha {BS}1", expect: "Thá", note: "reopen across a space"),
        Case(id: "VN-E11", keys: "d9e6i {BS}{BS}m", expect: "đêm", note: "through the space"),
        Case(id: "VN-E12", keys: "nghie6ng2{BS}", expect: "nghiền", note: "one Backspace removes the last letter (engine-confirmed: simulate_keys gives nghiền)"),
        Case(id: "VN-C1", keys: "VIE6T5", expect: "VIỆT", smoke: true, note: "Shift per letter"),
        Case(id: "VN-X1", keys: "VIE6T", expect: "VIÊT", note: "localise VN-C1: all-caps, mark digit only"),
        Case(id: "VN-X2", keys: "VIE6", expect: "VIÊ", note: "localise VN-C1"),
        Case(id: "VN-X3", keys: "VIE5", expect: "VỊE", note: "localise VN-C1: all-caps, tone digit (engine: vie5 -> vịe)"),
        Case(id: "VN-X4", keys: "Vie6T", expect: "ViêT", note: "localise VN-C1: capital coda"),
        Case(id: "VN-C4", keys: "{CAPS}vie6t5{CAPS}", expect: "VIỆT", note: "Caps Lock (synthetic flag)"),
        Case(id: "VN-H1", keys: "{CTRL+SHIFT}tie6ng1 {CTRL+SHIFT}tie6ng1", expect: "tie6ng1 tiếng", note: "toggle hotkey off/on"),
        Case(id: "VN-H2", keys: "vie6t5{CTRL+SHIFT+Z}", expect: "vie6t5", note: "restore word"),
        Case(id: "VN-H3", keys: "vie{LEFT}{RIGHT}6t5", expect: "vie6t5", note: "arrow-key caret move resets the word"),
    ]
}

// MARK: - results
var resultsHandle: FileHandle?
var counts: [String: (pass: Int, fail: Int)] = [:]
func shot(_ winID: Int, _ name: String) -> String {
    let rel = "shots/\(name).png"
    try? FileManager.default.createDirectory(atPath: outDir + "/shots", withIntermediateDirectories: true)
    sh("/usr/sbin/screencapture", ["-x", "-o", "-l", String(winID), outDir + "/" + rel])
    return rel
}
func record(_ d: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: d, options: [.sortedKeys])
    resultsHandle?.write(data); resultsHandle?.write("\n".data(using: .utf8)!)
    let tgt = d["target"] as! String
    let ok = (d["verdict"] as? String) == "PASS"
    var c = counts[tgt] ?? (0, 0); if ok { c.pass += 1 } else { c.fail += 1 }; counts[tgt] = c
    log("[\(d["target"]!)] \(d["id"]!) \(d["verdict"]!) keys=\(d["keys"]!) expected=\(d["expected"]!) actual=\(d["actual"]!)")
}

func clearTarget(_ t: Target) throws {
    try guardFront(t, "pre-clear")
    try typeScript(t.clearKeys, speed: .fast, check: { try guardFront(t, "clear") })
    var cur = settleRead(t, minWait: 0.2)
    if !cur.isEmpty && t.name != "terminal" { // retry once
        try guardFront(t, "clear-retry")
        try typeScript("{CMD+A}{BS}", speed: .fast, check: { try guardFront(t, "clear") })
        cur = settleRead(t, minWait: 0.2)
    }
    if !cur.isEmpty { throw abort("could not clear \(t.name), residue: \(cur)") }
}

func runCase(_ t: Target, _ c: Case, method: String, shotAlways: Bool = false) throws {
    try clearTarget(t)
    try guardFront(t, "pre-batch \(c.id)")
    var script = ""
    for r in 0..<c.reps { script += (r > 0 ? c.sep : "") + c.keys }
    let started = Date()
    try typeScript(script, speed: c.speed, check: { try guardFront(t, "mid-batch \(c.id)") })
    let actualRaw = settleRead(t)
    try guardFront(t, "post-batch \(c.id)")
    let dur = Date().timeIntervalSince(started)
    let expectAll = Array(repeating: c.expect, count: c.reps).joined(separator: c.sep)
    var actual = actualRaw
    if t.name == "terminal" { actual = actual.trimmingCharacters(in: .whitespaces) }
    var pass = nfc(actual) == nfc(expectAll)
    var extra: [String: Any] = [:]
    if c.reps > 1 {
        let words = nfc(actual).components(separatedBy: c.sep)
        let per = max(1, c.expect.components(separatedBy: c.sep).count)
        var good = 0
        var i = 0
        while i < words.count { if words[i..<min(words.count, i + per)].joined(separator: c.sep) == nfc(c.expect) { good += 1 }; i += per }
        extra["repsPass"] = good; extra["reps"] = c.reps
        pass = pass && good == c.reps
    }
    var ev = ""
    if (!pass || shotAlways) && t.name != "notes" { ev = shot(t.winID, "\(t.name)-\(method)-\(c.id)") }
    var d: [String: Any] = ["target": t.name, "method": method, "id": c.id, "keys": script, "expected": expectAll, "actual": actual,
                            "verdict": pass ? "PASS" : "FAIL", "speed": c.speed.rawValue, "seconds": (dur * 10).rounded() / 10,
                            "note": c.note, "evidence": ev, "nfcEqualOnly": (pass && actual != expectAll)]
    for (k, v) in extra { d[k] = v }
    record(d)
}

// MARK: - target openers
struct Setup { let target: Target; let teardown: () -> Void }

func activatePid(_ pid: Int32) {
    _ = osa("tell application \"System Events\" to set frontmost of (first process whose unix id is \(pid)) to true")
    usleep(500_000)
}
func pidOf(bundle: String) -> Int32? { NSRunningApplication.runningApplications(withBundleIdentifier: bundle).first?.processIdentifier }

func openTextEdit(plain: Bool) throws -> Setup {
    let wasRunning = pidOf(bundle: "com.apple.TextEdit") != nil
    let id = Int(osa("tell application \"TextEdit\"\nactivate\nset d to make new document\nreturn id of front window\nend tell"))
    guard let wid = id else { throw abort("TextEdit: cannot create document") }
    usleep(800_000)
    guard let pid = pidOf(bundle: "com.apple.TextEdit") else { throw abort("TextEdit not running") }
    activatePid(pid)
    let menuTitle = osa("tell application \"System Events\" to tell process \"TextEdit\" to return name of menu item 1 of menu \"Format\" of menu bar 1 whose name contains \"Plain\" or name contains \"Rich\"")
    log("TextEdit window \(wid) pid \(pid); format menu hint: \(menuTitle)")
    let isPlain = osa("tell application \"System Events\" to tell process \"TextEdit\" to return exists menu item \"Make Rich Text\" of menu \"Format\" of menu bar 1") == "true"
    if plain && !isPlain {
        _ = osa("tell application \"System Events\" to tell process \"TextEdit\" to click menu item \"Make Plain Text\" of menu \"Format\" of menu bar 1")
        usleep(500_000)
        // a confirmation sheet may appear when the doc has content; it is empty so none expected
    }
    let nowPlain = osa("tell application \"System Events\" to tell process \"TextEdit\" to return exists menu item \"Make Rich Text\" of menu \"Format\" of menu bar 1") == "true"
    if plain != nowPlain { throw abort("TextEdit: wanted plain=\(plain) got \(nowPlain)") }
    let t = Target(name: plain ? "textedit-plain" : "textedit-rich", pid: pid, winID: wid) {
        let s = osa("tell application \"TextEdit\" to return text of document of (first window whose id is \(wid))")
        return s
    }
    // the front window must be ours
    activatePid(pid)
    return Setup(target: t, teardown: {
        _ = osa("tell application \"TextEdit\" to close (document of (first window whose id is \(wid))) saving no")
        usleep(400_000)
        if !wasRunning, osa("tell application \"TextEdit\" to return count of windows") == "0" { _ = osa("tell application \"TextEdit\" to quit saving no") }
    })
}

func openNotes() throws -> Setup {
    let wasRunning = pidOf(bundle: "com.apple.Notes") != nil
    _ = osa("tell application \"Notes\" to activate")
    usleep(2_000_000)
    guard let pid = pidOf(bundle: "com.apple.Notes") else { throw abort("Notes not running") }
    let before = Set(osa("tell application \"Notes\" to return id of every note").components(separatedBy: ", "))
    activatePid(pid)
    guard let top = topNormalWindow(), top.pid == pid else { throw abort("Notes window not frontmost") }
    let tmp = Target(name: "notes", pid: pid, winID: top.id) { nil }
    try guardFront(tmp, "notes pre-new")
    try typeScript("{CMD+N}", speed: .fast, check: {})
    var fresh: [String] = []
    for _ in 0..<25 {
        usleep(500_000)
        let after = osa("tell application \"Notes\" to return id of every note").components(separatedBy: ", ")
        fresh = after.filter { !before.contains($0) && !$0.isEmpty }
        if !fresh.isEmpty { break }
    }
    guard fresh.count == 1, let nid = fresh.first else {
        if !wasRunning { _ = osa("tell application \"Notes\" to quit") }
        throw abort("Notes: expected exactly one new note, got \(fresh)")
    }
    guard let top2 = topNormalWindow(), top2.pid == pid else { throw abort("Notes window lost") }
    let t = Target(name: "notes", pid: pid, winID: top2.id) {
        osa("tell application \"Notes\" to return plaintext of note id \"\(nid)\"")
    }
    log("Notes: created note \(nid)")
    return Setup(target: t, teardown: {
        _ = osa("tell application \"Notes\" to delete note id \"\(nid)\"")
        log("Notes: deleted note \(nid) (moved to Recently Deleted, NOT permanently deleted)")
        usleep(500_000)
        if !wasRunning { _ = osa("tell application \"Notes\" to quit") }
    })
}

func openStickies() throws -> Setup {
    let wasRunning = pidOf(bundle: "com.apple.Stickies") != nil
    _ = osa("tell application \"Stickies\" to activate")
    usleep(1_500_000)
    guard let pid = pidOf(bundle: "com.apple.Stickies") else { throw abort("Stickies not running") }
    activatePid(pid)
    let winsBefore = Set(windows().filter { $0.pid == pid && $0.layer == 0 }.map { $0.id })
    guard let top = topNormalWindow(), top.pid == pid else { throw abort("Stickies not frontmost") }
    let tmp = Target(name: "stickies", pid: pid, winID: top.id) { nil }
    try guardFront(tmp, "stickies pre-new")
    try typeScript("{CMD+N}", speed: .fast, check: {})
    usleep(1_000_000)
    let fresh = windows().filter { $0.pid == pid && $0.layer == 0 && !winsBefore.contains($0.id) }
    guard fresh.count == 1, let w = fresh.first else { throw abort("Stickies: expected one new note window, got \(fresh.count)") }
    let t = Target(name: "stickies", pid: pid, winID: w.id) { axFocusedValue(pid: pid) }
    return Setup(target: t, teardown: {
        // only our note: empty it, close it, accept "delete/don't save" on the prompt of OUR window
        do {
            activatePid(pid)
            try guardFront(t, "stickies teardown")
            try typeScript("{CMD+A}{BS}", speed: .fast, check: {})
            usleep(300_000)
            try guardFront(t, "stickies teardown close")
            try typeScript("{CMD+W}", speed: .fast, check: {})
            usleep(800_000)
            for win in (axCopy(AXUIElementCreateApplication(pid), kAXWindowsAttribute as String) as? [AXUIElement]) ?? [] where axWindowID(win) == w.id {
                if let b = pressSheetButton(win, titles: ["Delete", "Don\u{2019}t Save", "Don't Save", "Discard"]) { log("Stickies: pressed '\(b)' on our note's close sheet") }
            }
        } catch { log("Stickies teardown aborted: \(error)") }
        usleep(800_000)
        let stillThere = windows().contains { $0.id == w.id }
        log("Stickies: our note window \(w.id) \(stillThere ? "STILL OPEN" : "closed")")
        if !wasRunning && !stillThere { _ = osa("tell application \"Stickies\" to quit") }
    })
}

var children: [Process] = []
func launch(_ path: String, _ args: [String], env extra: [String: String] = [:], logName: String) throws -> Process {
    let p = Process(); p.executableURL = URL(fileURLWithPath: path); p.arguments = args
    var e = ProcessInfo.processInfo.environment; for (k, v) in extra { e[k] = v }; p.environment = e
    FileManager.default.createFile(atPath: outDir + "/" + logName, contents: nil)
    let fh = FileHandle(forWritingAtPath: outDir + "/" + logName); p.standardOutput = fh; p.standardError = fh
    try p.run(); children.append(p)
    log("LAUNCH pid=\(p.processIdentifier) \(path) \(args.prefix(3).joined(separator: " "))...")
    return p
}
func killChild(_ p: Process) {
    if p.isRunning { log("KILL pid=\(p.processIdentifier) (launched by this harness)"); p.terminate(); usleep(600_000); if p.isRunning { kill(p.processIdentifier, SIGKILL) } }
}
func waitForWindow(pid: Int32, timeout: Double = 20) -> Win? {
    let end = Date().addingTimeInterval(timeout)
    while Date() < end { if let w = windows().first(where: { $0.pid == pid && $0.layer == 0 && $0.bounds.width > 100 }) { return w }; usleep(300_000) }
    return nil
}

func openChrome(omnibox: Bool) throws -> Setup {
    let tmp = env["E2E_TMP"] ?? NSTemporaryDirectory()
    let html = "data:text/html,<title>vkbe2e</title><textarea id=t autofocus rows=6 cols=70></textarea><br><input id=i size=70>"
    let p = try launch("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                       ["--user-data-dir=\(tmp)/chrome", "--no-first-run", "--no-default-browser-check", "--force-renderer-accessibility",
                        "--disable-search-engine-choice-screen", "--disable-sync", "--new-window", html], logName: "chrome.log")
    guard let w = waitForWindow(pid: p.processIdentifier) else { killChild(p); throw abort("Chrome window did not appear") }
    sleep(2)
    activatePid(p.processIdentifier)
    // tell Chrome an AX client is here
    AXUIElementSetAttributeValue(AXUIElementCreateApplication(p.processIdentifier), "AXManualAccessibility" as CFString, kCFBooleanTrue)
    let win = windows().first { $0.pid == p.processIdentifier && $0.layer == 0 && $0.bounds.width > 100 } ?? w
    let t = Target(name: omnibox ? "chrome-omnibox" : "chrome-textarea", pid: p.processIdentifier, winID: win.id) { axFocusedValue(pid: p.processIdentifier) }
    t.allowSamePid = omnibox
    return Setup(target: t, teardown: { killChild(p) })
}

func openVSCode() throws -> Setup {
    let tmp = env["E2E_TMP"] ?? NSTemporaryDirectory()
    let ud = tmp + "/vscode-user", ex = tmp + "/vscode-ext"
    try FileManager.default.createDirectory(atPath: ud + "/User", withIntermediateDirectories: true)
    try FileManager.default.createDirectory(atPath: ex, withIntermediateDirectories: true)
    let settings = """
    {"workbench.startupEditor":"none","editor.quickSuggestions":{"other":"off","comments":"off","strings":"off"},
     "editor.suggestOnTriggerCharacter":false,"editor.acceptSuggestionOnEnter":"off","editor.wordBasedSuggestions":"off",
     "editor.parameterHints.enabled":false,"editor.inlineSuggest.enabled":false,"editor.accessibilitySupport":"on",
     "files.autoSave":"off","workbench.tips.enabled":false,"update.mode":"none","telemetry.telemetryLevel":"off",
     "security.workspace.trust.enabled":false,"workbench.welcomePage.walkthroughs.openOnInstall":false,
     "window.restoreWindows":"none","workbench.enableExperiments":false,"extensions.autoUpdate":false,"editor.autoClosingBrackets":"never"}
    """
    try settings.write(toFile: ud + "/User/settings.json", atomically: true, encoding: .utf8)
    let p = try launch("/Applications/Visual Studio Code.app/Contents/MacOS/Code",
                       ["--user-data-dir=\(ud)", "--extensions-dir=\(ex)", "--disable-extensions", "--disable-workspace-trust",
                        "--skip-release-notes", "--skip-welcome", "--new-window", "--force-renderer-accessibility"], logName: "vscode.log")
    guard waitForWindow(pid: p.processIdentifier, timeout: 40) != nil else { killChild(p); throw abort("VS Code window did not appear") }
    sleep(5)
    activatePid(p.processIdentifier)
    AXUIElementSetAttributeValue(AXUIElementCreateApplication(p.processIdentifier), "AXManualAccessibility" as CFString, kCFBooleanTrue)
    let win0 = windows().first { $0.pid == p.processIdentifier && $0.layer == 0 && $0.bounds.width > 100 }
    let t = Target(name: "vscode", pid: p.processIdentifier, winID: win0?.id ?? 0) { axFocusedValue(pid: p.processIdentifier) }
    t.trimNL = true
    try guardFront(t, "vscode pre-new")
    try typeScript("{CMD+N}", speed: .fast, check: {})
    sleep(2)
    return Setup(target: t, teardown: { killChild(p) })
}

/// Terminal without Apple events: a .command file of ours runs `cat > /dev/null` in a NEW Terminal window.
func openTerminal() throws -> Setup {
    let tmp = env["E2E_TMP"] ?? NSTemporaryDirectory()
    let script = tmp + "/vkbe2e-cat.command"
    try "#!/bin/zsh\nclear\necho VKBE2E-READY\ncat > /dev/null\n".write(toFile: script, atomically: true, encoding: .utf8)
    chmod(script, 0o755)
    guard let pid = pidOf(bundle: "com.apple.Terminal") else { throw abort("Terminal not running; refusing to start the owner's app cold") }
    let before = Set(windows().filter { $0.pid == pid }.map { $0.id })
    sh("/usr/bin/open", ["-a", "Terminal", script])
    sleep(3)
    activatePid(pid)
    let fresh = windows().filter { $0.pid == pid && $0.layer == 0 && !before.contains($0.id) && $0.bounds.width > 100 }
    guard fresh.count == 1, let w = fresh.first else { throw abort("Terminal: expected 1 new window, got \(fresh.count)") }
    // AX: the new window's text area
    func termText() -> String? {
        let app = AXUIElementCreateApplication(pid)
        guard let wins = axCopy(app, kAXWindowsAttribute as String) as? [AXUIElement] else { return nil }
        for win in wins {
            var wid: CGWindowID = 0
            if _AXUIElementGetWindow(win, &wid) == .success, Int(wid) == w.id {
                func find(_ el: AXUIElement, _ d: Int) -> String? {
                    if d > 8 { return nil }
                    if (axCopy(el, kAXRoleAttribute as String) as? String) == "AXTextArea", let s = axCopy(el, kAXValueAttribute as String) as? String { return s }
                    for k in (axCopy(el, kAXChildrenAttribute as String) as? [AXUIElement]) ?? [] { if let r = find(k, d + 1) { return r } }
                    return nil
                }
                return find(win, 0)
            }
        }
        return nil
    }
    let t = Target(name: "terminal", pid: pid, winID: w.id) {
        guard let s = termText() else { return nil }
        // text after the READY banner, soft-wrapped rows joined, trailing blanks trimmed
        let lines = s.components(separatedBy: "\n")
        guard let idx = lines.lastIndex(where: { $0.contains("VKBE2E-READY") }) else { return nil }
        return lines[(idx + 1)...].map { $0.replacingOccurrences(of: "\\s+$", with: "", options: .regularExpression) }.joined()
    }
    t.clearKeys = "{CTRL+U}"
    if t.read() == "<unreadable>" { log("Terminal: AX text not readable") }
    return Setup(target: t, teardown: {
        do {
            try guardFront(t, "terminal teardown")
            try typeScript("{CTRL+C}", speed: .fast, check: {})
        } catch { log("Terminal teardown: \(error)") }
        usleep(700_000)
        // close only our window by its CG id via AX
        let app = AXUIElementCreateApplication(pid)
        for win in (axCopy(app, kAXWindowsAttribute as String) as? [AXUIElement]) ?? [] {
            var wid: CGWindowID = 0
            if _AXUIElementGetWindow(win, &wid) == .success, Int(wid) == w.id,
               let cb = axCopy(win, kAXCloseButtonAttribute as String) {
                AXUIElementPerformAction(cb as! AXUIElement, kAXPressAction as CFString)
            }
        }
        try? FileManager.default.removeItem(atPath: script)
    })
}
// MARK: - main flow per target
var vkPidGlobal: Int32 = 0
var perappRaw: Set<String> = []   // targets whose per-app rule disables VietKB: expect raw text
func selectCases(_ method: String, smokeOnly: Bool, only: Set<String>? = nil) -> [Case] {
    var cs = method == "vni" ? vniCases() : telexCases()
    if smokeOnly { cs = cs.filter { $0.smoke } }
    if let o = only { cs = cs.filter { o.contains($0.id) } }
    return cs
}

func runTarget(_ name: String, method: String, only: Set<String>?) throws {
    log("==== target \(name) / \(method) ====")
    let setup: Setup
    switch name {
    case "textedit-plain": setup = try openTextEdit(plain: true)
    case "textedit-rich": setup = try openTextEdit(plain: false)
    case "notes": setup = try openNotes()
    case "stickies": setup = try openStickies()
    case "chrome-textarea": setup = try openChrome(omnibox: false)
    case "chrome-omnibox": setup = try openChrome(omnibox: true)
    case "vscode": setup = try openVSCode()
    case "terminal": setup = try openTerminal()
    case "gui-toggle": setup = try openTextEdit(plain: true)
    default: throw abort("unknown target \(name)")
    }
    let t = setup.target
    var err: Error? = nil
    do {
        activatePid(t.pid)
        if name == "chrome-omnibox" {
            // focus the address bar with Cmd+L, only after the front check
            try guardFront(t, "omnibox focus")
            try typeScript("{CMD+L}", speed: .fast, check: {})
            usleep(400_000)
            t.clearKeys = "{CMD+A}{BS}"
        }
        try guardFront(t, "first batch")
        if name == "gui-toggle" { try guiToggle(vkPid: vkPidGlobal, t: t); setup.teardown(); return }
        let full = name.hasPrefix("textedit")
        var cs = selectCases(method, smokeOnly: !full, only: only)
        if perappRaw.contains(name) {
            cs = [Case(id: "PA-1", keys: "dduwowcj", expect: "dduwowcj", note: "per-app rule enabled=false -> raw"),
                  Case(id: "PA-2", keys: "Vieejt Nam", expect: "Vieejt Nam", note: "per-app rule enabled=false -> raw")]
        }
        if name == "terminal" { cs = cs.map { var c = $0; if c.reps > 3 { c.reps = 3 }; return c } }
        if name == "chrome-omnibox" { cs = cs.filter { ["B1", "B2", "E2"].contains(String($0.id.split(separator: "-")[1])) } }
        var first = true
        for c in cs {
            do {
                try runCase(t, c, method: method, shotAlways: first); first = false
                if !idleDetector {
                    let elapsed = Date().timeIntervalSince(idleStart)
                    if hidIdle() >= idle0 + elapsed - 1.5 { idleDetector = true; log("idle detector enabled: synthetic session-tap events do not reset HID idle (idle \(Int(hidIdle()))s)") }
                    else { log("idle detector NOT usable: HID idle \(hidIdle()) < expected \(idle0 + elapsed); relying on the event-tap watcher") }
                }
            }
            catch let a as Abort {
                if a.physical { throw a }
                log("CASE ABORT \(c.id): \(a.why)")
                record(["target": t.name, "method": method, "id": c.id, "keys": c.keys, "expected": c.expect, "actual": "", "verdict": "ABORT",
                        "note": a.why, "evidence": ""])
                throw a
            }
        }
    } catch { err = error }
    setup.teardown()
    if let e = err { throw e }
}

// MARK: - subcommands
func inputSourceID() -> String {
    let cur = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
    if let p = TISGetInputSourceProperty(cur, kTISPropertyInputSourceID) { return Unmanaged<CFString>.fromOpaque(p).takeUnretainedValue() as String }
    return "?"
}
func setInputSource(_ id: String) -> Bool {
    let filter = [kTISPropertyInputSourceID as String: id] as CFDictionary
    guard let list = TISCreateInputSourceList(filter, false)?.takeRetainedValue() as? [TISInputSource], let src = list.first else { return false }
    return TISSelectInputSource(src) == noErr
}

func calibrateIdle() {
    idle0 = hidIdle(); idleStart = Date(); idleDetector = false
}

func vietkbWindows(pid: Int32) -> [Win] { windows().filter { $0.pid == pid } }

var guiStateRestored = true   // false when a GUI check left the engine toggled: GUI-5 then refuses to run
/// GUI smoke: settings window, status item, menu. Pure screenshots + AX text; no typing except one Escape to close the menu.
func guiSmoke(vkPid: Int32) throws {
    log("==== GUI smoke (VietKB pid \(vkPid)) ====")
    var rows: [[String: Any]] = []
    let ws = vietkbWindows(pid: vkPid)
    for w in ws { log("VietKB window id=\(w.id) layer=\(w.layer) name=\(w.name) bounds=\(w.bounds)") }
    let settings = ws.first { $0.layer == 0 && $0.bounds.width > 200 }
    if let s = settings {
        let p = shot(s.id, "gui-settings-window")
        let texts = axTexts(pid: vkPid)
        try? texts.joined(separator: "\n").write(toFile: outDir + "/gui-settings-ax.txt", atomically: true, encoding: .utf8)
        let banner = texts.contains { $0.lowercased().contains("accessibility") || $0.lowercased().contains("trợ năng") }
        rows.append(["id": "GUI-1", "item": "settings window opens", "verdict": "PASS", "evidence": p, "note": "bounds \(s.bounds)"])
        rows.append(["id": "GUI-2", "item": "Accessibility banner NOT shown (permission granted)", "verdict": banner ? "REVIEW" : "PASS",
                     "evidence": "gui-settings-ax.txt + " + p, "note": banner ? "AX text mentions Accessibility/Trợ năng: verify by screenshot" : "no Accessibility text in AX tree"])
    } else {
        rows.append(["id": "GUI-1", "item": "settings window opens", "verdict": "FAIL", "evidence": "", "note": "no VietKB window on screen"])
    }
    // status item: AX extras menu bar (not a CGWindow on this macOS)
    if let (item, fr) = statusItem(pid: vkPid) {
        let p = shotRegion(fr.insetBy(dx: -2, dy: -1), "gui-statusitem-on")
        let desc = (axCopy(item, kAXDescriptionAttribute as String) as? String ?? "") + " / " + (axCopy(item, kAXTitleAttribute as String) as? String ?? "")
        rows.append(["id": "GUI-3", "item": "menu-bar status item present", "verdict": "PASS", "evidence": p, "note": "AXExtrasMenuBar item frame \(fr), desc '\(desc)'"])
        // M10: a LEFT click toggles V/E; the menu opens on right click only. Never left-click here to "try" the menu.
        let (rc, fresh) = clickStatusItem(pid: vkPid, frame: fr, right: true)
        if let m = fresh.max(by: { $0.bounds.height < $1.bounds.height }) {
            let mp = shot(m.id, "gui-statusitem-menu")
            rows.append(["id": "GUI-4", "item": "menu-bar menu opens", "verdict": "PASS", "evidence": mp, "note": "\(rc); menu window id \(m.id) owner \(m.owner) layer \(m.layer) \(m.bounds)"])
            // close: Escape goes to the menu being tracked; it is the only key sent here
            postKey(53, down: true); usleep(4_000); postKey(53, down: false)
            usleep(600_000)
            if windows().contains(where: { $0.id == m.id }) { log("menu window \(m.id) still on screen after Escape") }
        } else {
            rows.append(["id": "GUI-4", "item": "menu-bar menu opens", "verdict": "FAIL", "evidence": "", "note": "\(rc), no new window appeared"])
        }
        usleep(400_000)
        // GUI-4b (M10): left click flips V/E, a second left click flips back. The icon crop hash is the state probe.
        func iconHash(_ name: String) -> String {
            let rel = shotRegion((statusItem(pid: vkPid)?.1 ?? fr).insetBy(dx: -2, dy: -1), name)
            return sh("/usr/bin/shasum", ["-a", "256", outDir + "/" + rel]).out.split(separator: " ").first.map(String.init) ?? ""
        }
        let h0 = iconHash("gui-statusitem-left-0-start")
        _ = clickStatusItem(pid: vkPid, frame: fr, right: false); usleep(900_000)
        let h1 = iconHash("gui-statusitem-left-1-after-click")
        _ = clickStatusItem(pid: vkPid, frame: fr, right: false); usleep(900_000)
        let h2 = iconHash("gui-statusitem-left-2-after-second-click")
        var restored = h2 == h0
        var fixNote = ""
        if !restored { // restore the original state before anything else runs, and say so
            _ = clickStatusItem(pid: vkPid, frame: fr, right: false); usleep(900_000)
            let h3 = iconHash("gui-statusitem-left-3-restore")
            restored = h3 == h0; fixNote = "; extra click needed to restore, restored=\(restored)"
        }
        let flips = !h0.isEmpty && h1 != h0 && h2 == h0
        rows.append(["id": "GUI-4b", "item": "left click on the status item toggles V/E and a second left click toggles back (M10)", "verdict": flips ? "PASS" : "FAIL",
                     "evidence": "shots/gui-statusitem-left-0-start.png / -1-after-click.png / -2-after-second-click.png",
                     "note": "icon changed after click 1: \(h1 != h0), back after click 2: \(h2 == h0)\(fixNote)"])
        rows.append(["id": "GUI-4c", "item": "original V/E state restored after the left-click checks (asserted before GUI-5)", "verdict": restored ? "PASS" : "FAIL",
                     "evidence": "", "note": "icon equals the start icon: \(restored)"])
        guiStateRestored = restored
    } else {
        rows.append(["id": "GUI-3", "item": "menu-bar status item present", "verdict": "FAIL", "evidence": "", "note": "no AXExtrasMenuBar item"])
    }
    for r in rows {
        var d = r; d["target"] = "gui"; d["method"] = "-"; d["keys"] = r["item"]; d["expected"] = "present"; d["actual"] = r["verdict"] as? String ?? ""
        record(d)
    }
}

/// Toggle the engine with the real hotkey while a test target is open, screenshot the status item in both states.
func guiToggle(vkPid: Int32, t: Target) throws {
    guard let (_, fr) = statusItem(pid: vkPid) else { log("no status item for toggle shot"); return }
    let a = shotRegion(fr.insetBy(dx: -2, dy: -1), "gui-statusitem-V-before-toggle")
    func hh(_ rel: String) -> String { sh("/usr/bin/shasum", ["-a", "256", outDir + "/" + rel]).out.split(separator: " ").first.map(String.init) ?? "" }
    // precondition: engine ON (V). GUI-3 stored the ON icon; a different icon means an earlier check left it toggled (D-MAC-48 root cause)
    let onRef = outDir + "/shots/gui-statusitem-on.png"
    if !guiStateRestored || (FileManager.default.fileExists(atPath: onRef) && hh(a) != hh("shots/gui-statusitem-on.png")) {
        record(["target": "gui", "method": "-", "id": "GUI-5", "keys": "precondition", "expected": "engine ON (V) before the toggle check",
                "actual": "icon differs from the start icon: an earlier check left the state toggled", "verdict": "FAIL", "evidence": a,
                "note": "refused to send Ctrl+Shift; state not trusted (no cascade)"])
        return
    }
    try guardFront(t, "gui toggle")
    try typeScript("{CTRL+SHIFT}", speed: .fast, check: {})
    usleep(900_000)
    let fr2 = statusItem(pid: vkPid)?.1 ?? fr
    let b = shotRegion(fr2.insetBy(dx: -2, dy: -1), "gui-statusitem-E-after-toggle")
    try guardFront(t, "gui toggle typing")
    try typeScript("tieengs", speed: .fast, check: {})
    let offText = settleRead(t)
    try guardFront(t, "gui toggle 2")
    try typeScript("{CTRL+SHIFT}", speed: .fast, check: {})
    usleep(900_000)
    let c = shotRegion((statusItem(pid: vkPid)?.1 ?? fr).insetBy(dx: -2, dy: -1), "gui-statusitem-V-after-second-toggle")
    func h(_ rel: String) -> String { hh(rel) }
    let differs = h(a) != h(b) && h(a) == h(c)
    record(["target": "gui", "method": "-", "id": "GUI-5", "keys": "{CTRL+SHIFT} x2 with 'tieengs' typed while off", "expected": "icon image changes V -> E -> V; text raw while off",
            "actual": "icon differs: \(h(a) != h(b)), restored: \(h(a) == h(c)); text while off: \(offText)", "verdict": (differs && offText == "tieengs") ? "PASS" : "FAIL",
            "evidence": "\(a) / \(b) / \(c)", "note": "owner to eyeball V/E glyphs"])
}

/// Hidden-answer dialog: typed text must stay raw (secure input).
func secureCase(vkPid: Int32, method: String) throws {
    log("==== secure field (hidden answer dialog) ====")
    let keys = method == "vni" ? "d9u7o7c5 Vie65t" : "dduwowcj Vieejt"
    let before = Set(windows().map { $0.id })
    let p = Process(); p.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
    p.arguments = ["-e", "set r to display dialog \"vkbe2e secure-field test: type, then Return\" default answer \"\" with hidden answer buttons {\"Cancel\", \"OK\"} default button \"OK\"\nreturn text returned of r"]
    let pipe = Pipe(); p.standardOutput = pipe; p.standardError = pipe
    try p.run()
    defer { if p.isRunning { log("KILL pid=\(p.processIdentifier) osascript dialog (launched by this harness)"); p.terminate() } }
    // wait for the dialog window
    var win: Win? = nil
    let end = Date().addingTimeInterval(10)
    while Date() < end && win == nil {
        win = windows().first { !before.contains($0.id) && $0.bounds.width > 150 && $0.bounds.width < 800 && $0.bounds.height > 80 }
        usleep(300_000)
    }
    if let w = win { log("secure dialog window: id \(w.id) pid \(w.pid) owner \(w.owner) layer \(w.layer) \(w.bounds)") }
    guard let w = win else { record(["target": "secure", "method": method, "id": "SEC-1", "keys": keys, "expected": "raw", "actual": "dialog window not found", "verdict": "ABORT", "note": "", "evidence": ""]); return }
    activatePid(w.pid)
    let t = Target(name: "secure", pid: w.pid, winID: w.id) { nil }
    t.modal = true
    try guardFront(t, "secure pre")
    try typeScript(keys, speed: .fast, check: { try guardFront(t, "secure mid") })
    let ev = shot(w.id, "secure-dialog-\(method)")
    try guardFront(t, "secure post")
    allowReturn = true
    defer { allowReturn = false }
    try typeScript("{RETURN}", speed: .fast, check: {})
    let end2 = Date().addingTimeInterval(8)
    while p.isRunning && Date() < end2 { usleep(100_000) }
    let answer = String(data: pipe.fileHandleForReading.availableData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
    let pass = answer == keys
    record(["target": "secure", "method": method, "id": "SEC-1", "keys": keys, "expected": keys, "actual": answer, "verdict": pass ? "PASS" : "FAIL",
            "note": "hidden-answer dialog: IsSecureEventInputEnabled while open must make VietKB pass keys through", "evidence": ev])
}

// MARK: - report
func report() {
    guard let data = FileManager.default.contents(atPath: outDir + "/results.jsonl"), let s = String(data: data, encoding: .utf8) else { print("no results"); return }
    var rows: [[String: Any]] = []
    for l in s.split(separator: "\n") {
        if var o = try? JSONSerialization.jsonObject(with: Data(l.utf8)) as? [String: Any] {
            // re-judge text cases (v1 harness mis-counted multi-word repetitions): verdict = NFC(actual) == NFC(expected)
            if let e = o["expected"] as? String, let a = o["actual"] as? String, o["method"] as? String != "-", o["verdict"] as? String != "ABORT" {
                let ok = nfc(a.trimmingCharacters(in: .whitespaces)) == nfc(e.trimmingCharacters(in: .whitespaces))
                if ok != ((o["verdict"] as? String) == "PASS") { o["note"] = "\(o["note"] ?? "") [re-judged by report: was \(o["verdict"] ?? "")]" }
                o["verdict"] = ok ? "PASS" : "FAIL"
            }
            rows.append(o)
        }
    }
    var md = "| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |\n|---|---|---|---|---|---|---|---|---|\n"
    func esc(_ x: Any?) -> String { "`" + String(describing: x ?? "").replacingOccurrences(of: "|", with: "\\|").replacingOccurrences(of: "\n", with: "⏎") + "`" }
    for r in rows { md += "| \(r["target"] ?? "") | \(r["method"] ?? "") | \(r["id"] ?? "") | \(r["cases"] ?? "") | \(esc(r["keys"])) | \(esc(r["expected"])) | \(esc(r["actual"])) | \(r["verdict"] ?? "") | \(r["evidence"] ?? "") |\n" }
    var tally: [String: (Int, Int, Int)] = [:]
    for r in rows { let k = "\(r["target"] ?? "")"; var c = tally[k] ?? (0, 0, 0); switch r["verdict"] as? String { case "PASS": c.0 += 1; case "FAIL": c.1 += 1; default: c.2 += 1 }; tally[k] = c }
    md += "\n| target | PASS | FAIL | other |\n|---|---|---|---|\n"
    for (k, v) in tally.sorted(by: { $0.key < $1.key }) { md += "| \(k) | \(v.0) | \(v.1) | \(v.2) |\n" }
    try? md.write(toFile: outDir + "/results.md", atomically: true, encoding: .utf8)
    print(md)
}

// MARK: - entry
let args = Array(CommandLine.arguments.dropFirst())
guard let cmd = args.first else { print("usage: vkbe2e preflight|inputsource|run|report"); exit(2) }
func opt(_ name: String) -> String? { if let i = args.firstIndex(of: "--" + name), i + 1 < args.count { return args[i + 1] }; return nil }
try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)
_ = NSApplication.shared          // connect to the window server first: system-wide AX focus queries fail in a CLI process until it has
_ = windows()

switch cmd {
case "preflight":
    print("ax_trusted=\(AXIsProcessTrusted()) listen=\(CGPreflightListenEventAccess()) post=\(CGPreflightPostEventAccess()) screen=\(CGPreflightScreenCaptureAccess())")
    print("hid_idle_seconds=\(hidIdle())")
    print("input_source=\(inputSourceID())")
    print("secure_input=\(IsSecureEventInputEnabled())")
    exit(0)
case "inputsource":
    if let id = args.dropFirst().first { print(setInputSource(id) ? "set \(id)" : "FAILED to set \(id)"); usleep(500_000) }
    print(inputSourceID()); exit(0)
case "report":
    report(); exit(0)
case "appsuite":   // CAP-1/2/3/4/14: launch the dev-channel bundle via LaunchServices and drive/inspect it by AX only. See appsuite.swift
    if !FileManager.default.fileExists(atPath: outDir + "/results.jsonl") { FileManager.default.createFile(atPath: outDir + "/results.jsonl", contents: nil, attributes: nil) }
    resultsHandle = FileHandle(forWritingAtPath: outDir + "/results.jsonl"); _ = try? resultsHandle?.seekToEnd()
    guard AXIsProcessTrusted(), CGPreflightScreenCaptureAccess() else { log("ABORT: Accessibility / Screen Recording grant missing for this shell"); exit(3) }
    let idleNow = hidIdle()
    log("appsuite idle=\(Int(idleNow))s")
    if env["SKIP_IDLE_GATE"] == nil && idleNow < 60 { log("ABORT: owner not idle (\(idleNow) s < 60 s). Nothing launched."); exit(4) }
    exit(runAppSuite(app: opt("app") ?? "", phases: Set((opt("phases") ?? "main,badlang,hidden,single,update,quit").split(separator: ",").map(String.init)), withTextEdit: !args.contains("--no-textedit")))
case "docktile":   // read-only: is there a Dock tile for this bundle path? (AX read of the Dock, no action)
    print(dockTile(appPath: args.dropFirst().first ?? "").map { "tile \($0.1)" } ?? "no tile"); exit(0)
case "appdump":   // recon: launch the dev bundle (LaunchServices or --direct) with a temp HOME, dump AX/Dock/status-item state, reap it
    guard hidIdle() >= 60 else { print("owner not idle"); exit(4) }
    do { try appDump(app: opt("app") ?? "", home: opt("home") ?? NSTemporaryDirectory() + "vkb-dump-home", mode: args.contains("--direct") ? .direct : .launchServices) }
    catch { print("appdump failed: \(error)") }
    exit(0)
case "axdump":   // debugging aid: click the status item of a pid (left or right) and report the windows that appear
    let pid = Int32(args.dropFirst().first ?? "0") ?? 0
    let right = args.contains("right")
    if let (_, fr) = statusItem(pid: pid) {
        print("item", fr)
        let (rc, fresh) = clickStatusItem(pid: pid, frame: fr, right: right)
        print(rc)
        for w in fresh { print("new window", w.id, w.owner, "layer", w.layer, w.bounds) }
    } else { print("no status item") }
    exit(0)
case "stickies-cleanup":   // remove a leftover test note (text matches a test string) from Stickies; touches no other note
    guard hidIdle() >= 60 else { print("owner not idle"); exit(4) }
    guard let pid = pidOf(bundle: "com.apple.Stickies") else { print("Stickies not running"); exit(0) }
    guard startWatcher() else { print("watcher failed"); exit(3) }
    usleep(400_000)
    let mine: Set<String> = ["Việt Nam", "được", "đêm", "Vieejt Nam", "tieengs"]
    var hits: [(AXUIElement, Int)] = []
    for win in (axCopy(AXUIElementCreateApplication(pid), kAXWindowsAttribute as String) as? [AXUIElement]) ?? [] {
        if let ta = axFirst(win, role: "AXTextArea"), let v = axCopy(ta, kAXValueAttribute as String) as? String, mine.contains(v.trimmingCharacters(in: .whitespacesAndNewlines).precomposedStringWithCanonicalMapping) { hits.append((win, axWindowID(win))) }
        else if let id = env["STK_ID"], let doc = axCopy(win, kAXDocumentAttribute as String) as? String, doc.contains(id) { hits.append((win, axWindowID(win))) }
    }
    print("matching test notes: \(hits.count)")
    for (win, wid) in hits {
        AXUIElementPerformAction(win, kAXRaiseAction as CFString)
        activatePid(pid)
        let t = Target(name: "stickies", pid: pid, winID: wid) { nil }
        do {
            try guardFront(t, "cleanup")
            try typeScript("{CMD+A}{BS}", speed: .fast, check: {})
            usleep(300_000)
            try guardFront(t, "cleanup close")
            try typeScript("{CMD+W}", speed: .fast, check: {})
            usleep(800_000)
            for w2 in (axCopy(AXUIElementCreateApplication(pid), kAXWindowsAttribute as String) as? [AXUIElement]) ?? [] where axWindowID(w2) == wid {
                if let b = pressSheetButton(w2, titles: ["Delete", "Don\u{2019}t Save", "Don't Save", "Discard"]) { print("pressed \(b)") }
            }
            print("window \(wid) \(windows().contains { $0.id == wid } ? "still open" : "closed")")
        } catch { print("cleanup aborted: \(error)") }
    }
    exit(0)
case "watch":   // diagnostics: listen-only, print a histogram of who is generating input events for N seconds
    let secs = Double(args.dropFirst().first ?? "20") ?? 20
    wVerbose = true
    guard startWatcher() else { print("watcher failed"); exit(3) }
    usleep(UInt32(secs * 1_000_000))
    print("pid0 events by type: \(wPhys)"); print("seen=\(wSeen) ours=\(wOurs) physical-flag=\(wFlag ?? "none") others=\(wOthers) idle=\(hidIdle())")
    exit(0)
case "front":   // debugging aid: front pid / top window for 3 s, no input
    for _ in 0..<10 { let fp = focusedAppPid(); let tw = topNormalWindow(); print("front pid", fp, "top window", tw?.id ?? 0, tw?.owner ?? "", tw?.pid ?? 0); usleep(300_000) }
    exit(0)
case "winlist":   // debugging aid: every window (also off-screen) of a pid, or all owners
    let want = Int32(args.dropFirst().first ?? "0") ?? 0
    for d in (CGWindowListCopyWindowInfo([.optionAll], kCGNullWindowID) as? [[String: Any]]) ?? [] {
        let pid = Int32(d[kCGWindowOwnerPID as String] as? Int ?? 0)
        if want != 0 && pid != want { continue }
        print(d[kCGWindowNumber as String] ?? "", pid, d[kCGWindowOwnerName as String] ?? "", "layer", d[kCGWindowLayer as String] ?? "", d[kCGWindowName as String] ?? "", d[kCGWindowBounds as String] ?? "", "onscreen", d[kCGWindowIsOnscreen as String] ?? "no")
    }
    exit(0)
case "run":
    let method = opt("method") ?? "telex"
    let targets = (opt("targets") ?? "textedit-plain").split(separator: ",").map(String.init)
    let vkPid = Int32(opt("vk-pid") ?? "0") ?? 0
    vkPidGlobal = vkPid
    perappRaw = Set((opt("perapp-raw") ?? "").split(separator: ",").map(String.init))
    let only = opt("only").map { Set($0.split(separator: ",").map(String.init)) }
    if !FileManager.default.fileExists(atPath: outDir + "/results.jsonl") { FileManager.default.createFile(atPath: outDir + "/results.jsonl", contents: nil, attributes: nil) }
    resultsHandle = FileHandle(forWritingAtPath: outDir + "/results.jsonl"); resultsHandle?.seekToEndOfFile()
    guard AXIsProcessTrusted(), CGPreflightPostEventAccess(), CGPreflightListenEventAccess() else { log("ABORT: TCC grants missing"); exit(3) }
    guard startWatcher() else { log("ABORT: cannot create the physical-input watcher tap"); exit(3) }
    usleep(500_000)
    let idle = hidIdle()
    log("run method=\(method) targets=\(targets) vkPid=\(vkPid) idle=\(Int(idle))s input_source=\(inputSourceID()) secure_input=\(IsSecureEventInputEnabled())")
    if env["SKIP_IDLE_GATE"] == nil && idle < 60 { log("ABORT: owner not idle (\(idle) s < 60 s). Nothing sent."); exit(4) }
    calibrateIdle()
    var exitCode: Int32 = 0
    do {
        if targets.contains("gui") { try guiSmoke(vkPid: vkPid) }
        if targets.contains("secure") { try secureCase(vkPid: vkPid, method: method) }
        for tg in targets where tg != "gui" && tg != "secure" {
            do { try runTarget(tg, method: method, only: only) }
            catch let a as Abort {
                log("TARGET ABORT \(tg): \(a.why)")
                if a.physical { throw a }
                exitCode = 5
            }
        }
        log("watcher: seen=\(wSeen) ours=\(wOurs) vk=\(wVK) otherSoftware=\(wOthers) flag=\(wFlag ?? "none")")
    } catch let a as Abort {
        log("RUN ABORT: \(a.why)"); exitCode = a.physical ? 6 : 5
    } catch { log("RUN ERROR: \(error)"); exitCode = 7 }
    releaseAllMods()
    for c in children where c.isRunning { killChild(c) }
    for (k, v) in counts.sorted(by: { $0.key < $1.key }) { log("TALLY \(k): PASS \(v.pass) FAIL \(v.fail)") }
    exit(exitCode)
default:
    print("unknown command"); exit(2)
}
