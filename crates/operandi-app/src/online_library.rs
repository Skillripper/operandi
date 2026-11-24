//! Online libraries in the preset browser (master plan A34): the
//! preferences, and the background work that talks to the sources. What
//! the catalogues say and where things are cached is core's
//! `preset::online`; `online_library_ui.rs` puts it all on screen.
//!
//! - Opt-in (Jason, 2026-11-20): Preferences > Defaults > Application >
//!   Online Libraries has one switch per source, all off by default. Until
//!   a source is on, nothing here runs for it and it isn't in the browser.
//! - Each source's catalogue is fetched when it's first turned on, then
//!   again when it's older than the Catalogue Refresh setting (weekly by
//!   default) or on Refresh Catalogue. Opening the browser reads the cache.
//! - Thumbnails are fetched only for tiles on screen, one at a time; full
//!   files only when an asset is used or a download is asked for, at the
//!   Download Resolution. Everything runs on background threads, and
//!   whatever is cached keeps working offline.
//! - Requests are plain GETs by the system's `curl` (as the update check
//!   does) with a User-Agent naming Operandi, as Poly Haven's API terms
//!   ask. Nothing about the user is sent; there are no accounts or keys.
use crate::prefs_file::PrefsFile;
use operandi_core::preset::online::{self, Asset, AssetKind, Catalogue, SourceId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};

/// Sent with every request (Poly Haven's terms ask for one that names the
/// software; never a generic default).
pub const USER_AGENT: &str = concat!(
    "Operandi/",
    env!("CARGO_PKG_VERSION"),
    " (+https://operandi3d.org)"
);
/// Downloads running at once; more are refused with a note.
const MAX_DOWNLOADS: usize = 3;
/// Thumbnail requests kept waiting; older ones are dropped (and asked for
/// again when their tiles come back into view).
const THUMB_QUEUE: usize = 120;

// ------------------------------------------------------------ preferences

/// How often a catalogue is fetched again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refresh {
    #[default]
    Weekly,
    Monthly,
    /// Only by Refresh Catalogue (or `operandi.online.refresh`).
    Manual,
}

impl Refresh {
    pub const ALL: [Self; 3] = [Self::Weekly, Self::Monthly, Self::Manual];
    pub fn label(self) -> &'static str {
        match self {
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Manual => "Only When Asked",
        }
    }
    /// Seconds a catalogue stays fresh.
    pub fn max_age(self) -> Option<u64> {
        match self {
            Self::Weekly => Some(7 * 86_400),
            Self::Monthly => Some(30 * 86_400),
            Self::Manual => None,
        }
    }
}

/// Preferences > Defaults > Application > Online Libraries, saved in
/// `online-library-preferences.json`; `pref.value online.<field>` sets each.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OnlinePreferences {
    version: u8,
    /// Poly Haven's HDRIs, textures and models. Off by default.
    pub poly_haven: bool,
    /// ambientCG's HDRIs and materials. Off by default.
    pub ambient_cg: bool,
    /// The resolution downloads ask for, in thousands of pixels (the
    /// nearest offered at or below it, else the lowest).
    pub resolution: u32,
    pub refresh: Refresh,
}

impl Default for OnlinePreferences {
    fn default() -> Self {
        Self {
            version: 1,
            poly_haven: false,
            ambient_cg: false,
            resolution: 2,
            refresh: Refresh::Weekly,
        }
    }
}

impl PrefsFile for OnlinePreferences {
    const INVALID: &'static str = "the download resolution is 1, 2, 4, 8 or 16 (K)";
    fn valid(&self) -> bool {
        online::RESOLUTIONS.contains(&self.resolution)
    }
}

impl OnlinePreferences {
    pub fn enabled(&self, source: SourceId) -> bool {
        match source {
            SourceId::PolyHaven => self.poly_haven,
            SourceId::AmbientCg => self.ambient_cg,
        }
    }
    pub fn turn_on(&mut self, source: SourceId, on: bool) {
        match source {
            SourceId::PolyHaven => self.poly_haven = on,
            SourceId::AmbientCg => self.ambient_cg = on,
        }
    }
    /// The sources that are on.
    pub fn sources(&self) -> Vec<SourceId> {
        SourceId::ALL
            .into_iter()
            .filter(|s| self.enabled(*s))
            .collect()
    }
}

