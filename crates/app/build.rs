//! Puts the app icon (`assets/icon.ico`, drawn by `scripts/make-icon.py`) into the Windows .exe.

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Without the resource compiler the app still builds and runs, just with Windows' default
    // icon, so say so instead of failing the build.
    let result = embed_resource::compile("assets/app.rc", embed_resource::NONE).manifest_optional();
    if let Err(error) = result {
        println!("cargo:warning=The app icon wasn't embedded: {error}");
    }
}
