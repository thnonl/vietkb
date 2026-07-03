#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Wayland,
    X11,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFramework {
    IBus,
    Fcitx5,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxEnvironment {
    pub session: SessionKind,
    pub gtk_im_module: Option<String>,
    pub qt_im_module: Option<String>,
    pub xmodifiers: Option<String>,
}

impl LinuxEnvironment {
    pub fn detect_from(getenv: impl Fn(&str) -> Option<String>) -> Self {
        let session = match getenv("XDG_SESSION_TYPE").as_deref() {
            Some("wayland") => SessionKind::Wayland,
            Some("x11") => SessionKind::X11,
            _ if getenv("WAYLAND_DISPLAY").is_some() => SessionKind::Wayland,
            _ if getenv("DISPLAY").is_some() => SessionKind::X11,
            _ => SessionKind::Unknown,
        };

        Self {
            session,
            gtk_im_module: getenv("GTK_IM_MODULE"),
            qt_im_module: getenv("QT_IM_MODULE"),
            xmodifiers: getenv("XMODIFIERS"),
        }
    }

    pub fn preferred_framework(&self) -> InputFramework {
        if has_framework(&self.gtk_im_module, "fcitx")
            || has_framework(&self.qt_im_module, "fcitx")
            || has_framework(&self.xmodifiers, "fcitx")
        {
            return InputFramework::Fcitx5;
        }

        if has_framework(&self.gtk_im_module, "ibus")
            || has_framework(&self.qt_im_module, "ibus")
            || has_framework(&self.xmodifiers, "ibus")
        {
            return InputFramework::IBus;
        }

        InputFramework::Unknown
    }
}

fn has_framework(value: &Option<String>, needle: &str) -> bool {
    value
        .as_deref()
        .map(|value| value.to_ascii_lowercase().contains(needle))
        .unwrap_or(false)
}
