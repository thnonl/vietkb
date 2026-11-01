//! The "app profile" the D10 score uses: is the foreground app a code editor or
//! a terminal? One pure table, matched once on a focus change (never per key).

/// What kind of app has the focus. `Code` leans the English/code decision
/// toward English; Vietnamese comments still compose (see `lexicon.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AppKind {
    #[default]
    Neutral,
    Code,
}

/// Windows process image names (lowercase, with `.exe`).
const CODE_PROCESSES: &[&str] = &[
    "code.exe",
    "code - insiders.exe",
    "cursor.exe",
    "windsurf.exe",
    "zed.exe",
    "devenv.exe",
    "idea64.exe",
    "idea.exe",
    "pycharm64.exe",
    "webstorm64.exe",
    "phpstorm64.exe",
    "clion64.exe",
    "goland64.exe",
    "rider64.exe",
    "rubymine64.exe",
    "datagrip64.exe",
    "rustrover64.exe",
    "studio64.exe",
    "sublime_text.exe",
    "notepad++.exe",
    "nvim-qt.exe",
    "gvim.exe",
    "windowsterminal.exe",
    "wt.exe",
    "powershell.exe",
    "pwsh.exe",
    "cmd.exe",
    "conhost.exe",
    "openconsole.exe",
    "warp.exe",
    "ghostty.exe",
    "wezterm-gui.exe",
    "alacritty.exe",
    "mintty.exe",
];

/// macOS bundle identifiers, exact (lowercase).
const CODE_BUNDLES: &[&str] = &[
    "com.microsoft.vscode",
    "com.microsoft.vscodeinsiders",
    "com.todesktop.230313mzl4w4u92",
    "com.exafunction.windsurf",
    "dev.zed.zed",
    "dev.zed.zed-preview",
    "com.apple.dt.xcode",
    "com.google.android.studio",
    "com.sublimetext.4",
    "com.sublimetext.3",
    "org.vim.macvim",
    "com.apple.terminal",
    "com.googlecode.iterm2",
    "dev.warp.warp-stable",
    "com.mitchellh.ghostty",
    "net.kovidgoyal.kitty",
    "org.alacritty",
    "com.github.wez.wezterm",
];

/// macOS bundle prefixes (every JetBrains IDE is `com.jetbrains.<product>`).
const CODE_BUNDLE_PREFIXES: &[&str] = &["com.jetbrains."];

fn eq_ignore_case(a: &str, list: &[&str]) -> bool {
    list.iter().any(|item| a.eq_ignore_ascii_case(item))
}

/// Classify a Windows process image name (`Code.exe`, a full path is fine).
pub fn classify_process(image: &str) -> AppKind {
    let name = image.rsplit(['\\', '/']).next().unwrap_or(image);
    if eq_ignore_case(name, CODE_PROCESSES) {
        AppKind::Code
    } else {
        AppKind::Neutral
    }
}

/// Classify a macOS bundle identifier.
pub fn classify_bundle(bundle_id: &str) -> AppKind {
    let id = bundle_id.trim();
    let prefix = |p: &&str| id.len() >= p.len() && id.is_char_boundary(p.len()) && id[..p.len()].eq_ignore_ascii_case(p);
    if eq_ignore_case(id, CODE_BUNDLES) || CODE_BUNDLE_PREFIXES.iter().any(prefix) {
        AppKind::Code
    } else {
        AppKind::Neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_processes() {
        for name in ["Code.exe", "cursor.exe", "C:\\Program Files\\Microsoft VS Code\\Code.exe", "WindowsTerminal.exe", "pwsh.exe", "Notepad++.exe", "devenv.exe"] {
            assert_eq!(classify_process(name), AppKind::Code, "{name}");
        }
        for name in ["notepad.exe", "chrome.exe", "WINWORD.EXE", "slack.exe", ""] {
            assert_eq!(classify_process(name), AppKind::Neutral, "{name}");
        }
    }

    #[test]
    fn macos_bundles() {
        for id in ["com.microsoft.VSCode", "com.apple.dt.Xcode", "com.jetbrains.intellij", "com.jetbrains.pycharm.ce", "com.googlecode.iterm2", "com.apple.Terminal", "dev.zed.Zed", "com.mitchellh.ghostty"] {
            assert_eq!(classify_bundle(id), AppKind::Code, "{id}");
        }
        for id in ["com.apple.Safari", "com.tinyspeck.slackmacgap", "com.apple.TextEdit", "com.google.Chrome", ""] {
            assert_eq!(classify_bundle(id), AppKind::Neutral, "{id}");
        }
    }
}
