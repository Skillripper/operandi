//! Set Material on an existing material keeps that material's settings
//! (GitHub #12). Acceptance criteria:
//!
//! 1. The dialog's starting values for an existing tag are its Shader Tree
//!    Material's (edited there, they differ from the stored table entry).
//! 2. Assigning an existing tag by name leaves every setting of its tree
//!    Material as it was, including ones the dialog doesn't show.
//! 3. Undo removes the new tags and leaves the material alone.
use glam::DVec3;
use operandi_core::shader_tree::*;
use operandi_core::*;

fn tagged_cube() -> (Editor, NodeId) {
    let mut e = Editor::default();
    e.scene.new_mesh().unwrap();
    e.scene.active_mut().unwrap().mesh = Mesh::box_mesh(DVec3::ZERO, DVec3::ONE, 1).unwrap().into();
    e.scene.selection.mode = Mode::Polygon;
    e.scene.selection.faces = vec![0];
    e.synchronize_identities().unwrap();
    e.dispatch(Command::SetMaterial(MaterialAssignment {
        name: "Paint".into(),
        expected: None,
        material: Some(Material::default()),
    }))
    .unwrap();
    // Edit the tree Material the way its properties panel does.
    let id = e.scene.shader_tree.material_for_tag("Paint").unwrap();
    let mut n = e.scene.shader_tree.get(id).unwrap().clone();
    if let NodeKind::Material(m) = &mut n.kind {
        m.diffuse_color = [0.9, 0.1, 0.2];
        m.diffuse_amount = 0.4;
        m.roughness = 0.123;
        m.smoothing = 0.5;
    }
    e.dispatch(Command::ShaderTree(ShaderTreeEdit::Replace {
        node: n,
        continuing: false,
    }))
    .unwrap();
    (e, id)
}

fn channels(e: &Editor, id: NodeId) -> MaterialChannels {
    match &e.scene.shader_tree.get(id).unwrap().kind {
        NodeKind::Material(m) => (**m).clone(),
        _ => panic!("not a Material"),
    }
}

#[test]
fn the_dialog_shows_the_tree_materials_settings() {
    let (e, _) = tagged_cube();
    let shown = e.scene.material_shown("Paint");
    assert_eq!(shown.color, [0.9, 0.1, 0.2]);
    assert_eq!(shown.diffuse, 0.4);
    assert!(shown.smoothing);
    // A name with no material shows factory settings.
    assert_eq!(e.scene.material_shown("Nothing"), Material::default());
}

#[test]
fn assigning_an_existing_material_keeps_its_settings() {
    let (mut e, id) = tagged_cube();
    let before = channels(&e, id);
    e.scene.selection.faces = vec![1, 2];
    e.dispatch(Command::SetMaterial(MaterialAssignment {
        name: "Paint".into(),
        expected: None,
        material: None,
    }))
    .unwrap();
    assert_eq!(channels(&e, id), before);
    let mesh = &e.scene.active().unwrap().mesh;
    assert!([0, 1, 2].iter().all(|&f| mesh.faces[f].material == "Paint"));
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(channels(&e, id), before);
    assert_eq!(e.scene.active().unwrap().mesh.faces[1].material, "");
}
