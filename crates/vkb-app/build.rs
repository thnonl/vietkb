fn main() {
    // tauri-build only emits `rerun-if-changed` for tauri.conf.json and the
    // capabilities dir, so replacing an icon in place would leave the stale
    // bitmap embedded in the exe (Explorer, Start menu, uninstall entry).
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/32x32.png");
    println!("cargo:rerun-if-changed=icons/128x128.png");
    tauri_build::build();
}
