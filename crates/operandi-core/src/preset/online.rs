//! Online libraries in the preset browser (master plan A34), the part
//! without a window or a network: what each source's catalogue says, where
//! its cache lives, and which files a download fetches. The app's
//! `online_library.rs` does the fetching (the system's `curl`, on
//! background threads) and puts the results on screen.
//!
//! - Every source is opt-in (Jason, 2026-11-20): one switch per source in
//!   Preferences, all off by default. A source that's off is never
//!   contacted and isn't shown. Nothing in this file touches the network.
//! - An enabled source is a read-only root of the browser whose folders
//!   are ordinary folders in the cache: one per asset kind (HDRIs,
//!   Materials, Models), one per category below, and in them one small
//!   `.oponline` stub per asset (its catalogue entry as JSON). So the tree,
//!   search, sorting and kind filter work as for any folder, and whatever
//!   was fetched last is there offline.
//! - Thumbnails go in the source's hidden `.thumbs` folder, downloads in
//!   `.downloads/<kind>/<id>/<n>k/`, finished by a `.complete` marker that
//!   names the file to load. A download is only ever made when the user
//!   uses an asset (or asks for it), at the resolution they picked.
//! - Sources (terms checked 2026-11-23): Poly Haven (api.polyhaven.com, no
//!   key, needs a unique User-Agent and a visible "Powered by Poly Haven"
//!   style credit near its content; assets CC0) and ambientCG
//!   (ambientcg.com/api/v3, GET only, no key; assets CC0, credit welcome).
//!   Our own client, written from their public API docs.
use super::library::EntryKind;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// An asset's stub file in the cache (its catalogue entry as JSON).
pub const STUB_EXTENSION: &str = "oponline";
/// Every asset from every source so far is CC0.
pub const LICENCE: &str = "CC0 1.0";
/// The cache's hidden folders and files, inside each source's root.
pub const THUMBS_DIR: &str = ".thumbs";
pub const DOWNLOADS_DIR: &str = ".downloads";
pub const CATALOGUE_FILE: &str = ".catalogue.json";
/// Written last into a download's folder: names the file to load.
pub const COMPLETE_FILE: &str = ".complete";
/// Beside an asset's downloads: the resolutions its source offers.
pub const OFFERED_FILE: &str = ".offered.json";
/// Resolutions a download can be asked for, in thousands of pixels.
pub const RESOLUTIONS: [u32; 5] = [1, 2, 4, 8, 16];
/// The catalogue format; an older or newer one is fetched again.
const CATALOGUE_VERSION: u32 = 1;

// ------------------------------------------------------------ sources

/// An online library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceId {
    PolyHaven,
    AmbientCg,
}

impl SourceId {
    pub const ALL: [Self; 2] = [Self::PolyHaven, Self::AmbientCg];

    /// The name commands and the MCP bridge use.
    pub fn slug(self) -> &'static str {
        match self {
            Self::PolyHaven => "polyhaven",
            Self::AmbientCg => "ambientcg",
        }
    }
    pub fn from_slug(text: &str) -> Option<Self> {
        let t: String = text
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        Self::ALL.into_iter().find(|s| s.slug() == t)
    }
    /// The source's own name, as it writes it (also its root's folder).
    pub fn label(self) -> &'static str {
        match self {
            Self::PolyHaven => "Poly Haven",
            Self::AmbientCg => "ambientCG",
        }
    }
    pub fn home(self) -> &'static str {
        match self {
            Self::PolyHaven => "https://polyhaven.com",
            Self::AmbientCg => "https://ambientcg.com",
        }
    }
    /// The credit shown near the source's content. Poly Haven's API terms
    /// ask for a small visible credit that doesn't imply endorsement;
    /// ambientCG asks for none, but is credited anyway.
    pub fn credit(self) -> &'static str {
        match self {
            Self::PolyHaven => "Powered by Poly Haven",
            Self::AmbientCg => "Assets from ambientCG.com",
        }
    }
    /// The About box's line.
    pub fn about_line(self) -> String {
        format!(
            "{}: free {} assets from {} (optional, off unless turned on in Preferences).",
            self.credit(),
            LICENCE,
            self.home().trim_start_matches("https://")
        )
    }
    /// What the source offers.
    pub fn kinds(self) -> &'static [AssetKind] {
        match self {
            Self::PolyHaven => &[AssetKind::Hdri, AssetKind::Texture, AssetKind::Model],
            Self::AmbientCg => &[AssetKind::Hdri, AssetKind::Texture],
        }
    }
    /// The first catalogue page for `kind`.
    pub fn catalogue_url(self, kind: AssetKind) -> String {
        match self {
            Self::PolyHaven => format!(
                "https://api.polyhaven.com/assets?type={}",
                match kind {
                    AssetKind::Hdri => "hdris",
                    AssetKind::Texture => "textures",
                    AssetKind::Model => "models",
                }
            ),
            Self::AmbientCg => ambientcg_url(kind, 0),
        }
    }
    /// The asset's page on the source's site (Open Web Page).
    pub fn asset_page(self, id: &str) -> String {
        match self {
            Self::PolyHaven => format!("https://polyhaven.com/a/{id}"),
            Self::AmbientCg => format!("https://ambientcg.com/view?id={id}"),
        }
    }
}

