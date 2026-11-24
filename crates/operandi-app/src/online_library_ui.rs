//! Online libraries on screen (master plan A34): their roots in the preset
//! browser's tree, the credit and Refresh Catalogue line over their
//! contents, tile badges, tooltips and menus, loading an asset (download
//! first), the Preferences section and the `operandi.online.*` commands.
//! The network side is `online_library.rs`.
//!
//! UI rules followed: checkbox rows with no left label (UIC-87), dropdowns
//! for choices (UIC-113), a tooltip on every control (UIC-04/119), and
//! read-only roots grey their write actions with a note (UIC-117).
use super::*;
use crate::online_library::{Outcome, Then, key_of, size_label};
use crate::preset_browser::Target;
use crate::tree_list::{Cell, RowIcon, Selected, TreeRow};
use crate::unavailable::Gate;
use operandi_core::preset::library::Entry;
use operandi_core::preset::online::{self, Asset, AssetKind, SourceId};
use std::collections::HashMap;

/// What an online tile's menu asked for, run after drawing.
#[derive(Clone, Debug)]
pub(crate) enum Want {
    Load(Entry),
    /// Fetch it at this resolution without loading it.
    Download(Entry, u32),
    Remove(Entry),
    Reveal(PathBuf),
    Page(String),
}

/// An online asset's right-click menu.
pub(crate) fn entry_menu(ui: &mut egui::Ui, entry: &Entry, want: &mut Option<Want>) {
    let Some(asset) = &entry.online else {
        return;
    };
    let root = online::root_of(&entry.path, asset);
    let have: Vec<(u32, PathBuf)> = root
        .as_ref()
        .map(|root| online::downloaded(root, asset))
        .unwrap_or_default();
    let offered = root
        .as_ref()
        .and_then(|root| online::offered(root, asset))
        .unwrap_or_else(|| asset.resolutions());
    if ui
        .button("Load")
        .on_hover_text(
            "Loads it as any preset loads; downloads it first at the Download Resolution \
             (Preferences > Defaults > Application) unless it's cached",
        )
        .clicked()
    {
        *want = Some(Want::Load(entry.clone()));
        ui.close();
    }
    ui.menu_button("Download At", |ui| {
        for k in offered {
            let done = have.iter().any(|(h, _)| *h == k);
            let label = if done {
                format!("{k}K (downloaded)")
            } else {
                format!("{k}K")
            };
            let tip = format!(
                "Downloads it at {k}K into the cache without loading it (operandi.online.download)"
            );
            if ui
                .gated(
                    done.then_some("Already downloaded at that size"),
                    egui::Button::new(label),
                )
                .on_hover_text(tip)
                .clicked()
            {
                *want = Some(Want::Download(entry.clone(), k));
                ui.close();
            }
        }
    });
    if let Some((_, main)) = have.last() {
        if ui
            .button("Show Downloaded Files")
            .on_hover_text("Shows the cached files in the file manager")
            .clicked()
        {
            *want = Some(Want::Reveal(main.clone()));
            ui.close();
        }
        if ui
            .button("Remove Downloaded Files")
            .on_hover_text("Deletes its cached files; it downloads again when next used")
            .clicked()
        {
            *want = Some(Want::Remove(entry.clone()));
            ui.close();
        }
    }
    ui.separator();
    if ui
        .button("Open Web Page")
        .on_hover_text(format!(
            "Opens its page on {} in your web browser",
            asset.source.home().trim_start_matches("https://")
        ))
        .clicked()
    {
        *want = Some(Want::Page(asset.source.asset_page(&asset.id)));
        ui.close();
    }
}

impl Operandi {
    /// Per frame: the browser's online roots follow the preferences; a
    /// shown browser keeps catalogues fresh; finished work lands.
    pub(super) fn tick_online_library(&mut self, ctx: &Context, shown: bool) {
        self.preset_browser.set_online_roots(self.online.roots());
        if shown {
            self.online.keep_fresh();
        }
        for outcome in self.online.tick() {
            match outcome {
                Outcome::Thumb(path) => self.preset_browser.thumbs.forget(&path, true),
                Outcome::Catalogue { message } => {
                    self.preset_browser.refresh(None);
                    self.status = message;
                }
                Outcome::Downloaded { asset, main, then } => match then {
                    Then::Keep => {
                        self.status =
                            format!("Downloaded {} from {}", asset.name, asset.source.label())
                    }
                    Then::Load(target) => {
                        let entry = self.entry_for_path(&main);
                        self.browser_load(&entry, target);
                    }
                },
                Outcome::Failed(message) => self.status = message,
            }
            ctx.request_repaint();
        }
        if self.online.busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
    }