// ------------------------------------------------------------ fetching

/// Fetches `url`: into the file given (answering no bytes), or into
/// memory. Replaced in tests, which never go online.
pub type Fetch = Arc<dyn Fn(&str, Option<&Path>) -> Result<Vec<u8>, String> + Send + Sync>;

/// The real fetch: the system's `curl`, HTTPS only.
pub fn curl_fetch(url: &str, to: Option<&Path>) -> Result<Vec<u8>, String> {
    if cfg!(test) {
        return Err("Tests don't go online".into());
    }
    curl(url, to)
}

/// `curl` itself (the opt-in live test calls it directly).
pub fn curl(url: &str, to: Option<&Path>) -> Result<Vec<u8>, String> {
    let mut command = std::process::Command::new("curl");
    command
        .args(["-fsSL", "--proto", "=https", "--proto-redir", "=https"])
        .args(["--connect-timeout", "20"])
        // A stalled download gives up after a minute below 1 KB/s.
        .args(["--speed-limit", "1024", "--speed-time", "60"])
        .args(["-A", USER_AGENT])
        .stdin(std::process::Stdio::null());
    if let Some(path) = to {
        command.arg("-o").arg(path);
    } else {
        command.args(["--max-time", "120"]);
    }
    command.arg(url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window flashing up for curl.exe.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let out = command.output().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => "curl isn't installed".to_owned(),
        _ => format!("curl couldn't run ({e})"),
    })?;
    if !out.status.success() {
        let why = String::from_utf8_lossy(&out.stderr);
        let why = why.trim().trim_start_matches("curl: ");
        let host = url.split('/').nth(2).unwrap_or(url);
        return Err(if why.is_empty() {
            format!("{host} could not be reached")
        } else {
            format!("{host} could not be reached ({why})")
        });
    }
    Ok(out.stdout)
}

/// Fetch `url` into `dest` by way of `dest.part`, so a broken download
/// never looks finished.
fn fetch_file(fetch: &Fetch, url: &str, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let part = PathBuf::from(format!("{}.part", dest.display()));
    let result = fetch(url, Some(&part)).and_then(|bytes| {
        // A fetch that answered in memory (a test) still lands on disk.
        if !bytes.is_empty() || !part.exists() {
            std::fs::write(&part, &bytes).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&part, dest).map_err(|e| e.to_string())
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// ------------------------------------------------------------ state

/// Which asset: source, kind and id.
pub type Key = (SourceId, AssetKind, String);

pub fn key_of(asset: &Asset) -> Key {
    (asset.source, asset.kind, asset.id.clone())
}

/// What to do once a download lands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Then {
    /// Nothing: it was only fetched (Download At, or `load:false`).
    Keep,
    /// Load it as the browser would have.
    Load(crate::preset_browser::Target),
}

/// A download in progress (or just failed).
#[derive(Clone, Debug)]
pub struct Download {
    pub name: String,
    /// Known once the source's file list has been read.
    pub k: Option<u32>,
    pub bytes: u64,
    pub dir: Option<PathBuf>,
    pub then: Then,
    pub asset: Asset,
}

impl Download {
    /// 0..1 when the size is known.
    pub fn progress(&self) -> Option<f32> {
        let dir = self.dir.as_ref()?;
        (self.bytes > 0)
            .then(|| (online::folder_size(dir) as f64 / self.bytes as f64).min(1.) as f32)
    }
}

/// What the background threads report.
enum Event {
    Catalogue {
        source: SourceId,
        result: Result<usize, String>,
    },
    Thumb(PathBuf),
    Planned {
        key: Key,
        k: u32,
        bytes: u64,
        dir: PathBuf,
    },
    Downloaded {
        key: Key,
        result: Result<PathBuf, String>,
    },
}

/// What a tick hands the app.
#[derive(Debug)]
pub enum Outcome {
    /// A thumbnail arrived: forget the failed slot so it's read again.
    Thumb(PathBuf),
    /// A source's catalogue changed (its folders are read again).
    Catalogue {
        message: String,
    },
    /// A download landed; load it if asked.
    Downloaded {
        asset: Asset,
        main: PathBuf,
        then: Then,
    },
    Failed(String),
}

/// Thumbnails waiting, newest last; and every file asked for so far.
#[derive(Default)]
struct ThumbQueue {
    jobs: Vec<(String, PathBuf)>,
    pending: HashSet<PathBuf>,
    failed: HashSet<PathBuf>,
}

pub struct OnlineLibrary {
    pub prefs: OnlinePreferences,
    /// Where every source's root is (`<data>/Online Libraries`).
    pub folder: PathBuf,
    fetch: Fetch,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    /// Catalogues being fetched.
    pub refreshing: BTreeSet<SourceId>,
    /// Sources whose catalogue age was looked at this session (a failed
    /// automatic refresh isn't tried again until Refresh Catalogue).
    checked: BTreeSet<SourceId>,
    /// The last refresh's failure per source, shown in the browser.
    pub errors: HashMap<SourceId, String>,
    thumbs: Arc<(Mutex<ThumbQueue>, Condvar)>,
    thumb_worker: bool,
    pub downloads: HashMap<Key, Download>,
    /// Catalogues as read from the cache, by source (for the bridge's
    /// list and the browser's "updated" line).
    catalogues: HashMap<SourceId, Arc<Catalogue>>,
    /// The resolutions downloaded per asset, as last read from disk (tiles
    /// ask every frame).
    have: std::cell::RefCell<HashMap<Key, Vec<u32>>>,
    /// The cache's size and when it was measured (Preferences).
    size: Option<(std::time::Instant, u64)>,
}

impl Default for OnlineLibrary {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("operandi-online-libraries"))
    }
}

