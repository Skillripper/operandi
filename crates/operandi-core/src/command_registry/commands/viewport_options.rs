//! Viewport Properties rows and view header toggles (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // Viewport Properties rows (viewport_properties.rs VIEW3D); bgmesh:1
    // is the Inactive Meshes group (provisional).
    script("view3d.controls", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.controls ?"),
    script("view3d.showHUD", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showHUD ?"),
    script("view3d.showWireframeItemMode", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showWireframeItemMode ?"),
    script("view3d.bgEnvironment", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.bgEnvironment ?"),
    script("view3d.showLights", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showLights ?"),
    script("view3d.showCameras", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showCameras ?"),
    script("view3d.showMeshes", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showMeshes ?"),
    script("view3d.showGrid", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showGrid ?"),
    script("view3d.showWorkPlane", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showWorkPlane ?"),
    script("view3d.fadedGrid", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.fadedGrid ?"),
    script("view3d.fadedGridAxes", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.fadedGridAxes ?"),
    script("view3d.showLocators", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showLocators ?"),
    script("view3d.showTextureLocators", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showTextureLocators ?"),
    script("view3d.showLightIllumination", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showLightIllumination ?"),
    script("view3d.showSelections", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showSelections ?"),
    script("view3d.selectedNormals", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.selectedNormals ?"),
    script("view3d.fillSelected", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.fillSelected ?"),
    script("view3d.outlineSelected", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.outlineSelected ?"),
    script("view3d.showBackdrop", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showBackdrop ?"),
    script("view3d.useTrackball", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.useTrackball ?"),
    script("view3d.orbitSelected", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.orbitSelected ?"),
    script("view3d.gnzAmbientIntensity", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.gnzAmbientIntensity ?"),
    script("operandi.view3d.showSymmetryPlane", ViewportOptions, "Show Symmetry Plane of the active view: the mirror plane's outline while Symmetry is on (off hides only the outline); ? reads it.")
        .operandi()
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("operandi.view3d.showSymmetryPlane false"),
    script("operandi.view3d.matcap", ViewportOptions, "The active view's matcap (the Matcap style): a built-in name, or custom with image:<path>; name:custom without image: opens the file chooser; all:true sets it in every 3D view, each switched to the Matcap style (the Preset Browser's Use in All 3D Views); ? reads it.")
        .operandi()
        .args(&[
            arg("name", TEXT, N),
            opt("image", TEXT, N, None),
            opt("bgmesh", INT, N, Some("0")),
            opt("all", BOOL, N, Some("false")),
        ])
        .query()
        .ui()
        .example("operandi.view3d.matcap name:fullmetal")
        .example("operandi.view3d.matcap ?"),
    script("view3d.gnzAmbientColor", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.gnzAmbientColor ?"),
    script("view3d.indPointSize", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.indPointSize ?"),
    script("view3d.pointSize", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.pointSize ?"),
    script("view3d.showSelectionRollover", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showSelectionRollover ?"),
    script("view3d.meshSmoothing", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.meshSmoothing ?"),
    script("view3d.showGnzShadows", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showGnzShadows ?"),
    script("view3d.showUVBounds", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showUVBounds ?"),
    // HT-5/HT-6: the view header's Weightmaps and UV Distortion toggles.
    script("view3d.showweights", ViewportOptions, "Show Weightmaps in the active view: meshes tinted by the selected weight map; ? reads it.")
        // In the name map since HT-Q5 (state only); bgmesh is accepted like
        // the other view3d settings and changes nothing.
        .args(&[arg("state", BOOL, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showweights true"),
    script("view.topologyMode", ViewportOptions, "Topology Mode in the active view: the active meshes draw over the background meshes, see-through (missing-tools-g3 MT7); bare toggles.")
        .operandi()
        .args(&[opt("state", BOOL, N, None)])
        .ui()
        .example("view.topologyMode on"),
    script("view3d.showDistortion", ViewportOptions, "Show UV Distortion in the active view: polygons tinted by UV stretch; ? reads it.")
        // In the name map since HT-Q5 (enabled, bgmesh). Provisional:
        // bgmesh:1 is accepted but treated as 0 (the toggle is one per view,
        // not split into active/inactive meshes; HT-Q5).
        .args(&[arg("enabled", BOOL, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.showDistortion true"),
    script("view3d.sameAsActive", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.sameAsActive ?"),
    script("view3d.inactiveInvisible", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.inactiveInvisible ?"),
    script("view3d.projection", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.projection ?"),
    script("view3d.shadingStyle", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.shadingStyle ?"),
    script("view3d.wireframeOverlay", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.wireframeOverlay ?"),
    script("view3d.wireframeAlpha", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.wireframeAlpha ?"),
    script("view3d.wireframeViewColor", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.wireframeViewColor ?"),
    script("view3d.TransparencyOverride", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverride ?"),
    script("view3d.TransparencyOverrideSolidColorFront", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverrideSolidColorFront ?"),
    script("view3d.TransparencyOverrideSolidAlphaFront", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverrideSolidAlphaFront ?"),
    script("view3d.TransparencyOverrideSolidColorBack", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverrideSolidColorBack ?"),
    script("view3d.TransparencyOverrideSolidAlphaBack", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverrideSolidAlphaBack ?"),
    script("view3d.TransparencyOverrideOverlayMode", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverrideOverlayMode ?"),
    script("view3d.TransparencyOverrideFlatShade", ViewportOptions, "Viewport Properties setting of the active view; ? reads it.")
        .args(&[arg("value", TEXT, N), opt("bgmesh", INT, N, Some("0"))])
        .query()
        .ui()
        .example("view3d.TransparencyOverrideFlatShade ?"),
    script("view3d.showVertices", ViewportOptions, "Shows or hides vertices in the views.")
        .args(&[opt("state", ON_OFF_QUERY, N, None)])
        .query()
        .ui(),
];