/// What kind of asset: where it goes in the browser and how it loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    /// A panorama: loads as the environment's image.
    Hdri,
    /// A set of texture maps: loads as a material.
    Texture,
    /// A glTF model: imported into the scene.
    Model,
}

impl AssetKind {
    pub const ALL: [Self; 3] = [Self::Hdri, Self::Texture, Self::Model];
    pub fn slug(self) -> &'static str {
        match self {
            Self::Hdri => "hdri",
            Self::Texture => "texture",
            Self::Model => "model",
        }
    }
    pub fn from_slug(text: &str) -> Option<Self> {
        let t = text.trim().to_ascii_lowercase();
        let t = t.trim_end_matches('s');
        Self::ALL.into_iter().find(|k| {
            k.slug() == t
                || matches!(
                    (k, t),
                    (Self::Texture, "material") | (Self::Hdri, "environment")
                )
        })
    }
    /// The browser folder the kind's assets are in.
    pub fn folder(self) -> &'static str {
        match self {
            Self::Hdri => "HDRIs",
            Self::Texture => "Materials",
            Self::Model => "Models",
        }
    }
    /// The entry kind its tiles show and its downloads load as.
    pub fn entry_kind(self) -> EntryKind {
        match self {
            Self::Hdri => EntryKind::Environment,
            Self::Texture => EntryKind::TextureSet,
            Self::Model => EntryKind::Model,
        }
    }
}

// ------------------------------------------------------------ catalogue

/// One asset as its source's catalogue lists it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub source: SourceId,
    pub kind: AssetKind,
    /// The source's id (Poly Haven's slug, ambientCG's asset id).
    pub id: String,
    pub name: String,
    /// The top-level category, the browser folder it's in ("" for none).
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    /// The thumbnail picture's address.
    #[serde(default)]
    pub thumbnail: String,
    /// The largest resolution offered, in thousands of pixels.
    #[serde(default)]
    pub max_k: Option<u32>,
    /// ambientCG: the archives the catalogue offers (Poly Haven's files
    /// are asked for when one is downloaded).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<CatalogueFile>,
}

/// One downloadable archive an ambientCG catalogue entry lists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CatalogueFile {
    /// ambientCG's attribute ("2K-JPG", "4K").
    pub label: String,
    /// Its resolution in thousands of pixels.
    pub k: u32,
    pub url: String,
    pub size: u64,
}

impl Asset {
    /// The resolutions it can be downloaded at, lowest first.
    pub fn resolutions(&self) -> Vec<u32> {
        let mut r: Vec<u32> = if self.files.is_empty() {
            RESOLUTIONS
                .iter()
                .copied()
                .filter(|&k| self.max_k.is_none_or(|m| k <= m))
                .collect()
        } else {
            self.files.iter().map(|f| f.k).collect()
        };
        r.sort_unstable();
        r.dedup();
        r
    }
    /// The tooltip's credit lines: licence, authors and source.
    pub fn credit_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("Licence: {LICENCE}")];
        if !self.authors.is_empty() {
            lines.push(format!("By {}", self.authors.join(", ")));
        }
        lines.push(self.source.credit().to_owned());
        lines
    }
    /// Whether a search for `text` (lower case) finds it: by name, id,
    /// category or tag.
    pub fn matches(&self, text: &str) -> bool {
        [&self.name, &self.id, &self.category]
            .into_iter()
            .chain(self.tags.iter())
            .any(|s| s.to_lowercase().contains(text))
    }
}