    /// Load an online asset (PB-30/PB-110 routes): its cached download at
    /// the Download Resolution, or download it and load it when it lands.
    pub(super) fn online_load(&mut self, entry: &Entry, target: Target) -> bool {
        let Some(asset) = entry.online.clone() else {
            return false;
        };
        let want = self.online.prefs.resolution;
        self.online_get(&asset, want, Then::Load(target))
    }

    /// Load `asset` at about `want_k` (cached), or start its download.
    pub(super) fn online_get(&mut self, asset: &Asset, want_k: u32, then: Then) -> bool {
        if !self.online.prefs.enabled(asset.source) {
            self.status = format!(
                "{} is off; turn it on in Preferences > Defaults > Application > Online Libraries",
                asset.source.label()
            );
            return false;
        }
        let root = self.online.root(asset.source);
        let k = online::pick_k(&self.online.resolutions(asset), want_k);
        let cached = online::downloaded(&root, asset)
            .into_iter()
            .find(|(h, _)| Some(*h) == k);
        if let Some((_, main)) = cached {
            return match then {
                Then::Load(target) => {
                    let local = self.entry_for_path(&main);
                    self.browser_load(&local, target)
                }
                Then::Keep => {
                    self.status = format!("{} is already downloaded", asset.name);
                    true
                }
            };
        }
        match self.online.download(asset, want_k, then) {
            Ok(_) => {
                self.status = format!("Downloading {} from {}…", asset.name, asset.source.label());
                true
            }
            Err(e) => {
                self.status = e;
                false
            }
        }
    }

    pub(super) fn run_online_want(&mut self, want: Want) {
        match want {
            Want::Load(entry) => {
                self.browser_load(&entry, Target::Default);
            }
            Want::Download(entry, k) => {
                if let Some(asset) = &entry.online {
                    self.online_get(asset, k, Then::Keep);
                }
            }
            Want::Remove(entry) => {
                if let Some(asset) = &entry.online {
                    self.status = match self.online.remove_download(asset) {
                        Ok(()) => format!("Removed {}'s downloaded files", asset.name),
                        Err(e) => e,
                    };
                }
            }
            Want::Reveal(path) => self.browser_reveal(&path),
            Want::Page(url) => {
                if let Err(e) = self.open_with_default_app(std::ffi::OsStr::new(&url), &url) {
                    self.status = format!("Couldn't open {url} ({e})");
                }
            }
        }
    }

    /// The online roots' rows in the browser's tree, after the Basics:
    /// read-only, no remove toggle, no drag.
    pub(super) fn online_root_rows(
        &mut self,
        rows: &mut Vec<TreeRow<u64>>,
        keys: &mut HashMap<u64, (PathBuf, bool)>,
    ) {
        for (root, source) in self.preset_browser.online_roots.clone() {
            let key = crate::preset_browser_ui::key_of(&root);
            keys.insert(key, (root.clone(), true));
            let open = self.preset_browser.expanded.contains(&root);
            let children = self.preset_browser.subfolders(&root);
            let mut row = TreeRow::new(key, 0, source.label(), 2);
            row.icon = RowIcon::Icon(crate::icons::Icon::TreeFolder);
            row.disclosure = (!children.is_empty()).then_some(open);
            row.style.bold = true;
            let mut tip = format!(
                "{}: free {} assets, downloaded when you use them; read-only\nCache: {}",
                source.credit(),
                online::LICENCE,
                root.display()
            );
            if self.online.refreshing.contains(&source) {
                tip += "\nUpdating the catalogue…";
            } else if let Some(e) = self.online.errors.get(&source) {
                tip += &format!("\n{e}");
                row.style.dimmed = true;
            }
            row.tip = Some(tip);
            row.selected = if self.preset_browser.folder.as_ref() == Some(&root) {
                Selected::Primary
            } else {
                Selected::No
            };
            row.cells[0] = Cell::Empty;
            rows.push(row);
            if open {
                self.folder_rows(&root, 1, rows, keys);
            }
        }
    }

