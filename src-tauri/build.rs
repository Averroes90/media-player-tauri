fn main() {
    // Tell Rust about the mobile cfg
    println!("cargo::rustc-check-cfg=cfg(mobile)");

    // libmpv-sys should handle the linking automatically
    // but we can help it find the right path
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=/opt/homebrew/Cellar/mpv/0.40.0_1/lib");
    }

    tauri_build::build();
}
