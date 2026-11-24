//! Online libraries in the preset browser (A34), with a fake network:
//! off by default and silent, on through Preferences, catalogue to tiles,
//! download on use, offline from the cache, commands and the MCP tool.
use super::*;
use crate::online_library::{Fetch, OnlinePreferences};
use crate::prefs_file::PrefsFile;
use crate::preset_browser::Target;
use operandi_core::preset::library::EntryKind;
use operandi_core::preset::online::{self, AssetKind, SourceId};
use std::sync::{Arc, Mutex};

const HDRIS: &str = r#"{"sky_one": {"type": 0, "name": "Sky One", "category": "Skies/Clear",
  "authors": {"A. Person": "All"}, "max_resolution": [4096, 2048],
  "thumbnail_url": "https://cdn.polyhaven.com/asset_img/thumbs/sky_one.png?width=256&height=256"}}"#;
const TEXTURES: &str = r#"{"rock_wall": {"type": 1, "name": "Rock Wall", "category": "Rock",
  "authors": {"B. Person": "All"}, "max_resolution": [2048, 2048],
  "thumbnail_url": "https://cdn.polyhaven.com/asset_img/thumbs/rock_wall.png"}}"#;
const ROCK_FILES: &str = r#"{
  "Diffuse": {"1k": {"jpg": {"url": "https://dl.polyhaven.org/t/rock_wall_diff_1k.png", "size": 10, "md5": ""}}},
  "Rough": {"1k": {"jpg": {"url": "https://dl.polyhaven.org/t/rock_wall_rough_1k.png", "size": 10, "md5": ""}}}
}"#;
const SKY_FILES: &str = r#"{"hdri": {"1k": {"hdr": {"url": "https://dl.polyhaven.org/h/sky_one_1k.hdr", "size": 10, "md5": ""}}}}"#;

fn png() -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(4, 4, image::Rgba([120, 100, 90, 255]))
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// A fake network that answers Poly Haven's addresses and records every
/// one asked for; `offline` makes every request fail.
fn network(offline: Arc<Mutex<bool>>, asked: Arc<Mutex<Vec<String>>>) -> Fetch {
    Arc::new(move |url: &str, to: Option<&std::path::Path>| {
        asked.lock().unwrap().push(url.to_owned());
        if *offline.lock().unwrap() {
            return Err("offline".into());
        }
        let body: Vec<u8> = if url.contains("assets?type=hdris") {
            HDRIS.into()
        } else if url.contains("assets?type=textures") {
            TEXTURES.into()
        } else if url.contains("assets?type=models") {
            b"{}".to_vec()
        } else if url.ends_with("/files/rock_wall") {
            ROCK_FILES.into()
        } else if url.ends_with("/files/sky_one") {
            SKY_FILES.into()
        } else if url.ends_with(".png") || url.contains("thumbs/") {
            png()
        } else if url.ends_with(".hdr") {
            b"#?RADIANCE\n".to_vec()
        } else {
            return Err(format!("no such address {url}"));
        };
        match to {
            Some(path) => {
                std::fs::write(path, &body).map_err(|e| e.to_string())?;
                Ok(Vec::new())
            }
            None => Ok(body),
        }
    })
}

struct Net {
    offline: Arc<Mutex<bool>>,
    asked: Arc<Mutex<Vec<String>>>,
}

fn app() -> (Operandi, Net) {
    let mut a = Operandi::empty();
    a.initialize_new_document();
    a.load_preset_browser();
    let net = Net {
        offline: Arc::new(Mutex::new(false)),
        asked: Arc::new(Mutex::new(Vec::new())),
    };
    a.online
        .set_fetch(network(net.offline.clone(), net.asked.clone()));
    (a, net)
}

