//! The MeshBlend op's Properties (MeshBlend B1, PLAN.md): the source list
//! with each source's role, the Primaries a Trim acts on, its per-source
//! settings and any problem, plus Add and Convert. Every change runs a
//! `meshblend.*` command (one undo step each, SB-80); the panel holds no
//! state of its own.
//!
//! Drawn with the shared row component (UIC-125): one section per source,
//! labels right-aligned, dropdowns for choices (UIC-113), checkboxes with
//! their text on the right (UIC-115), full-width buttons (UIC-116).
//! Strip settings are left out until strips exist (B2; UIC-117: features
//! not built yet are left out, not greyed).
use crate::form::{Unit, rows};
use eframe::egui::Ui;
use operandi_core::Command;
use operandi_core::mesh_stack::OpStatus;
use operandi_core::meshblend::{MeshBlendCommand as C, Role, recipe_of};
use operandi_core::scene::Item;

/// The rows; returns the commands the user's clicks asked for, and
/// whether a field is being dragged (one undo step per drag, SAFE-13).
pub(crate) fn sections(
    ui: &mut Ui,
    scene: &operandi_core::Scene,
    item: &Item,
    status: Option<&OpStatus>,
) -> (Vec<Command>, bool) {
    let mut out = Vec::new();
    let mut dragged = false;
    let Some((_, recipe)) = recipe_of(scene, item.id) else {
        return (out, false);
    };
    let blend = Some(item.id);
    let name = |id: u64| {
        scene
            .item(id)
            .map_or_else(|| format!("Missing item {id}"), |i| i.name.clone())
    };
    let notes = match status {
        Some(OpStatus::Noted(n)) => n.clone(),
        _ => Vec::new(),
    };
    for n in notes.iter().filter(|n| n.item == 0) {
        rows::note(ui, &n.text);
    }
    let primaries: Vec<u64> = recipe
        .sources
        .iter()
        .filter(|s| s.role == Role::Primary)
        .map(|s| s.item)
        .collect();
    let roles: Vec<(Role, &str)> = Role::ALL.iter().map(|r| (*r, r.label())).collect();
    for s in &recipe.sources {
        let title = format!("{} ({})", name(s.item), s.role.label());
        rows::section(ui, &format!("MeshBlend source {}", s.item), false, |ui| {
            rows::value(ui, "Source", &title);
            for n in notes.iter().filter(|n| n.item == s.item) {
                if n.problem {
                    ui.colored_label(crate::theme::error(), &n.text);
                } else {
                    rows::note(ui, &n.text);
                }
            }
            let mut role = s.role;
            if rows::choice_d(
                ui,
                "Role",
                &mut role,
                &Role::Primary,
                &roles,
                "Primary adds volume; a Subtractive Trim cuts it away; an Intersect Trim keeps only the overlap (meshblend.role)",
            )
            .changed()
                && role != s.role
            {
                out.push(Command::MeshBlend(C::Role {
                    blend,
                    source: s.item,
                    role,
                }));
            }
            if s.role.is_trim() {
                for &p in primaries.iter().filter(|&&p| p != s.item) {
                    let mut on = s.trims.contains(&p);
                    if rows::check_d(
                        ui,
                        &format!("Acts on {}", name(p)),
                        &mut on,
                        true,
                        "A Trim only changes the Primaries it acts on (meshblend.trim / meshblend.untrim)",
                    )
                    .changed()
                    {
                        out.push(Command::MeshBlend(C::Trim {
                            blend,
                            trims: vec![s.item],
                            of: vec![p],
                            untrim: !on,
                        }));
                    }
                }
            }
            let mut enabled = s.enabled;
            if rows::check_d(
                ui,
                "In Result",
                &mut enabled,
                true,
                "Off leaves this source out of the result; hiding it in the viewport doesn't (meshblend.source enabled)",
            )
            .changed()
            {
                out.push(source(blend, s.item, Some(enabled), None, None));
            }
            let mut level = s.subdivision;
            if rows::number_d(
                ui,
                "Subdivision",
                &mut level,
                2,
                Unit::Count,
                0..=3,
                "How finely the smooth source becomes polygons; 0 uses the cage as it is (meshblend.source subdivision)",
            )
            .changed()
                && level != s.subdivision
            {
                out.push(source(blend, s.item, None, Some(level), None));
            }
            let mut cage = s.keep_cage;
            if rows::check_d(
                ui,
                "Keep Cage Points",
                &mut cage,
                false,
                "Use the cage's own points instead of its smooth surface (meshblend.source keepCage)",
            )
            .changed()
            {
                out.push(source(blend, s.item, None, None, Some(cage)));
            }
            if rows::button(ui, "Take Out of MeshBlend")
                .on_hover_text("Clear its role: it is an ordinary mesh again (meshblend.clearRole)")
                .clicked()
            {
                out.push(Command::MeshBlend(C::Clear {
                    blend,
                    source: s.item,
                }));
            }
        });
    }
    // SB-36: each strip by its stable id, with its own Width and Profile.
    let values = item
        .mesh_stack
        .as_ref()
        .and_then(|s| s.ops.first())
        .map(|o| o.kind.values())
        .unwrap_or_default();
    let strips = operandi_core::meshblend::op::last_strips(&values);
    if !strips.is_empty() {
        rows::section(ui, "MeshBlend Strips", false, |ui| {
            for s in &strips {
                let over = recipe.strips.iter().find(|o| o.id == s.id);
                let on = over.is_some_and(|o| o.enabled);
                let n: usize =
                    s.id.rsplit('-')
                        .next()
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(0);
                let title = format!("{} / {} {}", name(s.items.0), name(s.items.1), n + 1);
                rows::value(ui, "Strip", &title).on_hover_text(format!("Strip id {}", s.id));
                let mut own = on;
                if rows::check_d(
                    ui,
                    "Override Defaults",
                    &mut own,
                    false,
                    "Give this strip its own Width and Profile (meshblend.strip)",
                )
                .changed()
                {
                    out.push(strip(blend, &s.id, Some(own), None, None));
                }
                ui.add_enabled_ui(on, |ui| {
                    let mut w = over.and_then(|o| o.width).unwrap_or(recipe.strip_width);
                    let r = rows::number_d(
                        ui,
                        "Width",
                        &mut w,
                        recipe.strip_width,
                        Unit::Distance,
                        0.0..=1e9,
                        "This strip's Strip Width (meshblend.strip width)",
                    );
                    dragged |= r.dragged();
                    if r.changed() {
                        out.push(strip(blend, &s.id, None, Some(w), None));
                    }
                    let mut p = over.and_then(|o| o.profile).unwrap_or(recipe.strip_profile) * 100.;
                    let r = rows::number_d(
                        ui,
                        "Profile",
                        &mut p,
                        recipe.strip_profile * 100.,
                        Unit::Percent,
                        0.0..=200.0,
                        "This strip's Strip Profile: 0 % chamfer, 100 % round (meshblend.strip profile)",
                    );
                    dragged |= r.dragged();
                    if r.changed() {
                        out.push(strip(blend, &s.id, None, None, Some(p / 100.)));
                    }
                });
            }
        });
    }
    rows::section(ui, "MeshBlend", false, |ui| {
        // Meshes that could be added: not sources yet, not this item.
        let mut candidates: Vec<(u64, String)> = vec![(0, "(choose a mesh)".into())];
        candidates.extend(
            scene
                .items
                .iter()
                .filter(|i| i.is_mesh() && i.id != item.id && recipe.source(i.id).is_none())
                .map(|i| (i.id, i.name.clone())),
        );
        let options: Vec<(u64, &str)> = candidates.iter().map(|(i, n)| (*i, n.as_str())).collect();
        for (label, role, tip) in [
            (
                "Add Primary",
                Role::Primary,
                "Add a mesh whose volume joins the result (meshblend.addSource)",
            ),
            (
                "Add Subtractive Trim",
                Role::Subtract,
                "Add a mesh that cuts its volume out of every Primary (meshblend.addSource role:subtract)",
            ),
            (
                "Add Intersect Trim",
                Role::Intersect,
                "Add a mesh that keeps only where it overlaps every Primary (meshblend.addSource role:intersect)",
            ),
        ] {
            let mut pick = 0u64;
            rows::choice_d(ui, label, &mut pick, &0, &options, tip);
            if pick != 0 {
                out.push(Command::MeshBlend(C::AddSource {
                    blend,
                    source: pick,
                    role,
                    of: Vec::new(),
                }));
            }
        }
        if rows::button(ui, "Convert to Mesh")
            .on_hover_text(
                "Freeze the result into this item as an ordinary mesh (meshblend.convert)",
            )
            .clicked()
        {
            out.push(Command::MeshBlend(C::Convert {
                blend,
                duplicate: false,
            }));
        }
        if rows::button(ui, "Duplicate and Convert")
            .on_hover_text("Make a new ordinary mesh from the result; this MeshBlend stays live (meshblend.convert duplicate:true)")
            .clicked()
        {
            out.push(Command::MeshBlend(C::Convert {
                blend,
                duplicate: true,
            }));
        }
    });
    (out, dragged)
}

fn strip(
    blend: Option<u64>,
    id: &str,
    enabled: Option<bool>,
    width: Option<f64>,
    profile: Option<f64>,
) -> Command {
    Command::MeshBlend(C::Strip {
        blend,
        id: id.into(),
        enabled,
        width,
        profile,
        clear: false,
    })
}

fn source(
    blend: Option<u64>,
    item: u64,
    enabled: Option<bool>,
    subdivision: Option<u8>,
    keep_cage: Option<bool>,
) -> Command {
    Command::MeshBlend(C::Source {
        blend,
        source: item,
        enabled,
        subdivision,
        keep_cage,
    })
}
