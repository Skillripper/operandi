// Shared by the build scripts of operandi-core, operandi-app and
// operandi-mcp (`include!`d there).
//
// The behaviour specs and their oracle data live in a private checkout next
// to this one (`../Operandi-specs`), or wherever `OPERANDI_SPECS` points.
// Tests that read them are compiled only when that folder is found (cfg
// `operandi_specs`), so a public checkout builds and tests without it.
// OPERANDI_SPECS_DIR is always set, to the place it was looked for.

/// Sets `OPERANDI_SPECS_DIR` and, when the folder exists, cfg `operandi_specs`.
fn find_private_specs() {
    println!("cargo:rustc-check-cfg=cfg(operandi_specs)");
    println!("cargo:rerun-if-env-changed=OPERANDI_SPECS");
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let dir = match std::env::var_os("OPERANDI_SPECS") {
        Some(dir) => std::path::PathBuf::from(dir),
        // crates/<crate> -> the checkout -> the folder holding the checkouts.
        None => manifest.join("../../../Operandi-specs"),
    };
    let dir = dir.canonicalize().unwrap_or(dir);
    println!("cargo:rustc-env=OPERANDI_SPECS_DIR={}", dir.display());
    let marker = dir.join("specifications");
    if marker.is_dir() {
        println!("cargo:rustc-cfg=operandi_specs");
        println!("cargo:rerun-if-changed={}", marker.display());
    }
}