/// A source's whole catalogue as cached.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Catalogue {
    pub version: u32,
    pub source: SourceId,
    /// When it was fetched (seconds since 1970).
    pub fetched: u64,
    pub assets: Vec<Asset>,
}

impl Catalogue {
    pub fn new(source: SourceId, fetched: u64, assets: Vec<Asset>) -> Self {
        Self {
            version: CATALOGUE_VERSION,
            source,
            fetched,
            assets,
        }
    }
    /// The cached catalogue under `root` (a source's root), if readable.
    pub fn read(root: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(root.join(CATALOGUE_FILE)).ok()?;
        serde_json::from_str::<Self>(&text)
            .ok()
            .filter(|c| c.version == CATALOGUE_VERSION)
    }
    pub fn write(&self, root: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(root)?;
        let text = serde_json::to_string(self).map_err(std::io::Error::other)?;
        write_atomic(&root.join(CATALOGUE_FILE), text.as_bytes())
    }
    /// Old enough to fetch again: `max_age` seconds, or never with `None`.
    pub fn stale(&self, now: u64, max_age: Option<u64>) -> bool {
        max_age.is_some_and(|age| now.saturating_sub(self.fetched) >= age)
    }
    pub fn find(&self, kind: AssetKind, id: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.kind == kind && a.id == id)
    }
}

