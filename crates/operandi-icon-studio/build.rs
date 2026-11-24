//! With `--features cycles`, lets the studio find liboperandi_cycles at run
//! time next to the build output, as the app's build.rs does.
fn main() {
    println!("cargo:rerun-if-env-changed=DEP_OPERANDI_CYCLES_ROOT");
    if let Ok(dir) = std::env::var("DEP_OPERANDI_CYCLES_ROOT")
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");
    }
}