/// Run frames until `done`, or fail after a few seconds.
fn until(a: &mut Operandi, what: &str, done: impl Fn(&mut Operandi) -> bool) {
    let ctx = egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut t = 0.;
    while !done(a) {
        assert!(
            std::time::Instant::now() < deadline,
            "{what} never happened: {}",
            a.status
        );
        a.tick_online_library(&ctx, true);
        t += 2.;
        a.preset_browser.tick(t);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn turn_on(a: &mut Operandi) {
    a.execute("pref.value online.poly_haven true");
    assert!(a.online.prefs.poly_haven, "{}", a.status);
    until(a, "the catalogue", |a| {
        a.online.refreshing.is_empty() && a.online.catalogue(SourceId::PolyHaven).is_some()
    });
}

#[test]
fn every_library_is_off_by_default_and_never_contacted() {
    let (mut a, net) = app();
    assert_eq!(a.online.prefs, OnlinePreferences::default());
    assert!(!a.online.prefs.poly_haven && !a.online.prefs.ambient_cg);
    let ctx = egui::Context::default();
    for _ in 0..5 {
        a.tick_online_library(&ctx, true);
    }
    assert!(
        a.preset_browser.online_roots.is_empty(),
        "not in the browser"
    );
    assert!(net.asked.lock().unwrap().is_empty(), "no request at all");
    // Commands refuse while it's off, without a request.
    a.execute("operandi.online.refresh");
    assert!(a.status.contains("off"), "{}", a.status);
    a.execute("operandi.online.download source:polyhaven kind:hdri id:sky_one");
    assert!(a.status.contains("off"), "{}", a.status);
    assert!(net.asked.lock().unwrap().is_empty());
}

#[test]
fn turned_on_its_catalogue_becomes_folders_of_tiles_with_thumbnails() {
    let (mut a, net) = app();
    turn_on(&mut a);
    // Saved like any preference.
    let saved = OnlinePreferences::load(&a.environment.paths().online_preferences());
    assert!(saved.poly_haven && !saved.ambient_cg);
    let root = a.online.root(SourceId::PolyHaven);
    assert_eq!(
        a.preset_browser.online_roots,
        [(root.clone(), SourceId::PolyHaven)]
    );
    assert!(a.preset_browser.read_only(&root.join("HDRIs")));
    // Only Poly Haven was asked, with its catalogue pages.
    assert!(
        net.asked
            .lock()
            .unwrap()
            .iter()
            .all(|u| u.starts_with("https://api.polyhaven.com/assets?type="))
    );
    let folder = root.join("Materials").join("Rock");
    a.preset_browser.show_folder(folder.clone());
    until(&mut a, "the listing", |a| {
        a.preset_browser.listing.is_some()
    });
    let entries = a.preset_browser.shown();
    assert_eq!(entries.len(), 1);
    let rock = &entries[0];
    assert_eq!(rock.name, "Rock Wall");
    assert_eq!(rock.kind, EntryKind::TextureSet);
    let tip = a.entry_tip(rock);
    assert!(
        tip.contains("CC0") && tip.contains("Powered by Poly Haven"),
        "{tip}"
    );
    assert!(tip.contains("Not downloaded"), "{tip}");
    // A tile on screen asks for its thumbnail; it lands in the cache.
    let thumb = a
        .online
        .want_thumbnail(&root, rock.online.as_ref().unwrap());
    until(&mut a, "the thumbnail", |_| thumb.is_file());
    let job = crate::preset_browser::PresetBrowser::thumb_job(rock);
    assert_eq!(job.key, thumb);
    // Turned off: gone from the browser, the shown folder back to Basics.
    a.execute("pref.value online.poly_haven false");
    let ctx = egui::Context::default();
    a.tick_online_library(&ctx, true);
    assert!(a.preset_browser.online_roots.is_empty());
    assert_ne!(a.preset_browser.folder.as_ref(), Some(&folder));
}

#[test]
fn using_a_material_downloads_it_then_loads_it_and_later_works_offline() {
    let (mut a, net) = app();
    turn_on(&mut a);
    let root = a.online.root(SourceId::PolyHaven);
    a.preset_browser
        .show_folder(root.join("Materials").join("Rock"));
    until(&mut a, "the listing", |a| {
        a.preset_browser.listing.is_some()
    });
    let rock = a.preset_browser.shown().remove(0);
    let asset = rock.online.clone().unwrap();
    // A material loads into the selected material group.
    a.run(Command::ShaderTree(ShaderTreeEdit::Add {
        layer: operandi_core::shader_tree::NewLayer::Group,
        selected: None,
    }));
    let group = a.editor.scene.shader_tree.next_id - 1;
    a.shading.selected = vec![group];
    // Double-click: a download starts; nothing is loaded yet.
    let groups = a.editor.scene.shader_tree.nodes.len();
    assert!(a.browser_load(&rock, Target::Default), "{}", a.status);
    assert!(
        a.status.starts_with("Downloading Rock Wall"),
        "{}",
        a.status
    );
    until(&mut a, "the download and load", |a| {
        a.online.downloads.is_empty()
    });
    assert!(a.status.contains("Loaded texture set"), "{}", a.status);
    assert!(a.editor.scene.shader_tree.nodes.len() > groups);
    assert_eq!(a.online.have(&asset), [1]);
    assert!(a.online_tip(&rock).unwrap().contains("Downloaded: 1K"));
    // Offline, the cached download loads again without a request.
    *net.offline.lock().unwrap() = true;
    let before = net.asked.lock().unwrap().len();
    assert!(a.browser_load(&rock, Target::Default), "{}", a.status);
    assert!(a.status.contains("Loaded texture set"), "{}", a.status);
    assert_eq!(net.asked.lock().unwrap().len(), before);
    // A refresh offline fails quietly and keeps the cached catalogue.
    a.execute("operandi.online.refresh");
    until(&mut a, "the failed refresh", |a| {
        a.online.refreshing.is_empty()
    });
    assert!(a.online.errors.contains_key(&SourceId::PolyHaven));
    assert!(a.online.catalogue(SourceId::PolyHaven).is_some());
    // Remove Downloaded Files, then Clear Cache.
    a.run_online_want(crate::online_library_ui::Want::Remove(rock.clone()));
    assert!(a.online.have(&asset).is_empty(), "{}", a.status);
    a.execute("operandi.online.clearCache");
    assert!(a.status.starts_with("Cleared"), "{}", a.status);
    assert!(!root.join(online::THUMBS_DIR).exists());
    assert!(
        root.join(online::CATALOGUE_FILE).is_file(),
        "catalogue kept"
    );
}

#[test]
fn the_command_downloads_without_loading_and_the_bridge_reports_it() {
    let (mut a, _net) = app();
    // The MCP tool turns the library on like its switch.
    let r = a
        .mcp_online_library(&serde_json::json!({"action": "enable", "source": "polyhaven"}))
        .unwrap();
    assert_eq!(r["sources"][0]["on"], true, "{r}");
    until(&mut a, "the catalogue", |a| {
        a.online.refreshing.is_empty() && a.online.catalogue(SourceId::PolyHaven).is_some()
    });
    let list = a
        .mcp_online_library(&serde_json::json!({"action": "list", "kind": "hdri", "search": "sky"}))
        .unwrap();
    assert_eq!(list["total"], 1, "{list}");
    assert_eq!(list["assets"][0]["id"], "sky_one");
    assert_eq!(list["assets"][0]["licence"], online::LICENCE);
    a.execute("operandi.online.download source:polyhaven kind:hdri id:sky_one resolution:4");
    assert!(a.status.starts_with("Downloading Sky One"), "{}", a.status);
    until(&mut a, "the download", |a| a.online.downloads.is_empty());
    assert!(a.status.starts_with("Downloaded Sky One"), "{}", a.status);
    let got = a
        .mcp_online_library(&serde_json::json!({
            "action": "get", "source": "polyhaven", "kind": "hdri", "id": "sky_one"
        }))
        .unwrap();
    assert_eq!(got["state"], "ready", "{got}");
    assert_eq!(got["downloaded"], serde_json::json!([1]));
    assert!(got["file"].as_str().unwrap().ends_with("sky_one_1k.hdr"));
    let bad = a.mcp_online_library(
        &serde_json::json!({"action": "get", "source": "polyhaven", "kind": "model", "id": "x"}),
    );
    assert!(bad.is_err());
    assert!(
        a.mcp_online_library(&serde_json::json!({"action": "status", "nope": 1}))
            .is_err()
    );
}

/// Live: the real Poly Haven and ambientCG, one asset of each kind fetched
/// at 1K and loaded. Opt-in (it downloads about 30 MB):
/// `cargo test -p operandi-app online_library_tests::live -- --ignored --nocapture`
#[test]
#[ignore = "goes online; run by hand"]
fn live_sources_download_and_load_one_of_each_kind() {
    let mut a = Operandi::empty();
    a.initialize_new_document();
    a.load_preset_browser();
    a.online.set_fetch(Arc::new(crate::online_library::curl));
    a.online.prefs.resolution = 1;
    for source in SourceId::ALL {
        a.online.prefs.turn_on(source, true);
    }
    let ctx = egui::Context::default();
    a.tick_online_library(&ctx, true);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    while !a.online.refreshing.is_empty() {
        assert!(
            std::time::Instant::now() < deadline,
            "catalogues: {}",
            a.status
        );
        a.tick_online_library(&ctx, true);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(a.online.errors.is_empty(), "{:?}", a.online.errors);
    a.run(Command::ShaderTree(ShaderTreeEdit::Add {
        layer: operandi_core::shader_tree::NewLayer::Group,
        selected: None,
    }));
    a.shading.selected = vec![a.editor.scene.shader_tree.next_id - 1];
    for source in SourceId::ALL {
        let n = a.online.catalogue(source).unwrap().assets.len();
        println!("{}: {n} assets", source.label());
        for &kind in source.kinds() {
            let assets = a.online.find(Some(source), Some(kind), "");
            // A small, early one: the catalogues are sorted by id.
            let asset = assets.first().expect("the kind has assets").clone();
            let items = a.editor.scene.items.len();
            let nodes = a.editor.scene.shader_tree.nodes.len();
            assert!(
                a.online_get(
                    &asset,
                    1,
                    crate::online_library::Then::Load(Target::Default)
                ),
                "{}",
                a.status
            );
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
            while !a.online.downloads.is_empty() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "{}: {}",
                    asset.name,
                    a.status
                );
                a.tick_online_library(&ctx, true);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            println!("  {} {}: {}", kind.slug(), asset.id, a.status);
            assert_eq!(a.online.have(&asset).len(), 1, "{}", a.status);
            match kind {
                AssetKind::Texture => assert!(a.editor.scene.shader_tree.nodes.len() > nodes),
                AssetKind::Model => assert!(a.editor.scene.items.len() > items, "{}", a.status),
                AssetKind::Hdri => assert!(!a.status.contains("Couldn't"), "{}", a.status),
            }
        }
    }
}