impl OnlineLibrary {
    pub fn new(folder: PathBuf) -> Self {
        let (tx, rx) = channel();
        Self {
            prefs: OnlinePreferences::default(),
            folder,
            fetch: Arc::new(curl_fetch),
            tx,
            rx,
            refreshing: BTreeSet::new(),
            checked: BTreeSet::new(),
            errors: HashMap::new(),
            thumbs: Default::default(),
            thumb_worker: false,
            downloads: HashMap::new(),
            catalogues: HashMap::new(),
            have: Default::default(),
            size: None,
        }
    }

    /// Replace the network (tests).
    #[cfg(test)]
    pub fn set_fetch(&mut self, fetch: Fetch) {
        self.fetch = fetch;
    }

    pub fn root(&self, source: SourceId) -> PathBuf {
        online::source_root(&self.folder, source)
    }

    /// The roots the browser shows: the sources that are on.
    pub fn roots(&self) -> Vec<(PathBuf, SourceId)> {
        self.prefs
            .sources()
            .into_iter()
            .map(|s| (self.root(s), s))
            .collect()
    }

    /// The cached catalogue of `source` (read once, then kept).
    pub fn catalogue(&mut self, source: SourceId) -> Option<Arc<Catalogue>> {
        if let Some(c) = self.catalogues.get(&source) {
            return Some(c.clone());
        }
        let c = Arc::new(Catalogue::read(&self.root(source))?);
        self.catalogues.insert(source, c.clone());
        Some(c)
    }

    /// For each source that's on and hasn't been looked at this session:
    /// make its root, and fetch its catalogue when there's none or it's
    /// old. Never contacts a source that's off.
    pub fn keep_fresh(&mut self) {
        for source in self.prefs.sources() {
            if !self.checked.insert(source) {
                continue;
            }
            let _ = std::fs::create_dir_all(self.root(source));
            let due = match self.catalogue(source) {
                None => true,
                Some(c) => c.stale(now_secs(), self.prefs.refresh.max_age()),
            };
            if due {
                self.refresh(source);
            }
        }
    }

    /// Fetch `source`'s catalogue again in the background. Returns false
    /// when it's off or already being fetched.
    pub fn refresh(&mut self, source: SourceId) -> bool {
        if !self.prefs.enabled(source) || !self.refreshing.insert(source) {
            return false;
        }
        self.checked.insert(source);
        self.errors.remove(&source);
        let (fetch, tx, root) = (self.fetch.clone(), self.tx.clone(), self.root(source));
        let spawned = std::thread::Builder::new()
            .name("online-catalogue".into())
            .spawn(move || {
                let result = fetch_catalogue(&fetch, source).and_then(|assets| {
                    let n = assets.len();
                    let catalogue = Catalogue::new(source, now_secs(), assets);
                    online::sync_stubs(&root, &catalogue.assets)
                        .and_then(|_| catalogue.write(&root))
                        .map_err(|e| format!("Couldn't write the cache: {e}"))?;
                    Ok(n)
                });
                let _ = tx.send(Event::Catalogue { source, result });
            });
        if spawned.is_err() {
            self.refreshing.remove(&source);
            return false;
        }
        true
    }

