//! The MeshBlend recipe (PLAN.md rule 1): everything a MeshBlend Item is
//! made from, saved as the MeshBlend op's values. It already holds the
//! whole model (sources, roles, relationships, per-source settings, strip
//! settings and room for per-strip overrides), even where B1's panel shows
//! only part of it, so files made now open unchanged later (ST-02).
//!
//! The result is never saved as truth; it is rebuilt from this.
use serde::{Deserialize, Serialize};

/// A source's role (SB-10). Saved as its number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "u32", try_from = "u32")]
pub enum Role {
    /// Adds volume; Primaries are unioned.
    #[default]
    Primary,
    /// Removes its volume from the Primaries it is related to.
    Subtract,
    /// Keeps only the part of the related Primaries inside it.
    Intersect,
}
impl Role {
    pub const ALL: [Role; 3] = [Role::Primary, Role::Subtract, Role::Intersect];
    /// The name the panel shows (SB-10).
    pub fn label(self) -> &'static str {
        match self {
            Role::Primary => "Primary",
            Role::Subtract => "Subtractive Trim",
            Role::Intersect => "Intersect Trim",
        }
    }
    /// The word scripts use.
    pub fn word(self) -> &'static str {
        match self {
            Role::Primary => "primary",
            Role::Subtract => "subtract",
            Role::Intersect => "intersect",
        }
    }
    /// A script's word or number.
    pub fn parse(text: &str) -> crate::Result<Self> {
        let t = text.trim().to_ascii_lowercase();
        if let Ok(n) = t.parse::<u32>() {
            return Self::try_from(n);
        }
        Self::ALL
            .into_iter()
            .find(|r| r.word() == t || r.label().to_ascii_lowercase() == t)
            .ok_or_else(|| format!("role must be primary, subtract or intersect, not “{text}”"))
    }
    pub fn is_trim(self) -> bool {
        self != Role::Primary
    }
}
impl From<Role> for u32 {
    fn from(r: Role) -> u32 {
        r as u32
    }
}
impl TryFrom<u32> for Role {
    type Error = String;
    fn try_from(n: u32) -> crate::Result<Self> {
        Self::ALL.get(n as usize).copied().ok_or_else(|| {
            format!("role must be 0 (Primary), 1 (Subtractive Trim) or 2 (Intersect Trim), not {n}")
        })
    }
}

/// One source mesh of a MeshBlend Item.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlendSource {
    /// The source mesh item.
    pub item: u64,
    pub role: Role,
    /// For a Trim: the Primaries it acts on (SB-11), by item id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trims: Vec<u64>,
    /// Off leaves the source out of the result (SB-16: hiding doesn't).
    #[serde(default = "yes")]
    pub enabled: bool,
    /// How finely the smooth source becomes polygons, 0–3 (SB-20); 0 =
    /// the cage itself.
    #[serde(default = "two")]
    pub subdivision: u8,
    /// Use the cage's points instead of the smooth surface (SB-22).
    #[serde(default, rename = "keepCage")]
    pub keep_cage: bool,
}
fn twenty_percent() -> f64 {
    0.2
}
fn yes() -> bool {
    true
}
fn two() -> u8 {
    2
}
impl BlendSource {
    pub fn new(item: u64, role: Role) -> Self {
        Self {
            item,
            role,
            trims: Vec::new(),
            enabled: true,
            subdivision: 2,
            keep_cage: false,
        }
    }
}

/// Output modes (SB-50), the `output` value's names. New MeshBlends use
/// Closed, clean (provisional MB-Q07).
pub const OUTPUTS: [&str; 6] = [
    "Draft",
    "Closed",
    "Closed, clean",
    "Closed with parts per patch",
    "Closed with parts per source",
    "Strips only",
];

/// The MeshBlend op's values. Serde names are the saved channel names.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub sources: Vec<BlendSource>,
    /// 0 absolute distance, 1 relative to the sources' quad size (SB-31b).
    #[serde(rename = "widthUnits")]
    pub width_units: u32,
    /// Strip Width (Q05: crease to each band edge, on the surface).
    #[serde(rename = "stripWidth")]
    pub strip_width: f64,
    /// Strip Width with relative units: a fraction (0.2 = 20 %) of the
    /// sources' average cage edge length (SB-31b, MB-Q08).
    #[serde(rename = "stripWidthRel", default = "twenty_percent")]
    pub strip_width_rel: f64,
    /// Strip Profile as a fraction (1.0 = 100 %, round).
    #[serde(rename = "stripProfile")]
    pub strip_profile: f64,
    /// Strip Rows (2 × rows quads across, SB-33).
    #[serde(rename = "stripRows")]
    pub strip_rows: u32,
    /// Index into [`OUTPUTS`].
    pub output: u32,
    /// Narrow sharp strips for speed and diagnosis (ST-16).
    #[serde(rename = "designMode")]
    pub design_mode: bool,
    /// Per-strip overrides by stable strip id (ST-07); filled in B3.
    #[serde(default)]
    pub strips: Vec<serde_json::Value>,
}
impl Default for Recipe {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            width_units: 0,
            strip_width: 0.02,
            strip_width_rel: 0.2,
            strip_profile: 1.,
            strip_rows: 3,
            output: 2,
            design_mode: false,
            strips: Vec::new(),
        }
    }
}

impl Recipe {
    /// SAFE-10: values a file or script could have set out of range.
    pub fn check(&self) -> crate::Result<()> {
        let mut seen = std::collections::HashSet::new();
        for s in &self.sources {
            if s.item == 0 {
                return Err("A MeshBlend source names no item".into());
            }
            if !seen.insert(s.item) {
                return Err("A mesh is listed twice as a MeshBlend source".into());
            }
            if s.subdivision > 3 {
                return Err("MeshBlend Subdivision must be 0 to 3".into());
            }
        }
        if !self.strip_width.is_finite() || self.strip_width < 0. {
            return Err("MeshBlend Strip Width must be zero or more".into());
        }
        if !self.strip_width_rel.is_finite() || self.strip_width_rel < 0. {
            return Err("MeshBlend relative Strip Width must be zero or more".into());
        }
        if !self.strip_profile.is_finite() || !(0. ..=2.).contains(&self.strip_profile) {
            return Err("MeshBlend Strip Profile must be 0% to 200%".into());
        }
        if !(1..=16).contains(&self.strip_rows) {
            return Err("MeshBlend Strip Rows must be 1 to 16".into());
        }
        if self.width_units > 1 || self.output as usize >= OUTPUTS.len() {
            return Err("MeshBlend has a setting out of range".into());
        }
        Ok(())
    }

    pub fn source(&self, item: u64) -> Option<&BlendSource> {
        self.sources.iter().find(|s| s.item == item)
    }
    pub fn source_mut(&mut self, item: u64) -> Option<&mut BlendSource> {
        self.sources.iter_mut().find(|s| s.item == item)
    }
}
