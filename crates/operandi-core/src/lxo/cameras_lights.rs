//! LXO import: cameras and lights (spec lxo-import-anim-rig-2026-11-24,
//! LXA-50..54). They read their transforms, parents and visibility like
//! any item (phase1.rs); this module turns their own channels into
//! Operandi's camera and light settings. A light's colour and material
//! settings live on a child `lightMaterial` item and are folded into the
//! light (LXA-51). A missing channel keeps Operandi's default, which is
//! Modo's (LXA-53); channels starting `oc_` belong to a renderer plug-in
//! and are ignored.
use super::phase1::{RawItem, number, text};
use crate::camera_light::{
    AreaShape, CameraSettings, FilmFit, LightBlend, LightFalloff, LightKind, LightRender,
    LightSettings, Projection, ShadowType, Volumetrics,
};
use std::collections::BTreeMap;

/// LXA-50: a Modo light item type as Operandi's light kind.
pub fn light_kind(kind: &str) -> Option<LightKind> {
    Some(match kind {
        "sunLight" => LightKind::Directional,
        "pointLight" => LightKind::Point,
        "spotLight" => LightKind::Spot,
        "areaLight" => LightKind::Area,
        "cylinderLight" => LightKind::Cylinder,
        "domeLight" => LightKind::Dome,
        "photometryLight" => LightKind::Photometric,
        "portal" => LightKind::Portal,
        "meshLight" => LightKind::Mesh,
        _ => return None,
    })
}

/// Item types read here though they aren't scene items themselves.
pub fn handled(kind: &str) -> bool {
    kind == "lightMaterial"
}

fn flag(item: &RawItem, key: &str) -> Option<bool> {
    number(item, key).map(|v| v != 0.)
}

/// LXA-52: a camera's settings.
pub fn camera(item: &RawItem) -> CameraSettings {
    let mut c = CameraSettings::default();
    if let Some(p) = text(item, "projType") {
        c.projection = match p {
            "ortho" => Projection::Orthographic,
            "cylindrical" => Projection::Cylindrical,
            "cylindricalvr" => Projection::CylindricalVr,
            "spherical" => Projection::Spherical,
            "sphericalvr" => Projection::SphericalVr,
            _ => Projection::Perspective,
        };
    }
    let set = |v: &mut f64, key: &str| {
        if let Some(n) = number(item, key) {
            *v = n;
        }
    };
    set(&mut c.focal_length, "focalLen");
    set(&mut c.film_width, "apertureX");
    set(&mut c.film_height, "apertureY");
    set(&mut c.film_offset[0], "offsetX");
    set(&mut c.film_offset[1], "offsetY");
    set(&mut c.target_distance, "target");
    set(&mut c.focus_distance, "focusDist");
    set(&mut c.clip_near, "clipDist");
    set(&mut c.stored.lens_squeeze, "squeeze");
    set(&mut c.stored.film_roll, "filmRoll");
    set(&mut c.stored.overscan, "overscan");
    set(&mut c.lens_distortion, "distort");
    set(&mut c.dof.f_stop, "fStop");
    set(&mut c.dof.iris_rotation, "irisRot");
    set(&mut c.dof.edge_weighting, "irisBias");
    if let Some(f) = text(item, "filmFit") {
        c.film_fit = match f {
            "fill" => FilmFit::Fill,
            "horizontal" => FilmFit::Horizontal,
            "vertical" => FilmFit::Vertical,
            "overscan" => FilmFit::Overscan,
            _ => c.film_fit,
        };
    }
    if let Some(b) = flag(item, "clipping") {
        c.clipping = b;
    }
    if let Some(b) = flag(item, "dof") {
        c.dof.enable = b;
    }
    if let Some(n) = number(item, "irisBlades") {
        c.dof.iris_blades = n.max(0.) as u32;
    }
    if let Some(b) = flag(item, "resOverride") {
        c.resolution.enable = b;
    }
    if let Some(n) = number(item, "resX").filter(|n| *n >= 1.) {
        c.resolution.width = n as u32;
    }
    if let Some(n) = number(item, "resY").filter(|n| *n >= 1.) {
        c.resolution.height = n as u32;
    }
    if c.validate().is_err() {
        // A value outside Operandi's ranges: the camera keeps its
        // defaults rather than failing the import.
        return CameraSettings::default();
    }
    c
}