    /// Ask for `asset`'s thumbnail when it isn't cached (a tile on
    /// screen). Returns its cache file.
    pub fn want_thumbnail(&mut self, root: &Path, asset: &Asset) -> PathBuf {
        let path = online::thumb_path(root, asset);
        if !self.prefs.enabled(asset.source) || asset.thumbnail.is_empty() || path.is_file() {
            return path;
        }
        let (lock, wake) = &*self.thumbs;
        let mut q = lock.lock().expect("thumbnail queue");
        if q.pending.contains(&path) || q.failed.contains(&path) {
            return path;
        }
        q.pending.insert(path.clone());
        q.jobs.push((asset.thumbnail.clone(), path.clone()));
        if q.jobs.len() > THUMB_QUEUE {
            let (_, dropped) = q.jobs.remove(0);
            q.pending.remove(&dropped);
        }
        drop(q);
        wake.notify_one();
        if !self.thumb_worker {
            self.thumb_worker = true;
            let (queue, fetch, tx) = (self.thumbs.clone(), self.fetch.clone(), self.tx.clone());
            let _ = std::thread::Builder::new()
                .name("online-thumbnails".into())
                .spawn(move || thumb_worker(queue, fetch, tx));
        }
        path
    }

    /// Start downloading `asset` at about `want_k`. Already downloaded at
    /// that choice: nothing to fetch (`Ok(false)`).
    pub fn download(&mut self, asset: &Asset, want_k: u32, then: Then) -> Result<bool, String> {
        if !self.prefs.enabled(asset.source) {
            return Err(format!(
                "{} is off; turn it on in Preferences > Defaults > Application > Online Libraries",
                asset.source.label()
            ));
        }
        let key = key_of(asset);
        if let Some(d) = self.downloads.get_mut(&key) {
            // Asked again while it runs: the newest request wins.
            d.then = then;
            return Ok(true);
        }
        if self.downloads.len() >= MAX_DOWNLOADS {
            return Err(format!(
                "{MAX_DOWNLOADS} downloads are running; try {} again when one is done",
                asset.name
            ));
        }
        let root = self.root(asset.source);
        let (fetch, tx, job) = (self.fetch.clone(), self.tx.clone(), asset.clone());
        let k = key.clone();
        std::thread::Builder::new()
            .name("online-download".into())
            .spawn(move || {
                let result = download(&fetch, &root, &job, want_k, &|k2, bytes, dir| {
                    let _ = tx.send(Event::Planned {
                        key: k.clone(),
                        k: k2,
                        bytes,
                        dir: dir.to_path_buf(),
                    });
                });
                let _ = tx.send(Event::Downloaded { key: k, result });
            })
            .map_err(|e| e.to_string())?;
        self.downloads.insert(
            key,
            Download {
                name: asset.name.clone(),
                k: None,
                bytes: 0,
                dir: None,
                then,
                asset: asset.clone(),
            },
        );
        Ok(true)
    }

    /// The resolutions of `asset` downloaded so far, lowest first.
    pub fn have(&self, asset: &Asset) -> Vec<u32> {
        let root = self.root(asset.source);
        self.have
            .borrow_mut()
            .entry(key_of(asset))
            .or_insert_with(|| {
                online::downloaded(&root, asset)
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect()
            })
            .clone()
    }

    /// The resolutions `asset` can be had at: what its source said when
    /// it was last downloaded, else what its catalogue entry suggests.
    pub fn resolutions(&self, asset: &Asset) -> Vec<u32> {
        online::offered(&self.root(asset.source), asset).unwrap_or_else(|| asset.resolutions())
    }

