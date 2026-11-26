//! One load/save for the small JSON preference files: size cap, parse,
//! validity check, then a temp-file write so a failed save leaves the old
//! file alone. Each preference type says only what "valid" means.
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

pub(crate) trait PrefsFile: Serialize + DeserializeOwned + Default {
    /// Files larger than this are ignored on load.
    const MAX_BYTES: u64 = 64 * 1024;
    /// The error `save` returns for an invalid value.
    const INVALID: &'static str;
    fn valid(&self) -> bool;
    /// A hook to bring an older valid file up to date after loading.
    fn upgrade(self) -> Self {
        self
    }
    /// The preferences in `path`, or None if the file is missing or invalid.
    fn read(path: &Path) -> Option<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)
            .ok()?
            .take(Self::MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        if bytes.len() as u64 > Self::MAX_BYTES {
            crate::log_ring::log_line!(
                "Warning: ignored preferences file {} (larger than {} KiB)",
                path.display(),
                Self::MAX_BYTES / 1024
            );
            return None;
        }
        serde_json::from_slice::<Self>(&bytes)
            .ok()
            .filter(Self::valid)
            .map(Self::upgrade)
    }
    /// The preferences in `path`, or the defaults.
    fn load(path: &Path) -> Self {
        Self::read(path).unwrap_or_default()
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        if !self.valid() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, Self::INVALID));
        }
        write_json(self, path)
    }
}

/// Write `value` as JSON to `path` through a temp file next to it.
pub(crate) fn write_json(value: &impl Serialize, path: &Path) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    let temp = path.with_extension(format!("json.tmp-{}", std::process::id()));
    crate::resource_write::write_bytes(path, &temp, &bytes)
}

#[cfg(test)]
#[test]
fn defaults_preferences_accept_larger_files_and_log_oversize() {
    use crate::defaults_preferences::DefaultsPreferences;
    let a = crate::Operandi::empty();
    let path = a.environment.paths().defaults_preferences();
    let mut prefs = DefaultsPreferences::default();
    prefs.default_scene = "Original.opscene".into();
    let mut json = serde_json::to_vec(&prefs).unwrap();
    json.resize(64 * 1024, b' ');
    fs::write(&path, &json).unwrap();
    assert_eq!(
        DefaultsPreferences::load(&path).default_scene,
        "Original.opscene"
    );
    json.push(b' ');
    fs::write(&path, &json).unwrap();
    assert_eq!(
        DefaultsPreferences::load(&path).default_scene,
        DefaultsPreferences::default().default_scene
    );
    assert!(
        crate::log_ring::tail(200)
            .iter()
            .any(|line| line.contains("ignored preferences file")
                && line.contains(&path.display().to_string()))
    );
}