/// LXA-53: a light's settings, with its light material folded in
/// (LXA-51). `None` for an item that isn't a light.
pub fn light(item: &RawItem, items: &BTreeMap<u32, RawItem>) -> Option<LightSettings> {
    let kind = light_kind(&item.kind)?;
    let mut l = LightSettings::new(kind);
    let set = |v: &mut f64, from: &RawItem, key: &str| {
        if let Some(n) = number(from, key) {
            *v = n;
        }
    };
    set(&mut l.intensity, item, "radiance");
    set(&mut l.dissolve, item, "dissolve");
    set(&mut l.stored.importance, item, "importance");
    set(&mut l.spread_angle, item, "spread");
    set(&mut l.stored.shadow_map_size, item, "mapSize");
    set(&mut l.radius, item, "radius");
    set(&mut l.cone_angle, item, "cone");
    set(&mut l.soft_edge, item, "edge");
    set(&mut l.width, item, "width");
    set(&mut l.height, item, "height");
    set(&mut l.length, item, "length");
    if let Some(f) = text(item, "fallType") {
        l.falloff = match f {
            "none" => LightFalloff::None,
            "invdist" => LightFalloff::InverseDistance,
            "invdist2" => LightFalloff::InverseDistanceSquared,
            _ => l.falloff,
        };
    }
    if let Some(s) = text(item, "shadType") {
        l.shadow = match s {
            "none" => ShadowType::None,
            _ => ShadowType::RayTraced,
        };
    }
    if let Some(r) = text(item, "render") {
        l.render = match r {
            "on" => LightRender::On,
            "off" => LightRender::Off,
            _ => LightRender::Default,
        };
    }
    if let Some(s) = text(item, "shape") {
        l.shape = match s {
            "ellipse" => AreaShape::Elliptical,
            _ => AreaShape::Rectangular,
        };
    }
    if let Some(b) = flag(item, "visCam") {
        l.visible_to_camera = b;
    }
    if let Some(b) = flag(item, "visRefl") {
        l.stored.visible_to_reflection = b;
    }
    if let Some(b) = flag(item, "visRefr") {
        l.stored.visible_to_refraction = b;
    }
    if let Some(n) = number(item, "samples").filter(|n| *n >= 1.) {
        l.stored.samples = n as u32;
    }
    if let Some(b) = flag(item, "envOnly") {
        l.environment_only = b;
    }
    if let Some(t) = text(item, "lpeLabel") {
        l.lpe_label = t.to_owned();
    }
    if let Some(t) = text(item, "filename") {
        l.photometry_file = t.to_owned();
    }
    if flag(item, "volumetrics") == Some(true) {
        let mut v = Volumetrics::new(kind);
        v.enabled = true;
        if let Some(n) = number(item, "vsamples").filter(|n| *n >= 1.) {
            v.samples = n as u32;
        }
        l.volumetrics = Some(v);
    }
    // LXA-51: the light material child.
    if let Some(m) = items
        .values()
        .find(|m| m.kind == "lightMaterial" && m.link("parent") == Some(item.id))
    {
        for (i, axis) in ["R", "G", "B"].iter().enumerate() {
            set(&mut l.color[i], m, &format!("lightCol.{axis}"));
            set(&mut l.stored.shadow_color[i], m, &format!("shadCol.{axis}"));
            if let Some(v) = l.volumetrics.as_mut() {
                set(&mut v.color[i], m, &format!("scatteringColor.{axis}"));
            }
        }
        set(&mut l.affect_diffuse, m, "diffuse");
        set(&mut l.affect_specular, m, "specular");
        set(&mut l.stored.affect_caustics, m, "caustics");
        set(&mut l.stored.affect_subsurface, m, "subsurface");
        set(&mut l.stored.opacity, m, "opacity");
        if let Some(b) = flag(m, "enable") {
            l.stored.material_enabled = b;
        }
        if let Some(v) = l.volumetrics.as_mut() {
            set(&mut v.density, m, "density");
            set(&mut v.attenuation, m, "attenuate");
            set(&mut v.shift, m, "shift");
        }
        if let Some(b) = text(m, "blend") {
            l.blend_mode = match b {
                "add" => LightBlend::Add,
                "subtract" => LightBlend::Subtract,
                "multiply" | "normalmult" => LightBlend::Multiply,
                "screen" => LightBlend::Screen,
                "overlay" => LightBlend::Overlay,
                _ => LightBlend::Normal,
            };
        }
    }
    if l.validate().is_err() {
        return Some(LightSettings::new(kind));
    }
    Some(l)
}

/// LXA-32 / LXA-52 / LXA-53: the Operandi channel a keyed camera, light or
/// light material channel maps to (`None`: no equivalent).
pub fn channel(kind: &str, modo: &str) -> Option<String> {
    if kind == "camera" {
        return Some(
            match modo {
                "focalLen" => "focalLength",
                "focusDist" => "focusDistance",
                "target" => "targetDistance",
                "fStop" => "fStop",
                _ => return None,
            }
            .into(),
        );
    }
    if kind == "lightMaterial" {
        if let Some(axis) = modo.strip_prefix("lightCol.") {
            return Some(format!("color.{}", axis.to_ascii_lowercase()));
        }
        return Some(
            match modo {
                "diffuse" => "diffuse",
                "specular" => "specular",
                "opacity" => "opacity",
                _ => return None,
            }
            .into(),
        );
    }
    light_kind(kind)?;
    Some(
        match modo {
            "radiance" => "intensity",
            "radius" => "radius",
            "cone" => "coneAngle",
            "edge" => "softEdge",
            "spread" => "spread",
            "width" => "width",
            "height" => "height",
            "length" => "length",
            "dissolve" => "dissolve",
            _ => return None,
        }
        .into(),
    )
}