    /// Remove `asset`'s downloaded files.
    pub fn remove_download(&mut self, asset: &Asset) -> Result<(), String> {
        if self.downloads.contains_key(&key_of(asset)) {
            return Err(format!("{} is downloading", asset.name));
        }
        let dir = online::downloads_dir(&self.root(asset.source), asset);
        self.have.borrow_mut().remove(&key_of(asset));
        self.size = None;
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("Couldn't remove {}: {e}", dir.display())),
        }
    }

    /// Bytes in the cache, measured at most every ten seconds.
    pub fn cache_size(&mut self) -> u64 {
        match self.size {
            Some((at, bytes)) if at.elapsed().as_secs() < 10 => bytes,
            _ => {
                let bytes = online::folder_size(&self.folder);
                self.size = Some((std::time::Instant::now(), bytes));
                bytes
            }
        }
    }

    /// Something is running (the app keeps repainting).
    pub fn busy(&self) -> bool {
        !self.refreshing.is_empty() || !self.downloads.is_empty() || self.thumbs_waiting()
    }

    fn thumbs_waiting(&self) -> bool {
        self.thumb_worker
            && !self
                .thumbs
                .0
                .lock()
                .expect("thumbnail queue")
                .pending
                .is_empty()
    }

    /// Take what the threads finished.
    pub fn tick(&mut self) -> Vec<Outcome> {
        let mut out = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Thumb(path) => out.push(Outcome::Thumb(path)),
                Event::Catalogue { source, result } => {
                    self.refreshing.remove(&source);
                    self.catalogues.remove(&source);
                    let message = match result {
                        Ok(n) => format!("{}: {n} assets in the catalogue", source.label()),
                        Err(e) => {
                            let m = format!("Couldn't refresh {}'s catalogue: {e}", source.label());
                            self.errors.insert(source, m.clone());
                            m
                        }
                    };
                    out.push(Outcome::Catalogue { message });
                }
                Event::Planned { key, k, bytes, dir } => {
                    if let Some(d) = self.downloads.get_mut(&key) {
                        d.k = Some(k);
                        d.bytes = bytes;
                        d.dir = Some(dir);
                    }
                }
                Event::Downloaded { key, result } => {
                    let Some(d) = self.downloads.remove(&key) else {
                        continue;
                    };
                    self.have.borrow_mut().remove(&key);
                    self.size = None;
                    out.push(match result {
                        Ok(main) => Outcome::Downloaded {
                            asset: d.asset,
                            main,
                            then: d.then,
                        },
                        Err(e) => Outcome::Failed(format!("Couldn't download {}: {e}", d.name)),
                    });
                }
            }
        }
        out
    }

    /// Remove downloaded files and thumbnails (one source or all). A
    /// source that's off loses its whole folder, catalogue included.
    pub fn clear_cache(&mut self, only: Option<SourceId>) -> Result<String, String> {
        let mut freed = 0;
        for source in SourceId::ALL {
            if only.is_some_and(|o| o != source) {
                continue;
            }
            if self.downloads.values().any(|d| d.asset.source == source) {
                return Err(format!(
                    "{} has downloads running; clear it when they're done",
                    source.label()
                ));
            }
            let root = self.root(source);
            let parts: Vec<PathBuf> = if self.prefs.enabled(source) {
                vec![
                    root.join(online::DOWNLOADS_DIR),
                    root.join(online::THUMBS_DIR),
                ]
            } else {
                vec![root.clone()]
            };
            for part in parts {
                if part.is_dir() {
                    freed += online::folder_size(&part);
                    std::fs::remove_dir_all(&part)
                        .map_err(|e| format!("Couldn't clear {}: {e}", part.display()))?;
                }
            }
            if !self.prefs.enabled(source) {
                self.catalogues.remove(&source);
                self.checked.remove(&source);
            }
        }
        self.have.borrow_mut().clear();
        self.size = None;
        let (lock, _) = &*self.thumbs;
        let mut q = lock.lock().expect("thumbnail queue");
        q.failed.clear();
        Ok(format!(
            "Cleared {} of online library downloads and thumbnails",
            size_label(freed)
        ))
    }

    /// Every asset of the sources that are on, from their caches, that
    /// matches the filters (the bridge's list).
    pub fn find(
        &mut self,
        source: Option<SourceId>,
        kind: Option<AssetKind>,
        search: &str,
    ) -> Vec<Asset> {
        let text = search.trim().to_lowercase();
        let mut out = Vec::new();
        for s in self.prefs.sources() {
            if source.is_some_and(|o| o != s) {
                continue;
            }
            let Some(c) = self.catalogue(s) else { continue };
            out.extend(
                c.assets
                    .iter()
                    .filter(|a| kind.is_none_or(|k| a.kind == k))
                    .filter(|a| text.is_empty() || a.matches(&text))
                    .cloned(),
            );
        }
        out
    }
}

