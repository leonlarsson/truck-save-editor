//! Sets the product name for the code (`APP_NAME`), and on Windows puts the app icon
//! (`assets/icon.ico`, drawn by `scripts/make-icon.py`) and the file's version details (product,
//! version, copyright: what Explorer shows under Properties → Details) into the .exe.

/// The product name: the one place it's written. The code reads it as `crate::APP_NAME`; the .exe
/// carries it as its product name.
const PRODUCT_NAME: &str = "Truck Save Editor";

fn main() {
    println!("cargo:rustc-env=APP_NAME={PRODUCT_NAME}");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=../../LICENSE");
    embed_live_plugin();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Without the resource compiler the app still builds and runs, just without its icon and
    // details, so say so instead of failing the build.
    if let Err(error) = embed_windows_resources() {
        println!("cargo:warning=The app icon and version details weren't embedded: {error}");
    }
}

/// Puts Live's plugin (`tse_live.dll`, built from `crates/live-plugin`) inside the .exe, when
/// it has been built: the code then carries its bytes (`live::EMBEDDED_PLUGIN`) and writes them
/// into the game's folder when the player asks. The plugin is a separate build, so it must be
/// built first (`scripts/windows.sh` and CI do); without it the app builds as before and Live
/// says the plugin is missing.
fn embed_live_plugin() {
    // Tell the compiler the setting exists, whether or not it's set.
    println!("cargo::rustc-check-cfg=cfg(has_live_plugin)");
    // The plugin is a Windows library, for the game on Windows: other builds carry none.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Cargo builds everything of one build into the same folder, `target/<profile>`: the folder
    // that holds `build`, which `OUT_DIR` is somewhere inside (how deep differs between Cargo
    // versions). Only that folder is looked in, so a file of the same name anywhere else is
    // never taken for the plugin.
    let out_dir = std::env::var_os("OUT_DIR").map(std::path::PathBuf::from);
    let Some(plugin) = out_dir.as_deref().and_then(|out| {
        out.ancestors()
            .find(|dir| dir.file_name().is_some_and(|name| name == "build"))
            .and_then(std::path::Path::parent)
            .map(|profile| profile.join("tse_live.dll"))
    }) else {
        return;
    };
    // Built again when the plugin appears or changes. (It's the plugin of whatever was built
    // last in this folder: the scripts and CI always build it right before the app.)
    println!("cargo:rerun-if-changed={}", plugin.display());
    if plugin.is_file() {
        println!("cargo:rustc-env=TSE_LIVE_PLUGIN={}", plugin.display());
        println!("cargo:rustc-cfg=has_live_plugin");
    }
}

/// Writes the resource script into the build folder (it holds this build's version) and compiles
/// it into the .exe.
fn embed_windows_resources() -> Result<(), String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?;
    let out_dir = std::env::var("OUT_DIR").map_err(|e| e.to_string())?;
    let version = std::env::var("CARGO_PKG_VERSION").map_err(|e| e.to_string())?;
    let icon = std::path::Path::new(&manifest_dir)
        .join("assets")
        .join("icon.ico");
    let license = std::path::Path::new(&manifest_dir).join("../../LICENSE");
    // "Copyright (c) 2026 …", as the license says it.
    let copyright = std::fs::read_to_string(&license)
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("Copyright"))
                .map(str::to_string)
        })
        .unwrap_or_default();
    let script = resource_script(&version, &copyright, &icon.to_string_lossy());
    let path = std::path::Path::new(&out_dir).join("app.rc");
    std::fs::write(&path, script).map_err(|e| e.to_string())?;
    embed_resource::compile(&path, embed_resource::NONE)
        .manifest_optional()
        .map_err(|e| e.to_string())
}

/// The resource script: icon 1 (the one GPUI loads for its windows, and the first one Explorer
/// shows for the .exe) and the version details.
fn resource_script(version: &str, copyright: &str, icon_path: &str) -> String {
    // The numeric version is major, minor, patch, 0; a pre-release (0.2.0-beta.1) keeps its full
    // name in the text fields.
    let numbers: Vec<u16> = version
        .split(['.', '-', '+'])
        .take(3)
        .map(|part| part.parse().unwrap_or(0))
        .collect();
    let number = |index: usize| numbers.get(index).copied().unwrap_or(0);
    let numeric = format!("{},{},{},0", number(0), number(1), number(2));
    // Text in a resource script is quoted, with backslashes and quotes escaped.
    let quote = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\"\""));
    let exe = "truck-save-editor.exe";
    format!(
        r#"#pragma code_page(65001)
1 ICON {icon}

1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x1
FILESUBTYPE 0x0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "ProductName", {product}
      VALUE "FileDescription", {product}
      VALUE "ProductVersion", {version}
      VALUE "FileVersion", {version}
      VALUE "LegalCopyright", {copyright}
      VALUE "InternalName", {exe}
      VALUE "OriginalFilename", {exe}
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
        icon = quote(icon_path),
        product = quote(PRODUCT_NAME),
        version = quote(version),
        copyright = quote(copyright),
        exe = quote(exe),
    )
}