    /// Over an online library's contents: its credit (as Poly Haven's
    /// terms ask: near its content, never implying endorsement), when the
    /// catalogue was fetched and Refresh Catalogue. Returns true when
    /// there's no catalogue yet and it said why (nothing else to draw).
    pub(super) fn online_contents_header(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(folder) = self.preset_browser.folder.clone() else {
            return false;
        };
        let Some((_, source)) = self
            .preset_browser
            .online_roots
            .iter()
            .find(|(r, _)| folder.starts_with(r))
            .cloned()
        else {
            return false;
        };
        let catalogue = self.online.catalogue(source);
        let refreshing = self.online.refreshing.contains(&source);
        let mut refresh = false;
        ui.horizontal(|ui| {
            ui.hyperlink_to(source.credit(), source.home())
                .on_hover_text(format!(
                    "{} · every asset is {}, free for any use",
                    source.home(),
                    online::LICENCE
                ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if refreshing {
                    ui.spinner();
                    ui.weak("Updating catalogue…");
                } else {
                    refresh = ui
                        .button("Refresh Catalogue")
                        .on_hover_text(format!(
                            "Fetches {}'s list of assets again (operandi.online.refresh)",
                            source.label()
                        ))
                        .clicked();
                    if let Some(c) = &catalogue {
                        ui.weak(fetched_label(c.fetched));
                    }
                }
            });
        });
        if refresh {
            self.online.refresh(source);
        }
        if catalogue.is_some() {
            return false;
        }
        ui.add_space(6.);
        if refreshing {
            ui.weak(format!("Fetching {}'s catalogue…", source.label()));
        } else if let Some(e) = self.online.errors.get(&source) {
            ui.weak(format!("{e}. Refresh Catalogue tries again."));
        } else {
            ui.weak("No catalogue yet: Refresh Catalogue fetches it.");
        }
        true
    }

    /// An online tile: ask for its thumbnail, and say on the picture what
    /// is cached ("2K") or how far a download is.
    pub(super) fn online_tile_badge(&mut self, ui: &egui::Ui, entry: &Entry, picture: egui::Rect) {
        let Some(asset) = &entry.online else {
            return;
        };
        if let Some(root) = online::root_of(&entry.path, asset) {
            self.online.want_thumbnail(&root, asset);
        }
        let text = match self.online.downloads.get(&key_of(asset)) {
            Some(d) => match d.progress() {
                Some(p) => format!("{:.0} %", p * 100.),
                None => "starting…".into(),
            },
            None => match self.online.have(asset).as_slice() {
                [] => "online".into(),
                ks => ks
                    .iter()
                    .map(|k| format!("{k}K"))
                    .collect::<Vec<_>>()
                    .join(" "),
            },
        };
        ui.painter().text(
            picture.left_bottom() + egui::vec2(3., -3.),
            egui::Align2::LEFT_BOTTOM,
            text,
            egui::FontId::proportional(9.),
            ui.visuals().weak_text_color(),
        );
    }

