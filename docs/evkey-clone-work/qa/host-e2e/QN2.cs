using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

// QA host E2E native helper v2 (2026-10-30, rebuilt candidate 0D39D42F).
// Physical input is detected ONLY by our own WH_KEYBOARD_LL (LLKHF_INJECTED 0x10) and
// WH_MOUSE_LL (LLMHF_INJECTED 0x01) hooks. GetLastInputInfo is not used mid-run (pre-run idle gate is in PowerShell).
public static class QN2
{
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X; public int Y; }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct LASTINPUTINFO { public uint cbSize; public uint dwTime; }
    [StructLayout(LayoutKind.Sequential)] public struct KBDLLHOOKSTRUCT { public uint vkCode; public uint scanCode; public uint flags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)] public struct MSLLHOOKSTRUCT { public POINT pt; public uint mouseData; public uint flags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)] public struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam; public IntPtr lParam; public uint time; public POINT pt; }
    [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint mouseData; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit)] public struct INPUTUNION { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
    [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public INPUTUNION u; }

    public delegate IntPtr HookProc(int nCode, IntPtr wParam, IntPtr lParam);
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] p, int cb);
    [DllImport("user32.dll")] static extern uint MapVirtualKey(uint code, uint map);
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern bool GetCursorPos(out POINT p);
    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] static extern bool AttachThreadInput(uint a, uint b, bool f);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] static extern bool IsIconic(IntPtr h);
    [DllImport("user32.dll")] static extern bool IsWindow(IntPtr h);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc p, IntPtr l);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextLength(IntPtr h);
    [DllImport("user32.dll")] static extern bool GetLastInputInfo(ref LASTINPUTINFO p);
    [DllImport("user32.dll")] static extern short GetAsyncKeyState(int vk);
    [DllImport("user32.dll")] static extern short GetKeyState(int vk);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern IntPtr GetModuleHandle(string n);
    [DllImport("user32.dll")] static extern IntPtr SetWindowsHookEx(int id, HookProc fn, IntPtr mod, uint tid);
    [DllImport("user32.dll")] static extern IntPtr CallNextHookEx(IntPtr h, int n, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool UnhookWindowsHookEx(IntPtr h);
    [DllImport("user32.dll")] static extern bool GetMessage(out MSG m, IntPtr h, uint a, uint b);
    [DllImport("user32.dll")] static extern bool PostThreadMessage(uint tid, uint msg, IntPtr w, IntPtr l);

    public static volatile bool Abort = false;
    public static string AbortReason = "";
    public static int KbPhysical = 0;
    public static int MousePhysical = 0;
    public static int MouseOwnIgnored = 0;
    public static volatile uint MouseGraceUntil = 0;
    public static uint LastInjectTick = (uint)Environment.TickCount;
    public static List<string> Checks = new List<string>();

    static HookProc _kbp, _msp;
    static IntPtr _kh = IntPtr.Zero, _mh = IntPtr.Zero;
    static uint _tid = 0;
    static Thread _th;

    public class Step { public ushort[] Mods; public ushort Key; public int Kind; public string Label; }
    // Kind: 0 tap, 3 check (label), 4 caps on, 5 caps off, 6 shift down (held), 7 shift up

    // ---------- watchers: the ONLY physical-input detector ----------
    public static void StartWatcher()
    {
        ManualResetEvent ready = new ManualResetEvent(false);
        _th = new Thread(() =>
        {
            _tid = GetCurrentThreadId();
            _kbp = KbHook;
            _msp = MsHook;
            _kh = SetWindowsHookEx(13, _kbp, GetModuleHandle(null), 0);
            _mh = SetWindowsHookEx(14, _msp, GetModuleHandle(null), 0);
            ready.Set();
            MSG m;
            while (GetMessage(out m, IntPtr.Zero, 0, 0)) { }
            UnhookWindowsHookEx(_kh);
            UnhookWindowsHookEx(_mh);
        });
        _th.IsBackground = true;
        _th.Start();
        ready.WaitOne(5000);
    }

    public static void StopWatcher() { if (_tid != 0) PostThreadMessage(_tid, 0x0012, IntPtr.Zero, IntPtr.Zero); }

    static IntPtr KbHook(int n, IntPtr w, IntPtr l)
    {
        if (n >= 0)
        {
            KBDLLHOOKSTRUCT k = (KBDLLHOOKSTRUCT)Marshal.PtrToStructure(l, typeof(KBDLLHOOKSTRUCT));
            if ((k.flags & 0x10) == 0)
            {
                KbPhysical++;
                Abort = true;
                AbortReason = "physical key event (no LLKHF_INJECTED) vk=0x" + k.vkCode.ToString("X2") + " flags=0x" + k.flags.ToString("X");
            }
        }
        return CallNextHookEx(_kh, n, w, l);
    }

    static IntPtr MsHook(int n, IntPtr w, IntPtr l)
    {
        if (n >= 0)
        {
            MSLLHOOKSTRUCT m = (MSLLHOOKSTRUCT)Marshal.PtrToStructure(l, typeof(MSLLHOOKSTRUCT));
            if ((m.flags & 0x01) == 0)
            {
                if ((uint)Environment.TickCount <= MouseGraceUntil) { MouseOwnIgnored++; }
                else
                {
                    MousePhysical++;
                    Abort = true;
                    AbortReason = "physical mouse event (no LLMHF_INJECTED) msg=0x" + w.ToInt32().ToString("X");
                }
            }
        }
        return CallNextHookEx(_mh, n, w, l);
    }

    // ---------- pre-run idle gate only (not used mid-run) ----------
    public static uint IdleMs()
    {
        LASTINPUTINFO li = new LASTINPUTINFO();
        li.cbSize = (uint)Marshal.SizeOf(typeof(LASTINPUTINFO));
        GetLastInputInfo(ref li);
        return (uint)Environment.TickCount - li.dwTime;
    }

    public static void MarkInjected() { LastInjectTick = (uint)Environment.TickCount; }

    // ---------- shift / caps state ----------
    public static string ShiftState()
    {
        bool l = (GetAsyncKeyState(0xA0) & 0x8000) != 0;
        bool r = (GetAsyncKeyState(0xA1) & 0x8000) != 0;
        bool g = (GetAsyncKeyState(0x10) & 0x8000) != 0;
        return "LSHIFT=" + (l ? "DOWN" : "up") + " RSHIFT=" + (r ? "DOWN" : "up") + " SHIFT=" + (g ? "DOWN" : "up");
    }
    public static bool CapsOn() { return (GetKeyState(0x14) & 0x0001) != 0; }
    public static void SetCapsState(bool want)
    {
        if (CapsOn() != want) { KeyEvent(0x14, false); KeyEvent(0x14, true); Thread.Sleep(200); }
    }

    // ---------- focus / windows ----------
    public static IntPtr Fg() { return GetForegroundWindow(); }

    public static bool Force(IntPtr h)
    {
        IntPtr fg = GetForegroundWindow();
        uint dummy;
        uint fgT = GetWindowThreadProcessId(fg, out dummy);
        uint me = GetCurrentThreadId();
        bool att = fgT != 0 && fgT != me && AttachThreadInput(me, fgT, true);
        if (IsIconic(h)) ShowWindow(h, 9);
        SetForegroundWindow(h);
        if (att) AttachThreadInput(me, fgT, false);
        Thread.Sleep(200);
        return GetForegroundWindow() == h;
    }

    public static bool RestoreFg(IntPtr prev)
    {
        if (prev == IntPtr.Zero || !IsWindow(prev)) return false;
        if (GetForegroundWindow() == prev) return true;
        return Force(prev);
    }

    public static IntPtr FindWin(string sub, uint pid)
    {
        IntPtr found = IntPtr.Zero;
        EnumWindows((h, l) =>
        {
            if (!IsWindowVisible(h)) return true;
            uint p;
            GetWindowThreadProcessId(h, out p);
            if (pid != 0 && p != pid) return true;
            int len = GetWindowTextLength(h);
            if (len == 0) return true;
            StringBuilder sb = new StringBuilder(len + 1);
            GetWindowText(h, sb, len + 1);
            if (sb.ToString().IndexOf(sub, StringComparison.OrdinalIgnoreCase) >= 0) { found = h; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    public static uint PidOf(IntPtr h) { uint p; GetWindowThreadProcessId(h, out p); return p; }

    public static string RectOf(IntPtr h) { RECT r; GetWindowRect(h, out r); return r.Left + "," + r.Top + "," + r.Right + "," + r.Bottom; }

    public static int[] RectArr(IntPtr h) { RECT r; GetWindowRect(h, out r); return new int[] { r.Left, r.Top, r.Right, r.Bottom }; }

    public static void Place(IntPtr h, int x, int y, int w, int hgt)
    {
        if (IsIconic(h)) ShowWindow(h, 9);
        SetWindowPos(h, IntPtr.Zero, x, y, w, hgt, 0x0040);
        Thread.Sleep(300);
    }

    public static void CloseWin(IntPtr h) { PostMessage(h, 0x0010, IntPtr.Zero, IntPtr.Zero); }

    public static void Capture(int x, int y, int w, int hgt, string path)
    {
        using (Bitmap bmp = new Bitmap(w, hgt))
        {
            using (Graphics g = Graphics.FromImage(bmp))
            {
                g.CopyFromScreen(x, y, 0, 0, new Size(w, hgt));
            }
            bmp.Save(path, ImageFormat.Png);
        }
    }

    // ---------- injection ----------
    static void KeyEvent(ushort vk, bool up)
    {
        INPUT[] a = new INPUT[1];
        a[0].type = 1;
        a[0].u.ki.wVk = vk;
        a[0].u.ki.wScan = (ushort)MapVirtualKey(vk, 0);
        a[0].u.ki.dwFlags = up ? 2u : 0u;
        SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
    }

    static void MouseEvent(uint flags)
    {
        INPUT[] a = new INPUT[1];
        a[0].type = 0;
        a[0].u.mi.dwFlags = flags;
        SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
    }

    public static void ReleaseModifiers() { KeyEvent(0x10, true); KeyEvent(0xA0, true); KeyEvent(0xA1, true); KeyEvent(0x11, true); KeyEvent(0x12, true); }

    static Step St(ushort key, params ushort[] mods) { Step s = new Step(); s.Key = key; s.Mods = mods; s.Kind = 0; return s; }

    static ushort KeyFor(string p)
    {
        if (p.Length == 1 && ((p[0] >= 'A' && p[0] <= 'Z') || (p[0] >= '0' && p[0] <= '9'))) return (ushort)p[0];
        switch (p)
        {
            case "ESC": return 0x1B;
            case "ENTER": return 0x0D;
            case "DEL": return 0x2E;
            case "END": return 0x23;
            case "TAB": return 0x09;
            case "SPACE": return 0x20;
        }
        int n;
        if (p.Length >= 2 && p[0] == 'F' && int.TryParse(p.Substring(1), out n) && n >= 1 && n <= 24) return (ushort)(0x70 + n - 1);
        throw new ArgumentException("unknown key " + p);
    }

    // spec tokens: {CTRL+SHIFT+Z} chords, {F2}, {ESC}, {ENTER}, {END}, {SHIFT+J}, {SHIFTDOWN}/{SHIFTUP} (held Shift;
    // uppercase letters inside a held span are sent without a Shift wrap), {CAPS_ON}/{CAPS_OFF}, {CHK:label}.
    // Plain uppercase letters outside a held span are typed with Shift down/up around that letter.
    public static List<Step> Parse(string spec)
    {
        List<Step> list = new List<Step>();
        bool held = false;
        int i = 0;
        while (i < spec.Length)
        {
            char c = spec[i];
            if (c == '{')
            {
                int j = spec.IndexOf('}', i);
                if (j < 0) throw new ArgumentException("unclosed brace");
                string raw = spec.Substring(i + 1, j - i - 1);
                string tok = raw.ToUpperInvariant();
                i = j + 1;
                if (tok.StartsWith("CHK:"))
                {
                    Step s = new Step(); s.Kind = 3; s.Label = raw.Substring(4); s.Mods = new ushort[0]; list.Add(s); continue;
                }
                if (tok == "SHIFTDOWN") { Step s = new Step(); s.Kind = 6; s.Mods = new ushort[0]; s.Key = 0x10; list.Add(s); held = true; continue; }
                if (tok == "SHIFTUP") { Step s = new Step(); s.Kind = 7; s.Mods = new ushort[0]; s.Key = 0x10; list.Add(s); held = false; continue; }
                if (tok == "CAPS_ON") { Step s = new Step(); s.Kind = 4; s.Mods = new ushort[0]; list.Add(s); continue; }
                if (tok == "CAPS_OFF") { Step s = new Step(); s.Kind = 5; s.Mods = new ushort[0]; list.Add(s); continue; }
                List<ushort> mods = new List<ushort>();
                ushort key = 0;
                foreach (string part in tok.Split('+'))
                {
                    if (part == "CTRL") mods.Add(0x11);
                    else if (part == "SHIFT") mods.Add(0x10);
                    else if (part == "ALT") mods.Add(0x12);
                    else key = KeyFor(part);
                }
                if (key == 0) throw new ArgumentException("no key in " + tok);
                list.Add(St(key, mods.ToArray()));
                continue;
            }
            i++;
            if (c >= 'a' && c <= 'z') list.Add(St((ushort)(0x41 + c - 'a')));
            else if (c >= 'A' && c <= 'Z') { if (held) list.Add(St((ushort)(0x41 + c - 'A'))); else list.Add(St((ushort)(0x41 + c - 'A'), 0x10)); }
            else if (c >= '0' && c <= '9') list.Add(St((ushort)(0x30 + c - '0')));
            else if (c == ' ') list.Add(St(0x20));
            else if (c == '.') list.Add(St(0xBE));
            else if (c == ',') list.Add(St(0xBC));
            else if (c == '\n') list.Add(St(0x0D));
            else throw new ArgumentException("unsupported char " + c);
        }
        return list;
    }

    // Guard before each batch: no abort flag, window alive, and >=5 s of our own quiet time since the last injection.
    // Physical input is detected by the watchers only (Abort flag).
    static string Guard(IntPtr h, string name)
    {
        if (Abort) return "ABORT:" + AbortReason;
        if (!IsWindow(h)) return "ABORT:window-gone before " + name;
        while (true)
        {
            if (Abort) return "ABORT:" + AbortReason;
            uint now = (uint)Environment.TickCount;
            if (now - LastInjectTick >= 5000) break;
            Thread.Sleep(100);
        }
        return "";
    }

    public static void ClearChecks() { Checks.Clear(); }

    public static string Run(string name, IntPtr h, string spec, int delayMs)
    {
        string g = Guard(h, name);
        if (g != "") return g;
        if (!Force(h)) return "ABORT:test window not foreground before " + name;
        List<Step> steps = Parse(spec);
        foreach (Step s in steps)
        {
            if (Abort) { ReleaseModifiers(); return "ABORT:" + AbortReason; }
            if (GetForegroundWindow() != h) { ReleaseModifiers(); return "ABORT:foreground changed mid-batch " + name; }
            if (s.Kind == 3)
            {
                Checks.Add(s.Label + " | " + ShiftState() + " | capsLock=" + (CapsOn() ? "on" : "off"));
                continue;
            }
            if (s.Kind == 4) { SetCapsState(true); LastInjectTick = (uint)Environment.TickCount; Thread.Sleep(delayMs); continue; }
            if (s.Kind == 5) { SetCapsState(false); LastInjectTick = (uint)Environment.TickCount; Thread.Sleep(delayMs); continue; }
            if (s.Kind == 6) { KeyEvent(0x10, false); LastInjectTick = (uint)Environment.TickCount; Thread.Sleep(delayMs); continue; }
            if (s.Kind == 7) { KeyEvent(0x10, true); LastInjectTick = (uint)Environment.TickCount; Thread.Sleep(delayMs); continue; }
            foreach (ushort m in s.Mods) KeyEvent(m, false);
            KeyEvent(s.Key, false);
            KeyEvent(s.Key, true);
            for (int k = s.Mods.Length - 1; k >= 0; k--) KeyEvent(s.Mods[k], true);
            LastInjectTick = (uint)Environment.TickCount;
            Thread.Sleep(delayMs);
        }
        if (GetForegroundWindow() != h) return "ABORT:foreground changed after " + name;
        return "OK";
    }

    public static string Click(IntPtr h, int x, int y, string name)
    {
        string g = Guard(h, name);
        if (g != "") return g;
        if (!Force(h)) return "ABORT:test window not foreground before click";
        MouseGraceUntil = (uint)Environment.TickCount + 1500;
        SetCursorPos(x, y);
        Thread.Sleep(200);
        if (GetForegroundWindow() != h) return "ABORT:foreground changed before click";
        MouseEvent(0x0002);
        Thread.Sleep(60);
        MouseEvent(0x0004);
        LastInjectTick = (uint)Environment.TickCount;
        MouseGraceUntil = (uint)Environment.TickCount + 300;
        Thread.Sleep(300);
        if (Abort) return "ABORT:" + AbortReason;
        if (GetForegroundWindow() != h) return "ABORT:foreground changed after click";
        return "OK";
    }

    // Cursor smoke test: does our own SetCursorPos produce non-injected LL mouse events? (no keys sent)
    public static string CursorSmoke()
    {
        POINT p; GetCursorPos(out p);
        int before = MousePhysical; bool ab = Abort;
        MouseGraceUntil = 0;
        SetCursorPos(p.X + 2, p.Y + 2);
        Thread.Sleep(300);
        SetCursorPos(p.X, p.Y);
        Thread.Sleep(300);
        int seen = MousePhysical - before;
        // reset the physical-mouse flags caused by our own cursor move (smoke only)
        MousePhysical = before; Abort = ab;
        return "SetCursorPos produced " + seen + " non-injected LL mouse events; cursor restored to " + p.X + "," + p.Y;
    }

    public static void SetCursor(int x, int y) { SetCursorPos(x, y); }
}