/// Every kind's catalogue pages for `source`.
fn fetch_catalogue(fetch: &Fetch, source: SourceId) -> Result<Vec<Asset>, String> {
    let text = |url: &str| -> Result<String, String> {
        String::from_utf8(fetch(url, None)?).map_err(|_| "the answer wasn't text".to_string())
    };
    let mut assets = Vec::new();
    for &kind in source.kinds() {
        match source {
            SourceId::PolyHaven => {
                assets.extend(online::parse_polyhaven(
                    kind,
                    &text(&source.catalogue_url(kind))?,
                )?);
            }
            SourceId::AmbientCg => {
                let mut url = Some(source.catalogue_url(kind));
                // A runaway "next page" chain stops after 50 pages.
                for _ in 0..50 {
                    let Some(u) = url.take() else { break };
                    let (page, next) = online::parse_ambientcg(kind, &text(&u)?)?;
                    let empty = page.is_empty();
                    assets.extend(page);
                    if !empty {
                        url = next;
                    }
                }
            }
        }
    }
    Ok(assets)
}

/// Download `asset` at about `want_k` into its cache folder; answers the
/// file to load. `planned` hears the resolution, size and folder first.
fn download(
    fetch: &Fetch,
    root: &Path,
    asset: &Asset,
    want_k: u32,
    planned: &dyn Fn(u32, u64, &Path),
) -> Result<PathBuf, String> {
    let plan = match asset.source {
        SourceId::PolyHaven => {
            let files = fetch(&online::polyhaven_files_url(asset), None)?;
            let files = String::from_utf8(files).map_err(|_| "the file list wasn't text")?;
            online::polyhaven_plan(asset, &files, want_k)?
        }
        SourceId::AmbientCg => online::ambientcg_plan(asset, want_k)?,
    };
    if let Some((_, main)) = online::downloaded(root, asset)
        .into_iter()
        .find(|(k, _)| *k == plan.k)
    {
        return Ok(main);
    }
    let _ = online::write_offered(root, asset, &plan.offered);
    let dir = online::download_dir(root, asset, plan.k);
    planned(plan.k, plan.bytes(), &dir);
    let result = (|| {
        // A half-finished earlier try starts again.
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for f in &plan.files {
            fetch_file(fetch, &f.url, &dir.join(&f.rel))?;
        }
        let main = if plan.archive {
            let zip = dir.join(&plan.files[0].rel);
            let bytes = std::fs::read(&zip).map_err(|e| e.to_string())?;
            let main = online::unpack(&bytes, &dir, asset)?;
            let _ = std::fs::remove_file(&zip);
            main
        } else {
            dir.join(plan.main.as_ref().ok_or("nothing to load")?)
        };
        online::finish_download(&dir, &main).map_err(|e| e.to_string())?;
        Ok(main)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    result
}

/// The thumbnail thread: newest request first, one at a time.
fn thumb_worker(queue: Arc<(Mutex<ThumbQueue>, Condvar)>, fetch: Fetch, tx: Sender<Event>) {
    let (lock, wake) = &*queue;
    loop {
        let (url, path) = {
            let mut q = lock.lock().expect("thumbnail queue");
            loop {
                if let Some(job) = q.jobs.pop() {
                    break job;
                }
                q = wake.wait(q).expect("thumbnail queue");
            }
        };
        let ok = fetch_file(&fetch, &url, &path).is_ok();
        let mut q = lock.lock().expect("thumbnail queue");
        q.pending.remove(&path);
        if ok {
            drop(q);
            if tx.send(Event::Thumb(path)).is_err() {
                return;
            }
        } else {
            q.failed.insert(path);
        }
    }
}

/// "1.2 MB" for `bytes`.
pub fn size_label(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 30 => format!("{:.1} GB", b as f64 / (1u64 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1u64 << 20) as f64),
        b if b >= 1 << 10 => format!("{} KB", b >> 10),
        b => format!("{b} B"),
    }
}