/// Poly Haven's `/assets?type=…` answer: an object keyed by slug.
pub fn parse_polyhaven(kind: AssetKind, json: &str) -> Result<Vec<Asset>, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Poly Haven's catalogue: {e}"))?;
    let map = value
        .as_object()
        .ok_or("Poly Haven's catalogue isn't a list of assets")?;
    let mut assets = Vec::with_capacity(map.len());
    for (id, a) in map {
        if !safe_name(id) {
            continue;
        }
        let text = |key: &str| a.get(key).and_then(|v| v.as_str()).unwrap_or("");
        let name = match text("name") {
            "" => id.clone(),
            n => n.to_owned(),
        };
        // `category` is a path ("Outdoor/Urban/…"): its first part is the
        // folder. Older entries only have the `categories` tags.
        let category = text("category")
            .split('/')
            .next()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                a.get("categories")?
                    .as_array()?
                    .iter()
                    .filter_map(|c| c.as_str())
                    .find(|c| !c.is_empty())
                    .map(title_case)
            })
            .unwrap_or_default();
        let strings = |key: &str| -> Vec<String> {
            a.get(key)
                .and_then(|v| v.as_array())
                .map(|l| {
                    l.iter()
                        .filter_map(|s| s.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut authors: Vec<String> = a
            .get("authors")
            .and_then(|v| v.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        authors.sort();
        let max_k = a
            .get("max_resolution")
            .and_then(|v| v.as_array())
            .and_then(|r| r.iter().filter_map(|n| n.as_u64()).max())
            .map(|px| (px as f64 / 1024.).round().max(1.) as u32);
        let thumbnail = match text("thumbnail_url") {
            "" => {
                format!("https://cdn.polyhaven.com/asset_img/thumbs/{id}.png?width=256&height=256")
            }
            t => t.to_owned(),
        };
        assets.push(Asset {
            source: SourceId::PolyHaven,
            kind,
            id: id.clone(),
            name,
            category,
            tags: strings("tags"),
            authors,
            thumbnail: polyhaven_thumbnail(&thumbnail),
            max_k,
            files: Vec::new(),
        });
    }
    assets.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(assets)
}

/// Poly Haven's thumbnail address asking for a 256-pixel JPEG (their CDN
/// sends WebP otherwise, which Operandi doesn't read).
fn polyhaven_thumbnail(url: &str) -> String {
    let (base, query) = url.split_once('?').unwrap_or((url, ""));
    let kept: Vec<&str> = query
        .split('&')
        .filter(|p| {
            !p.is_empty()
                && !p.starts_with("format=")
                && !p.starts_with("width=")
                && !p.starts_with("height=")
        })
        .collect();
    let mut q = vec!["width=256", "height=256", "format=jpg"];
    q.extend(kept);
    format!("{base}?{}", q.join("&"))
}

/// One ambientCG catalogue page (`/api/v3/assets`) for `kind`.
pub fn ambientcg_url(kind: AssetKind, offset: usize) -> String {
    let t = match kind {
        AssetKind::Hdri => "hdri",
        AssetKind::Texture => "material",
        AssetKind::Model => "3d-model",
    };
    format!(
        "https://ambientcg.com/api/v3/assets?type={t}&limit=250&offset={offset}&sort=alphabet&include=title,type,downloads,thumbnails"
    )
}

/// One ambientCG catalogue page: its assets, and the next page's address.
pub fn parse_ambientcg(
    kind: AssetKind,
    json: &str,
) -> Result<(Vec<Asset>, Option<String>), String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("ambientCG's catalogue: {e}"))?;
    let list = value
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or("ambientCG's catalogue has no assets list")?;
    let mut assets = Vec::with_capacity(list.len());
    for a in list {
        let Some(id) = a
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|i| safe_name(i))
        else {
            continue;
        };
        let name = a
            .get("title")
            .and_then(|v| v.as_str())
            .filter(|t| !t.is_empty())
            .unwrap_or(id)
            .to_owned();
        let mut files: Vec<CatalogueFile> = a
            .get("downloads")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|d| {
                let label = d.get("attributes")?.as_str()?.to_owned();
                let url = d.get("url")?.as_str()?.to_owned();
                if d.get("extension").and_then(|e| e.as_str()) != Some("zip")
                    || !url.starts_with("https://")
                {
                    return None;
                }
                // Materials come as JPG and PNG sets: JPG is a fraction
                // of the size and looks the same in renders.
                if kind == AssetKind::Texture && !label.to_ascii_uppercase().ends_with("-JPG") {
                    return None;
                }
                let k: u32 = label
                    .split(|c: char| !c.is_ascii_digit())
                    .next()?
                    .parse()
                    .ok()?;
                Some(CatalogueFile {
                    label,
                    k,
                    url,
                    size: d.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                })
            })
            .collect();
        files.sort_by_key(|f| f.k);
        files.dedup_by_key(|f| f.k);
        if files.is_empty() {
            continue;
        }
        let thumbs = a.get("thumbnails").and_then(|v| v.as_object());
        let thumbnail = ["256-JPG-242424", "256-JPG-FFFFFF", "256-PNG", "512-PNG"]
            .iter()
            .find_map(|k| thumbs?.get(*k)?.as_str())
            .unwrap_or("")
            .to_owned();
        let category = if kind == AssetKind::Texture {
            ambientcg_category(id)
        } else {
            String::new()
        };
        assets.push(Asset {
            source: SourceId::AmbientCg,
            kind,
            id: id.to_owned(),
            name,
            category,
            tags: Vec::new(),
            authors: Vec::new(),
            thumbnail,
            max_k: files.last().map(|f| f.k),
            files,
        });
    }
    let next = value
        .get("nextPageHttp")
        .and_then(|v| v.as_str())
        .filter(|n| n.starts_with("https://ambientcg.com/"))
        .map(str::to_owned);
    Ok((assets, next))
}

