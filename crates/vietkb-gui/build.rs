#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rerun-if-changed=vietkb-gui.rc");
    println!("cargo:rerun-if-changed=vietkb-gui.exe.manifest");
    println!(
        "cargo:rustc-link-arg-bin=vietkb-gui=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
    );
    embed_resource::compile("vietkb-gui.rc", embed_resource::NONE);
}

#[cfg(not(target_os = "windows"))]
fn main() {}
