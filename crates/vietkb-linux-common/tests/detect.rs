use std::collections::BTreeMap;

use vietkb_linux_common::{InputFramework, LinuxEnvironment, SessionKind};

fn env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map = vars
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect::<BTreeMap<_, _>>();
    move |key| map.get(key).cloned()
}

#[test]
fn detects_wayland_and_ibus() {
    let detected = LinuxEnvironment::detect_from(env(&[
        ("XDG_SESSION_TYPE", "wayland"),
        ("GTK_IM_MODULE", "ibus"),
    ]));

    assert_eq!(detected.session, SessionKind::Wayland);
    assert_eq!(detected.preferred_framework(), InputFramework::IBus);
}

#[test]
fn fcitx_is_preferred_when_configured() {
    let detected = LinuxEnvironment::detect_from(env(&[
        ("DISPLAY", ":0"),
        ("QT_IM_MODULE", "fcitx"),
    ]));

    assert_eq!(detected.session, SessionKind::X11);
    assert_eq!(detected.preferred_framework(), InputFramework::Fcitx5);
}
