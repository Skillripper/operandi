//! Library packs, online libraries and content folders (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // Library pack (F1). Operandi-only: the map has no command for it.
    // Locating a library is pbview.basePathAdd (registry ruling 2026-11-11).
    script("operandi.pbview.libraryGet", Presets, "Opens the free Operandi Library's web page (or url:) in the default web browser; downloads nothing and isn't undoable.")
        .operandi()
        .args(&[opt("url", TEXT, N, Some("https://operandi3d.org/library"))])
        .ui(),
    // Online libraries in the preset browser (A34). Operandi-only; each
    // contacts only sources turned on in Preferences.
    script("operandi.online.clearCache", Presets, "Removes the online libraries' downloaded files and thumbnails (one source:, or all); a source that's off loses its whole cached folder. Not undoable.")
        .operandi()
        .args(&[opt("source", ArgType::Choice(&["polyhaven", "ambientcg"]), N, None)])
        .side_effect(),
    script("operandi.online.download", Presets, "Downloads an online library's asset (source:, kind:, id:) at resolution: (thousands of pixels; default the Download Resolution) into the cache, in the background; load:true then loads it as the Preset Browser would.")
        .operandi()
        .args(&[
            arg("source", ArgType::Choice(&["polyhaven", "ambientcg"]), N),
            arg("kind", ArgType::Choice(&["hdri", "texture", "model"]), N),
            arg("id", TEXT, N),
            opt("resolution", INT, N, None),
            opt("load", BOOL, N, Some("false")),
        ])
        .example("operandi.online.download source:polyhaven kind:hdri id:aarfontein_dawn load:true")
        .side_effect(),
    script("operandi.online.refresh", Presets, "Fetches the catalogue of each online library that's on (or source:) again, in the background.")
        .operandi()
        .args(&[opt("source", ArgType::Choice(&["polyhaven", "ambientcg"]), N, None)])
        .side_effect(),
    // Modo content folder (MC-20, MC-02; registry thread 2026-11-12).
    script("operandi.pbview.kindFilter", Presets, "Shows only one kind in the Preset Browser: all, meshes, materials, environments, images or scenes (0 to 5); ? returns it. Not undoable.")
        .operandi()
        .args(&[arg(
            "kind",
            ArgType::Choice(&["all", "meshes", "materials", "environments", "images", "scenes"]),
            N,
        )])
        .ui(),
    script("operandi.pbview.onlyUsable", Presets, "Hides (true) or lists greyed (false) files the Preset Browser can't use; bare toggles, ? returns it. Not undoable.")
        .operandi()
        .args(&[opt("state", BOOL, N, None)])
        .ui(),
    script("dircache.copy", Presets, "Copies folder source: into dest: (default: the library folder) in the background; an existing folder of that name needs overwrite:true. Not undoable.")
        .args(&[
            arg("source", ArgType::Path, N),
            opt("dest", ArgType::Path, N, None),
            opt("overwrite", BOOL, N, Some("false")),
        ])
        .ui(),
    script("file.rename", Presets, "Renames the selected Preset Browser entry (path: for scripts); a texture set's maps together.")
        .args(&[arg("newName", TEXT, N), opt("path", ArgType::Path, N, None)]).side_effect(),
    script("file.duplicate", Presets, "Copies the selected Preset Browser entry under a new name.")
        .args(&[arg("newName", TEXT, N), opt("path", ArgType::Path, N, None)]).side_effect(),
    script("file.delete", Presets, "Deletes the selected Preset Browser entry from disk (the menu asks first).")
        .args(&[opt("path", ArgType::Path, N, None)]).side_effect(),
];