/// ambientCG's ids are a kind of material and a number ("PavingStones142"):
/// the words are its category ("Paving Stones").
fn ambientcg_category(id: &str) -> String {
    let letters: String = id.chars().take_while(|c| !c.is_ascii_digit()).collect();
    let mut out = String::new();
    for (i, c) in letters.chars().enumerate() {
        if i > 0 && c.is_ascii_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

fn title_case(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// An id or relative file name from a source that is safe as one path
/// part: no separators, no `..`, nothing hidden.
fn safe_name(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 200
        && !text.starts_with('.')
        && !text.contains(['/', '\\', ':', '\0'])
}

// ------------------------------------------------------------ cache

/// A source's root folder under the online libraries folder.
pub fn source_root(libraries: &Path, source: SourceId) -> PathBuf {
    libraries.join(source.label())
}

/// A name made safe as a file or folder name on every platform.
pub fn file_name_for(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    let cut: String = cleaned.chars().take(80).collect();
    if cut.is_empty() {
        "Untitled".into()
    } else {
        cut
    }
}

/// Where `asset`'s stub goes under its source's root.
pub fn stub_path(root: &Path, asset: &Asset) -> PathBuf {
    let mut dir = root.join(asset.kind.folder());
    if !asset.category.is_empty() {
        dir = dir.join(file_name_for(&asset.category));
    }
    dir.join(format!("{}.{STUB_EXTENSION}", file_name_for(&asset.name)))
}

/// The source root a stub at `stub` is under: the folder above its kind's
/// folder, which is named for the source.
pub fn root_of(stub: &Path, asset: &Asset) -> Option<PathBuf> {
    stub.ancestors()
        .find(|a| {
            a.file_name().is_some_and(|n| n == asset.kind.folder())
                && a.parent()
                    .and_then(Path::file_name)
                    .is_some_and(|n| n == asset.source.label())
        })
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

/// Where `asset`'s thumbnail is cached.
pub fn thumb_path(root: &Path, asset: &Asset) -> PathBuf {
    root.join(THUMBS_DIR)
        .join(format!("{}-{}.jpg", asset.kind.slug(), asset.id))
}

/// `asset`'s downloads folder (every resolution).
pub fn downloads_dir(root: &Path, asset: &Asset) -> PathBuf {
    root.join(DOWNLOADS_DIR)
        .join(asset.kind.slug())
        .join(&asset.id)
}

/// The folder of `asset`'s download at `k` thousand pixels.
pub fn download_dir(root: &Path, asset: &Asset, k: u32) -> PathBuf {
    downloads_dir(root, asset).join(format!("{k}k"))
}

/// The finished downloads of `asset`: each resolution and its file to
/// load, lowest first.
pub fn downloaded(root: &Path, asset: &Asset) -> Vec<(u32, PathBuf)> {
    let mut found = Vec::new();
    let Ok(dirs) = std::fs::read_dir(downloads_dir(root, asset)) else {
        return found;
    };
    for dir in dirs.flatten() {
        let name = dir.file_name().to_string_lossy().into_owned();
        let Some(k) = name.strip_suffix('k').and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let Ok(main) = std::fs::read_to_string(dir.path().join(COMPLETE_FILE)) else {
            continue;
        };
        if let Some(rel) = safe_rel(main.trim()) {
            let path = dir.path().join(rel);
            if path.is_file() {
                found.push((k, path));
            }
        }
    }
    found.sort();
    found
}

/// The downloaded file to load for a wanted resolution: the one at
/// [`pick_k`]'s choice among what's downloaded.
pub fn downloaded_at(root: &Path, asset: &Asset, want_k: u32) -> Option<(u32, PathBuf)> {
    let have = downloaded(root, asset);
    let ks: Vec<u32> = have.iter().map(|(k, _)| *k).collect();
    let k = pick_k(&ks, want_k)?;
    have.into_iter().find(|(h, _)| *h == k)
}

/// The resolutions the source said it offers for `asset`, once a
/// download has read its file list (the catalogue only gives the largest).
pub fn offered(root: &Path, asset: &Asset) -> Option<Vec<u32>> {
    let text = std::fs::read_to_string(downloads_dir(root, asset).join(OFFERED_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Note what the source offers for `asset` (see [`offered`]).
pub fn write_offered(root: &Path, asset: &Asset, ks: &[u32]) -> std::io::Result<()> {
    let dir = downloads_dir(root, asset);
    std::fs::create_dir_all(&dir)?;
    let text = serde_json::to_string(ks).map_err(std::io::Error::other)?;
    write_atomic(&dir.join(OFFERED_FILE), text.as_bytes())
}

/// Mark a download finished: `main` (relative to `dir`) is what loads.
pub fn finish_download(dir: &Path, main: &Path) -> std::io::Result<()> {
    let rel = main.strip_prefix(dir).unwrap_or(main);
    write_atomic(
        &dir.join(COMPLETE_FILE),
        rel.to_string_lossy().replace('\\', "/").as_bytes(),
    )
}

/// The resolution to fetch from `available`: the highest at or below
/// `want_k`, else the lowest offered.
pub fn pick_k(available: &[u32], want_k: u32) -> Option<u32> {
    available
        .iter()
        .copied()
        .filter(|&k| k <= want_k)
        .max()
        .or_else(|| available.iter().copied().min())
}

/// A relative path from a source (a glTF's textures folder) made safe to
/// join under a download folder: no root, no `..`, nothing empty.
pub fn safe_rel(text: &str) -> Option<PathBuf> {
    let text = text.replace('\\', "/");
    let path = Path::new(&text);
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

/// What a sync of the stubs did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub written: usize,
    pub removed: usize,
}

/// Make the stubs under `root` match `assets`: write new and changed ones,
/// remove ones the catalogue no longer lists and folders left empty. Only
/// `.oponline` files and empty folders are ever removed.
pub fn sync_stubs(root: &Path, assets: &[Asset]) -> std::io::Result<SyncReport> {
    let mut report = SyncReport::default();
    let mut wanted: std::collections::HashMap<PathBuf, Vec<u8>> = Default::default();
    for asset in assets {
        let mut path = stub_path(root, asset);
        // Two assets with one name in one folder: the second says its id.
        if wanted.contains_key(&path) {
            path = path.with_file_name(format!(
                "{} ({}).{STUB_EXTENSION}",
                file_name_for(&asset.name),
                file_name_for(&asset.id)
            ));
        }
        let json = serde_json::to_vec_pretty(asset).map_err(std::io::Error::other)?;
        wanted.insert(path, json);
    }
    for kind in AssetKind::ALL {
        let dir = root.join(kind.folder());
        for stub in stubs_below(&dir) {
            if !wanted.contains_key(&stub) {
                std::fs::remove_file(&stub)?;
                report.removed += 1;
            }
        }
    }
    for (path, json) in &wanted {
        if std::fs::read(path).ok().as_deref() == Some(json.as_slice()) {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_atomic(path, json)?;
        report.written += 1;
    }
    for kind in AssetKind::ALL {
        remove_empty_dirs(&root.join(kind.folder()), true);
    }
    Ok(report)
}

/// Every stub file in and below `dir`.
fn stubs_below(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in read.flatten() {
        let path = e.path();
        if path.is_dir() {
            out.extend(stubs_below(&path));
        } else if is_stub(&path) {
            out.push(path);
        }
    }
    out
}

/// Remove `dir`'s empty subfolders (and `dir` itself unless `keep`).
fn remove_empty_dirs(dir: &Path, keep: bool) {
    if let Ok(read) = std::fs::read_dir(dir) {
        for e in read.flatten() {
            if e.path().is_dir() {
                remove_empty_dirs(&e.path(), false);
            }
        }
    }
    if !keep {
        // Fails (and is left) when anything is still inside.
        let _ = std::fs::remove_dir(dir);
    }
}

/// Whether `path` is an asset stub.
pub fn is_stub(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(STUB_EXTENSION))
}

/// The asset a stub file describes.
pub fn read_stub(path: &Path) -> Option<Asset> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Bytes in the files in and below `dir`.
pub fn folder_size(dir: &Path) -> u64 {
    let Ok(read) = std::fs::read_dir(dir) else {
        return 0;
    };
    read.flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => folder_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Write `bytes` to `path` by way of a temporary file beside it, so a
/// reader never sees half a file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension(format!(
        "{}.part",
        path.extension()
            .map(|e| e.to_string_lossy())
            .unwrap_or_default()
    ));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

// ------------------------------------------------------------ downloads

/// One file of a download: where from, where to (relative to the
/// download's folder) and how big the source says it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanFile {
    pub url: String,
    pub rel: PathBuf,
    pub size: u64,
}

/// What downloading an asset at one resolution fetches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub k: u32,
    pub files: Vec<PlanFile>,
    /// The file that loads, relative to the download's folder; `None` when
    /// it's only known after unpacking an archive ([`unpack`]).
    pub main: Option<PathBuf>,
    /// The one file is a ZIP archive to unpack.
    pub archive: bool,
    /// Every resolution the source offers for it, lowest first.
    pub offered: Vec<u32>,
}

impl Plan {
    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

/// The address of Poly Haven's file list for `asset`.
pub fn polyhaven_files_url(asset: &Asset) -> String {
    format!("https://api.polyhaven.com/files/{}", asset.id)
}

/// Poly Haven texture maps a material is made of, in the names their file
/// list uses, with the format each is fetched in. Their packed `arm`, the
/// DirectX normal and the other packed or special maps are left out: the
/// texture-set loader would only pick one of each kind anyway.
const POLYHAVEN_MAPS: [&str; 8] = [
    "Diffuse",
    "nor_gl",
    "Rough",
    "Displacement",
    "AO",
    "Metal",
    "Opacity",
    "Translucent",
];

/// The files to fetch for `asset` at about `want_k`, from Poly Haven's
/// `/files/<id>` answer.
pub fn polyhaven_plan(asset: &Asset, files_json: &str, want_k: u32) -> Result<Plan, String> {
    let value: serde_json::Value =
        serde_json::from_str(files_json).map_err(|e| format!("Poly Haven's file list: {e}"))?;
    let file = |v: &serde_json::Value, rel: PathBuf| -> Option<PlanFile> {
        let url = v.get("url")?.as_str()?;
        url.starts_with("https://").then(|| PlanFile {
            url: url.to_owned(),
            rel,
            size: v.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
        })
    };
    // "1k" → 1; each kind's resolutions are the keys of one object.
    let resolutions = |v: &serde_json::Value| -> Vec<u32> {
        v.as_object()
            .map(|o| {
                o.keys()
                    .filter_map(|k| k.strip_suffix('k')?.parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    };
    let missing = || format!("Poly Haven lists no files for {}", asset.name);
    match asset.kind {
        AssetKind::Hdri => {
            let hdri = value.get("hdri").ok_or_else(missing)?;
            let offered = sorted(resolutions(hdri));
            let k = pick_k(&offered, want_k).ok_or_else(missing)?;
            let at = &hdri[format!("{k}k")];
            // .hdr first: a fraction of the EXR's size, same pixels.
            let (v, ext) = ["hdr", "exr"]
                .iter()
                .find_map(|ext| at.get(*ext).map(|v| (v, *ext)))
                .ok_or_else(missing)?;
            let rel = PathBuf::from(format!("{}_{k}k.{ext}", asset.id));
            let f = file(v, rel.clone()).ok_or_else(missing)?;
            Ok(Plan {
                k,
                files: vec![f],
                main: Some(rel),
                archive: false,
                offered,
            })
        }
        AssetKind::Texture => {
            let base = value.get("Diffuse").ok_or_else(missing)?;
            let offered = sorted(resolutions(base));
            let k = pick_k(&offered, want_k).ok_or_else(missing)?;
            let mut files = Vec::new();
            let mut main = None;
            for map in POLYHAVEN_MAPS {
                let Some(at) = value.get(map).and_then(|m| m.get(format!("{k}k"))) else {
                    continue;
                };
                // Displacement as PNG keeps 16 bits (the texture-set loader
                // doesn't read their EXRs); the rest as JPG.
                let order: &[&str] = if map == "Displacement" {
                    &["png", "jpg"]
                } else {
                    &["jpg", "png"]
                };
                let Some(v) = order.iter().find_map(|f| at.get(*f)) else {
                    continue;
                };
                let Some(name) = url_file_name(v).filter(|n| safe_name(n)) else {
                    continue;
                };
                if let Some(f) = file(v, PathBuf::from(&name)) {
                    if map == "Diffuse" {
                        main = Some(f.rel.clone());
                    }
                    files.push(f);
                }
            }
            if main.is_none() {
                return Err(missing());
            }
            Ok(Plan {
                k,
                files,
                main,
                archive: false,
                offered,
            })
        }
        AssetKind::Model => {
            let gltf = value.get("gltf").ok_or_else(missing)?;
            let offered = sorted(resolutions(gltf));
            let k = pick_k(&offered, want_k).ok_or_else(missing)?;
            let v = gltf
                .get(format!("{k}k"))
                .and_then(|r| r.get("gltf"))
                .ok_or_else(missing)?;
            let name = url_file_name(v)
                .filter(|n| safe_name(n))
                .unwrap_or_else(|| format!("{}.gltf", asset.id));
            let rel = PathBuf::from(name);
            let mut files = vec![file(v, rel.clone()).ok_or_else(missing)?];
            for (path, inc) in v
                .get("include")
                .and_then(|i| i.as_object())
                .into_iter()
                .flatten()
            {
                let Some(rel) = safe_rel(path) else {
                    return Err(format!("Poly Haven listed an unsafe file name: {path}"));
                };
                files.extend(file(inc, rel));
            }
            Ok(Plan {
                k,
                files,
                main: Some(rel),
                archive: false,
                offered,
            })
        }
    }
}

fn sorted(mut ks: Vec<u32>) -> Vec<u32> {
    ks.sort_unstable();
    ks.dedup();
    ks
}

/// The last part of a file entry's address ("…/rock_diff_2k.jpg").
fn url_file_name(v: &serde_json::Value) -> Option<String> {
    let url = v.get("url")?.as_str()?;
    let path = url.split(['?', '#']).next()?;
    let name = path.rsplit('/').next()?;
    let name = name.replace("%20", " ").replace("%2B", "+");
    (!name.is_empty()).then_some(name)
}

/// The archive to fetch for an ambientCG `asset` at about `want_k`.
pub fn ambientcg_plan(asset: &Asset, want_k: u32) -> Result<Plan, String> {
    let ks: Vec<u32> = asset.files.iter().map(|f| f.k).collect();
    let k = pick_k(&ks, want_k)
        .ok_or_else(|| format!("ambientCG lists no files for {}", asset.name))?;
    let f = asset
        .files
        .iter()
        .find(|f| f.k == k)
        .expect("picked from the list");
    Ok(Plan {
        k,
        files: vec![PlanFile {
            url: f.url.clone(),
            rel: PathBuf::from(format!(
                "{}_{}.zip",
                file_name_for(&asset.id),
                file_name_for(&f.label)
            )),
            size: f.size,
        }],
        main: None,
        archive: true,
        offered: asset.resolutions(),
    })
}

/// Unpack an ambientCG archive's useful files into `dir` and return the
/// file to load: the panorama of an HDRI, a material's colour map. Its
/// Blender, USD and Godot files and its preview picture are left out.
pub fn unpack(archive: &[u8], dir: &Path, asset: &Asset) -> Result<PathBuf, String> {
    let entries = crate::interchange::zip::read(archive)?;
    let mut main: Option<(u8, PathBuf)> = None;
    for (name, bytes) in entries {
        let Some(rel) = safe_rel(&name) else {
            continue;
        };
        let Some(file) = rel.file_name().map(|f| f.to_string_lossy().into_owned()) else {
            continue;
        };
        let lower = file.to_ascii_lowercase();
        let ext = lower.rsplit('.').next().unwrap_or("");
        let picture = super::library::IMAGE_EXTENSIONS.contains(&ext);
        // `<Id>.png` is the site's preview render, not a map.
        let preview = lower == format!("{}.png", asset.id.to_ascii_lowercase());
        if !picture || preview || lower.contains("tonemapped") {
            continue;
        }
        let rank = match asset.kind {
            AssetKind::Hdri if ext == "exr" || ext == "hdr" => 1,
            AssetKind::Texture if lower.contains("_color") => 1,
            AssetKind::Texture => 2,
            _ => 3,
        };
        let out = dir.join(&file);
        write_atomic(&out, &bytes).map_err(|e| format!("Couldn't write {}: {e}", out.display()))?;
        if main.as_ref().is_none_or(|(r, _)| rank < *r) {
            main = Some((rank, out));
        }
    }
    match main {
        Some((rank, path)) if asset.kind != AssetKind::Hdri || rank == 1 => Ok(path),
        _ => Err(format!(
            "{}'s archive holds nothing Operandi can load",
            asset.name
        )),
    }
}

#[cfg(test)]
mod tests;
