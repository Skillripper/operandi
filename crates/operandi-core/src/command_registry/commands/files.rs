//! Debug and file commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- debug
    // The app's benchmarks (operandi.bench.*) need its window: the app
    // registers them itself (operandi-app command_handlers.rs).
    // ================================================= typed actions
    // ---------------------------------------------------------- files
    script("scene.open", Files, "Opens a scene file; without a filename opens the file panel.")
        .aliases(&["Open"])
        .args(&[opt("filename", ArgType::Path, N, None)])
        .example("scene.open /nonexistent.opscene").side_effect(),
    script("scene.new", Files, "Opens a new, empty scene in its own tab.").side_effect(),
    script("scene.save", Files, "Saves the current scene; a scene never saved asks for a file name.").side_effect(),
    script("scene.saveAs", Files, "Saves the scene under a file name; a format (or a filename whose extension names one) writes an interchange file instead, leaving the scene's own name alone. Without a filename opens the save panel.")
        .headless(
            "Export the scene to an interchange file, as File > Export does (Mesh Stack results, hidden items as hidden). The document keeps its own file name and unsaved state, and nothing is added to the history. Here it only exports: it needs filename plus a format (format:, or a file extension that names one); to save the .opscene use scene_save.",
            &[
            jreq("filename", "string", "The file to write."),
            jopt(
                "format",
                "string",
                "obj, stl, stlAscii, gltf, glb, geo, svg, lxo, ply, plyAscii, abc, usd, usda, usdc, usdz, fbx, 3mf, step or iges; default from the file's extension (.stl and .ply mean binary, .stp and .igs are STEP and IGES). USD needs the running app's exporter (operandi-mcp --attach).",
            ),
            jopt(
                "export",
                "boolean",
                "Accepted; here scene.saveAs always exports.",
            ),
            jopt(
                "animation",
                "string",
                "USD: on (default; time samples once a frame over the scene range), off, or current (the current frame only).",
            ),
        ],
        )
        .args(&[
            opt("filename", ArgType::Path, N, None),
            opt("format", EXPORT_FORMATS, N, None),
            opt("export", BOOL, N, Some("false")),
            opt("animation", ArgType::Choice(&["on", "off", "current"]), N, None),
        ]).side_effect(),
    script("scene.close", Files, "Closes the current scene (asks first when it has unsaved changes).").side_effect(),
    script("scene.closeAll", Files, "Asks to close every open scene.").side_effect(),
    // USD import (docs/BEHAVIOR-USD-IMPORT.md): Operandi's own name, a
    // placeholder until the registry thread rules on it.
    script("operandi.usd.import", Files, "Opens a USD file (.usd, .usda, .usdc, .usdz) as a new unsaved scene with these import options and shows the import report: upAxis file/y/z, units file/m/cm/mm/in/ft, materials materialx/preview/none, cameras, lights and graphs (the node graph sidecar, graphsFile or <name>_graphs.json beside it) true/false. Without a filename opens the file panel, whose USD files ask the Import USD options.")
        .operandi()
        .args(&[
            opt("filename", ArgType::Path, N, None),
            opt("upAxis", ArgType::Choice(&["file", "y", "z"]), N, Some("file")),
            opt("units", ArgType::Choice(&["file", "m", "cm", "mm", "in", "ft"]), N, Some("file")),
            opt("materials", ArgType::Choice(&["materialx", "preview", "none"]), N, Some("materialx")),
            opt("cameras", BOOL, N, Some("true")),
            opt("lights", BOOL, N, Some("true")),
            opt("graphs", BOOL, N, Some("true")),
            opt("graphsFile", ArgType::Path, N, None),
            opt("instances", ArgType::Choice(&["instances", "copies"]), N, Some("instances")),
        ])
        .side_effect(),
    script("scene.importReference", Files, "Adds a scene or model file's items (Operandi or Modo scene, OBJ, STL, glTF, GEO, PLY, FBX, 3MF, Alembic, USD; picked by extension) to the current scene, selected, as one undo step; meshes/shaders/lights/environments/cameras:false leave those out. Without a filename opens the import panel. scene.open opens a file as its own scene.")
        .args(&[
            opt("filename", ArgType::Path, N, None),
            opt("meshes", BOOL, N, Some("true")),
            opt("shaders", BOOL, N, Some("true")),
            opt("lights", BOOL, N, Some("true")),
            opt("environments", BOOL, N, Some("true")),
            opt("cameras", BOOL, N, Some("true")),
        ]).undo(),
];
