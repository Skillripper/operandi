//! The component selection modes (vertex, edge, polygon, item) and the
//! names of selection sets stored in a mesh's attributes. The mesh, the
//! scene and every tool read them, so they live in the base layer
//! (docs/AREA_DEPENDENCIES.md).
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Vertex,
    Edge,
    #[default]
    Polygon,
    Item,
}

/// Which components a selection set holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SetKind {
    Vertex,
    Edge,
    Polygon,
}
impl SetKind {
    /// The set kind of a component selection mode; none in Item mode.
    pub fn of_mode(mode: Mode) -> Option<Self> {
        match mode {
            Mode::Vertex => Some(Self::Vertex),
            Mode::Edge => Some(Self::Edge),
            Mode::Polygon => Some(Self::Polygon),
            Mode::Item => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Edge => "edge",
            Self::Polygon => "polygon",
        }
    }
}

/// One member of a selection set group: a set named by kind and name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetRef {
    pub kind: SetKind,
    pub name: String,
}
