//! With `--features cycles`, lets the app find liboperandi_cycles at run
//! time: next to the build output during development, and in the app
//! bundle's Frameworks folder once scripts/bundle-dev.sh copies it there.
//! Windows finds the DLLs next to Operandi.exe without any setting.
//!
//! Writes the rendered-icon table (rendered_icons.rs) from
//! assets/rendered-icons/icons.txt, embedding just the PNGs it lists.
//!
//! Also stamps the git commit into OPERANDI_GIT_COMMIT for bug and crash
//! reports (bug_report.rs): short hash, "+dirty" with uncommitted changes to
//! tracked files, "unknown" outside a git checkout. OPERANDI_GIT_COMMIT_TIME
//! is that commit's time (Unix seconds, empty when unknown) for the update
//! check (update_check.rs), which also reads OPERANDI_DEFAULT_UPDATE_CHANNEL
//! (`test` makes a build follow the Test channel until the user picks one).
include!("../operandi-core/build_specs.rs");

fn main() {
    find_private_specs();
    stamp_git_commit();
    embed_shipped_layouts();
    embed_rendered_icons();
    set_windows_main_stack();
    println!("cargo:rerun-if-env-changed=DEP_OPERANDI_CYCLES_ROOT");
    println!("cargo:rerun-if-env-changed=DEP_OPERANDI_USD_ROOT");
    // The USD bridge (`--features usd`) is found the same way as Cycles.
    let roots =
        ["DEP_OPERANDI_CYCLES_ROOT", "DEP_OPERANDI_USD_ROOT"].map(|key| std::env::var(key).ok());
    if roots.iter().any(Option::is_some) {
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
        } else if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
            // Next to the program, or in ../lib as in the AppImage
            // (scripts/build-release.sh).
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
        }
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
            for dir in roots.iter().flatten() {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");
            }
        }
    }
}

/// Windows gives a program's main thread only 1 MB of stack (Mac and Linux
/// give 8 MB). The app runs on the main thread (the window event loop must
/// live there on the Mac) and its state is big, so debug builds overflowed
/// on the first click. Ask the linker for the same 8 MB everywhere.
fn set_windows_main_stack() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
        Ok("msvc") => println!("cargo:rustc-link-arg-bins=/STACK:8388608"),
        Ok("gnu") => println!("cargo:rustc-link-arg-bins=-Wl,--stack,8388608"),
        _ => {}
    }
}

fn git(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn stamp_git_commit() {
    let commit = git(&["rev-parse", "--short", "HEAD"]).filter(|c| !c.is_empty());
    let dirty = commit.is_some()
        && git(&["status", "--porcelain", "--untracked-files=no"]).is_some_and(|s| !s.is_empty());
    let stamp = match commit {
        Some(c) if dirty => format!("{c}+dirty"),
        Some(c) => c,
        None => "unknown".into(),
    };
    println!("cargo:rustc-env=OPERANDI_GIT_COMMIT={stamp}");
    let time = git(&["log", "-1", "--format=%ct", "HEAD"]).unwrap_or_default();
    println!("cargo:rustc-env=OPERANDI_GIT_COMMIT_TIME={time}");
    println!("cargo:rerun-if-env-changed=OPERANDI_DEFAULT_UPDATE_CHANNEL");
    // Re-stamp when HEAD moves, the index changes or this crate's sources
    // change. A worktree's git files are not under ./.git, so ask git where
    // they are. (Edits elsewhere in the workspace can leave "+dirty" stale
    // until the next stamp.)
    let mut watched = vec!["HEAD".to_owned(), "index".to_owned()];
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"]) {
        watched.push(branch);
    }
    for file in watched {
        if let Some(path) = git(&["rev-parse", "--git-path", &file]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
}

/// Layouts stage 1 (LAY-70/75, UIC-190): every `*.json` file in
/// `assets/layouts` is a shipped layout, built into the app. Adding a
/// built-in layout is adding a file there; no code names them.
fn embed_shipped_layouts() {
    embed_json_folder("assets/layouts", "shipped_layouts.rs");
    // Toolbox forms (layouts stage 3, form/command_form.rs).
    embed_json_folder("assets/forms", "shipped_forms.rs");
    // Palettes (layouts stage 6, palettes.rs).
    embed_json_folder("assets/palettes", "shipped_palettes.rs");
}

/// Lists every `*.json` in `folder` as `(file name, include_str!(...))`
/// pairs in `OUT_DIR/<list>`.
fn embed_json_folder(folder: &str, list: &str) {
    let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(folder);
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    let mut out = String::from("&[\n");
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
        let name = file.file_name().unwrap().to_string_lossy();
        out.push_str(&format!(
            "    ({name:?}, include_str!({:?})),\n",
            file.display().to_string()
        ));
    }
    out.push(']');
    let target = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join(list);
    std::fs::write(target, out).expect("write the shipped files list");
}

/// The installed sizes per icon and set; `rendered_icons::PIXELS` matches
/// (`installed_files_match_the_table` fails if they drift).
const RENDERED_PIXELS: [u32; 8] = [16, 20, 24, 32, 40, 48, 64, 128];

/// Turns assets/rendered-icons/icons.txt ("<slug> <Icon variant>" lines)
/// into `RENDERED` and `FILES` for rendered_icons.rs, embedding each listed
/// icon's PNGs at every size in both outline sets. The icon pictures stay
/// files; only this small index is code.
fn embed_rendered_icons() {
    let dir = std::path::Path::new("assets/rendered-icons");
    let index = dir.join("icons.txt");
    println!("cargo:rerun-if-changed={}", index.display());
    for set in ["dark-outline", "light-outline"] {
        println!("cargo:rerun-if-changed={}", dir.join(set).display());
    }
    let text = std::fs::read_to_string(&index).expect("assets/rendered-icons/icons.txt");
    let entries: Vec<(&str, &str)> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut words = l.split_whitespace();
            let slug = words.next().unwrap();
            let icon = words
                .next()
                .unwrap_or_else(|| panic!("icons.txt: no Icon for {slug}"));
            (slug, icon)
        })
        .collect();
    let mut out = String::from(
        "/// The icons drawn from studio renders instead of their flat SVGs, with\n/// the slug each was rendered as (`tools/icon-studio/icons/<slug>.opscene`).\n/// The SVG stays as the fallback. Sorted by slug (icons.txt).\npub const RENDERED: &[(Icon, &str)] = &[\n",
    );
    for (slug, icon) in &entries {
        out.push_str(&format!("    (Icon::{icon}, {slug:?}),\n"));
    }
    out.push_str("];\n\nstatic FILES: &[(&str, &str, u32, &[u8])] = &[\n");
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    for (slug, _) in &entries {
        for set in ["dark-outline", "light-outline"] {
            for px in RENDERED_PIXELS {
                out.push_str(&format!(
                    "    ({set:?}, {slug:?}, {px}, include_bytes!({:?})),\n",
                    format!("{root}/assets/rendered-icons/{set}/{slug}-{px}.png")
                ));
            }
        }
    }
    out.push_str("];\n");
    let path =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("rendered_icons_table.rs");
    std::fs::write(path, out).unwrap();
}