    /// An online tile's tooltip lines: licence, authors, source and what's
    /// cached.
    pub(super) fn online_tip(&self, entry: &Entry) -> Option<String> {
        let asset = entry.online.as_ref()?;
        let mut tip = String::new();
        if !asset.category.is_empty() {
            tip += &format!("\n{}", asset.category);
        }
        for line in asset.credit_lines() {
            tip += &format!("\n{line}");
        }
        let want = online::pick_k(
            &self.online.resolutions(asset),
            self.online.prefs.resolution,
        );
        tip += &match (
            self.online.downloads.get(&key_of(asset)),
            self.online.have(asset),
        ) {
            (Some(d), _) => format!(
                "\nDownloading{}",
                d.progress()
                    .map(|p| format!(" ({:.0} %)", p * 100.))
                    .unwrap_or_default()
            ),
            (None, have) if have.is_empty() => format!(
                "\nNot downloaded: using it downloads it{}",
                want.map(|k| format!(" at {k}K")).unwrap_or_default()
            ),
            (None, have) => format!(
                "\nDownloaded: {}",
                have.iter()
                    .map(|k| format!("{k}K"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
        Some(tip)
    }

    /// Preferences > Defaults > Application > Online Libraries.
    pub(super) fn online_preferences_ui(&mut self, ui: &mut egui::Ui) {
        use crate::form::rows;
        let d = crate::online_library::OnlinePreferences::default();
        let before = self.online.prefs.clone();
        let size = self.online.cache_size();
        let folder = self.online.folder.clone();
        let mut reveal = false;
        let mut clear = false;
        let p = &mut self.online.prefs;
        let resolutions: Vec<(u32, String)> = online::RESOLUTIONS
            .iter()
            .map(|&k| (k, format!("{k}K")))
            .collect();
        let resolutions: Vec<(u32, &str)> =
            resolutions.iter().map(|(k, l)| (*k, l.as_str())).collect();
        let refreshes: Vec<_> = crate::online_library::Refresh::ALL
            .iter()
            .map(|&r| (r, r.label()))
            .collect();
        rows::section(ui, "Online Libraries", false, |ui| {
            // UIC-87: box then text, no left label.
            rows::check_d(
                ui,
                "Poly Haven",
                &mut p.poly_haven,
                d.poly_haven,
                "Shows Poly Haven's free CC0 HDRIs, textures and models in the Preset Browser. \
                 While off, Operandi never contacts polyhaven.com",
            );
            rows::check_d(
                ui,
                "ambientCG",
                &mut p.ambient_cg,
                d.ambient_cg,
                "Shows ambientCG's free CC0 HDRIs and materials in the Preset Browser. While \
                 off, Operandi never contacts ambientcg.com",
            );
            rows::choice_d(
                ui,
                "Download Resolution",
                &mut p.resolution,
                &d.resolution,
                &resolutions,
                "The size an asset downloads at when you use it: the nearest the library offers \
                 at or below it. Right-click a tile for Download At to pick another",
            );
            rows::choice_d(
                ui,
                "Catalogue Refresh",
                &mut p.refresh,
                &d.refresh,
                &refreshes,
                "How often each library's list of assets is fetched again. Refresh Catalogue in \
                 the Preset Browser fetches it at once",
            );
            rows::value(
                ui,
                "Cache",
                &format!("{} ({})", folder.display(), size_label(size)),
            )
            .on_hover_text(
                "Catalogues, thumbnails and downloaded files. Whatever is here keeps working \
                 offline",
            );
            reveal = rows::button(ui, "Show Cache Folder")
                .on_hover_text("Shows the cache folder in the file manager")
                .clicked();
            clear = rows::button(ui, "Clear Cache")
                .on_hover_text(
                    "Deletes downloaded files and thumbnails (operandi.online.clearCache). A \
                     library that's off loses its whole folder; nothing else is touched",
                )
                .clicked();
            rows::note(
                ui,
                "Nothing is fetched until a library is on. Then its catalogue is fetched, \
                 thumbnails as you browse, and an asset's files only when you use it. No \
                 account; nothing about you is sent.",
            );
        });
        if reveal {
            let _ = std::fs::create_dir_all(&folder);
            self.browser_reveal(&folder);
        }
        if clear {
            self.status = match self.online.clear_cache(None) {
                Ok(m) | Err(m) => m,
            };
        }
        if self.online.prefs != before
            && let Err(e) = self
                .online
                .prefs
                .save(&self.environment.paths().online_preferences())
        {
            self.status = format!("Preference changed; could not save it: {e}");
        }
    }

    /// The Application page's Reset: every library off, defaults back.
    pub(super) fn reset_online_preferences(&mut self) -> std::io::Result<()> {
        self.online.prefs = Default::default();
        self.online
            .prefs
            .save(&self.environment.paths().online_preferences())
    }

    /// `operandi.online.refresh|download|clearCache`.
    pub(super) fn online_command(&mut self, name: &str, rest: &str) {
        self.status = match self.online_command_result(name, rest) {
            Ok(m) | Err(m) => m,
        };
    }

    fn online_command_result(
        &mut self,
        name: &str,
        rest: &str,
    ) -> std::result::Result<String, String> {
        let tokens = command::tokens(rest)?;
        let bind = |names: &[&str]| operandi_core::command_args::bind(name, &tokens, names);
        let source = |v: Option<&str>| -> std::result::Result<Option<SourceId>, String> {
            v.map(|s| {
                SourceId::from_slug(s).ok_or(format!("{name}: source is polyhaven or ambientcg"))
            })
            .transpose()
        };
        match name {
            "operandi.online.refresh" => {
                let b = bind(&["source"])?;
                let only = source(b[0])?;
                let sources: Vec<SourceId> = self
                    .online
                    .prefs
                    .sources()
                    .into_iter()
                    .filter(|s| only.is_none_or(|o| o == *s))
                    .collect();
                if sources.is_empty() {
                    return Err(off_note(only));
                }
                for s in &sources {
                    self.online.refresh(*s);
                }
                Ok(format!(
                    "Fetching the catalogue of {}…",
                    sources
                        .iter()
                        .map(|s| s.label())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
            "operandi.online.clearCache" => {
                let b = bind(&["source"])?;
                self.online.clear_cache(source(b[0])?)
            }
            "operandi.online.download" => {
                let b = bind(&["source", "kind", "id", "resolution", "load"])?;
                let s =
                    source(b[0])?.ok_or(format!("{name} needs source:<polyhaven|ambientcg>"))?;
                let kind = b[1]
                    .map(|k| {
                        AssetKind::from_slug(k)
                            .ok_or(format!("{name}: kind is hdri, texture or model"))
                    })
                    .transpose()?
                    .ok_or(format!("{name} needs kind:<hdri|texture|model>"))?;
                let id = b[2].ok_or(format!("{name} needs id:<asset id>"))?;
                let k = match b[3] {
                    None => self.online.prefs.resolution,
                    Some(v) => v
                        .trim_end_matches(['k', 'K'])
                        .parse()
                        .map_err(|_| format!("{name}: resolution is a number of K, such as 2"))?,
                };
                let load = match b[4].map(|v| v.to_ascii_lowercase()) {
                    None => false,
                    Some(v) if v == "true" || v == "1" => true,
                    Some(v) if v == "false" || v == "0" => false,
                    Some(_) => return Err(format!("{name}: load is true or false")),
                };
                if !self.online.prefs.enabled(s) {
                    return Err(off_note(Some(s)));
                }
                let asset = self
                    .online
                    .catalogue(s)
                    .and_then(|c| c.find(kind, id).cloned())
                    .ok_or(format!(
                        "{} has no {} named {id} in its cached catalogue",
                        s.label(),
                        kind.slug()
                    ))?;
                let then = if load {
                    Then::Load(Target::Default)
                } else {
                    Then::Keep
                };
                if self.online_get(&asset, k, then) {
                    Ok(self.status.clone())
                } else {
                    Err(self.status.clone())
                }
            }
            _ => Err(format!("Unknown command {name}")),
        }
    }
}

impl Operandi {
    /// The MCP bridge's `online_library` tool (running app only).
    pub(super) fn mcp_online_library(
        &mut self,
        args: &serde_json::Value,
    ) -> std::result::Result<serde_json::Value, String> {
        use serde_json::{Value, json};
        const KEYS: [&str; 8] = [
            "action",
            "source",
            "kind",
            "id",
            "search",
            "limit",
            "resolution",
            "load",
        ];
        if let Some(map) = args.as_object()
            && let Some(bad) = map.keys().find(|k| !KEYS.contains(&k.as_str()))
        {
            return Err(format!("online_library doesn't take {bad}"));
        }
        let text = |k: &str| args.get(k).and_then(Value::as_str);
        let source = match text("source") {
            None => None,
            Some(s) => Some(SourceId::from_slug(s).ok_or("source is polyhaven or ambientcg")?),
        };
        let kind = match text("kind") {
            None => None,
            Some(k) => Some(AssetKind::from_slug(k).ok_or("kind is hdri, texture or model")?),
        };
        let action = text("action").ok_or("online_library needs an action")?;
        let asset_json = |a: &mut Self, asset: &Asset| -> Value {
            let root = a.online.root(asset.source);
            let have = online::downloaded(&root, asset);
            json!({
                "source": asset.source.slug(),
                "kind": asset.kind.slug(),
                "id": asset.id,
                "name": asset.name,
                "category": asset.category,
                "authors": asset.authors,
                "licence": online::LICENCE,
                "resolutions": a.online.resolutions(asset),
                "downloaded": have.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
                "file": have.last().map(|(_, p)| p.display().to_string()),
            })
        };
        match action {
            "status" => {}
            "enable" | "disable" => {
                let s = source.ok_or("enable and disable need a source")?;
                self.online.prefs.turn_on(s, action == "enable");
                self.online
                    .prefs
                    .save(&self.environment.paths().online_preferences())
                    .map_err(|e| format!("Couldn't save the preference: {e}"))?;
                self.preset_browser.set_online_roots(self.online.roots());
                // On: its catalogue is fetched now if there's none or it's old.
                self.online.keep_fresh();
            }
            "refresh" => self
                .online_command_result(
                    "operandi.online.refresh",
                    &match source {
                        Some(s) => format!("source:{}", s.slug()),
                        None => String::new(),
                    },
                )
                .map(drop)?,
            "list" => {
                let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
                let mut found = self.online.find(source, kind, text("search").unwrap_or(""));
                let total = found.len();
                found.truncate(limit);
                let assets: Vec<Value> = found.iter().map(|a| asset_json(self, a)).collect();
                return Ok(json!({"total": total, "assets": assets}));
            }
            "get" => {
                let s = source.ok_or("get needs source, kind and id")?;
                let k = kind.ok_or("get needs source, kind and id")?;
                let id = text("id").ok_or("get needs source, kind and id")?;
                let mut line = format!("source:{} kind:{} id:{id}", s.slug(), k.slug());
                if let Some(r) = args.get("resolution") {
                    line += &format!(
                        " resolution:{}",
                        r.as_u64().ok_or("resolution is a whole number of K")?
                    );
                }
                let load = args.get("load").and_then(Value::as_bool).unwrap_or(false);
                line += &format!(" load:{load}");
                let message = self.online_command_result("operandi.online.download", &line)?;
                let asset = self
                    .online
                    .catalogue(s)
                    .and_then(|c| c.find(k, id).cloned())
                    .ok_or("not in the catalogue")?;
                let mut out = asset_json(self, &asset);
                out["message"] = json!(message);
                out["state"] = json!(match self.online.downloads.get(&key_of(&asset)) {
                    Some(_) => "downloading",
                    None => "ready",
                });
                if let Some(d) = self.online.downloads.get(&key_of(&asset)) {
                    out["progress"] = json!(d.progress());
                }
                return Ok(out);
            }
            other => return Err(format!("online_library has no action {other}")),
        }
        let sources: Vec<Value> = SourceId::ALL
            .into_iter()
            .map(|s| {
                let on = self.online.prefs.enabled(s);
                let catalogue = if on { self.online.catalogue(s) } else { None };
                json!({
                    "source": s.slug(),
                    "name": s.label(),
                    "on": on,
                    "credit": s.credit(),
                    "assets": catalogue.as_ref().map(|c| c.assets.len()),
                    "fetched": catalogue.as_ref().map(|c| c.fetched),
                    "refreshing": self.online.refreshing.contains(&s),
                    "error": self.online.errors.get(&s),
                })
            })
            .collect();
        let downloads: Vec<Value> = self
            .online
            .downloads
            .values()
            .map(|d| {
                json!({
                    "source": d.asset.source.slug(),
                    "kind": d.asset.kind.slug(),
                    "id": d.asset.id,
                    "resolution": d.k,
                    "bytes": d.bytes,
                    "progress": d.progress(),
                })
            })
            .collect();
        Ok(json!({
            "sources": sources,
            "downloads": downloads,
            "resolution": self.online.prefs.resolution,
            "cache": self.online.folder.display().to_string(),
            "status": self.status,
        }))
    }
}

/// Why a command did nothing: no library (or not that one) is on.
fn off_note(only: Option<SourceId>) -> String {
    format!(
        "{} off; turn it on in Preferences > Defaults > Application > Online Libraries",
        match only {
            Some(s) => format!("{} is", s.label()),
            None => "Every online library is".into(),
        }
    )
}

/// "Updated today" / "Updated 3 days ago".
fn fetched_label(fetched: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    match now.saturating_sub(fetched) / 86_400 {
        0 => "Updated today".into(),
        1 => "Updated yesterday".into(),
        n => format!("Updated {n} days ago"),
    }
}
