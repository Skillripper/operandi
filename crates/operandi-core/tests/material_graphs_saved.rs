//! Materials stage 4 (docs/MATERIALX_GRAPHS.md, AR-32): scene files keep
//! every surface's MaterialX graph as the master copy, old scenes upgrade
//! on open, and a saved graph wins where the layers would now render
//! differently.
use glam::DVec3;
use operandi_core::materialx::{saved, tree};
use operandi_core::procedural::ProceduralKind;
use operandi_core::shader_eval::{Context, Expr, Input};
use operandi_core::shader_tree::{ItemMask, NewLayer, NodeKind, ShaderTreeEdit, TagMask, TagType};
use operandi_core::{Command, Editor, Scene, persistence};

fn add(e: &mut Editor, layer: NewLayer, selected: Option<u64>) -> u64 {
    e.dispatch(Command::ShaderTree(ShaderTreeEdit::Add { layer, selected }))
        .unwrap();
    e.scene.shader_tree.next_id - 1
}

/// Two cubes, one moved and turned; tagged polygons (material, part,
/// selection set); masked groups and one of every texture source,
/// procedural kind and several gradient inputs.
fn rich_scene() -> Editor {
    let mut e = Editor::default();
    for _ in 0..2 {
        e.scene.new_mesh().unwrap();
    }
    let ids: Vec<u64> = e.scene.items.iter().map(|i| i.id).collect();
    let (a, b) = (ids[ids.len() - 2], ids[ids.len() - 1]);
    for (k, id) in [a, b].into_iter().enumerate() {
        let i = e.scene.items.iter().position(|i| i.id == id).unwrap();
        let item = &mut e.scene.items[i];
        item.mesh = operandi_core::cube_geometry(DVec3::ZERO, DVec3::ONE)
            .unwrap()
            .into();
        item.mesh.faces[0].material = "Red".into();
        item.mesh.faces[1].part = Some("Lid".into());
        item.mesh
            .attributes
            .polygon_sets
            .insert("Top".into(), [2u32, 3].into_iter().collect());
        if k == 1 {
            item.transform.translation = DVec3::new(3., 1., -2.);
            item.transform.rotation = DVec3::new(10., 35., 0.);
        }
    }
    e.scene.images.push(operandi_core::backdrop::SceneImage {
        id: 9001,
        name: "Stripes".into(),
        path: concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/relief-stripes.png"
        )
        .into(),
        width: 64,
        height: 64,
    });
    e.scene.next_id = e.scene.next_id.max(9002);
    // Masked groups, each with a material and a texture.
    let masks = [
        (ItemMask::Item(b), TagType::Material, TagMask::All),
        (
            ItemMask::All,
            TagType::Material,
            TagMask::Name("Red".into()),
        ),
        (ItemMask::All, TagType::Part, TagMask::Name("Lid".into())),
        (
            ItemMask::All,
            TagType::SelectionSet,
            TagMask::Name("Top".into()),
        ),
    ];
    for (n, (item, tag_type, tag)) in masks.into_iter().enumerate() {
        let g = add(&mut e, NewLayer::Group, None);
        let node = e.scene.shader_tree.get(g).unwrap().clone();
        let mut node = node;
        if let NodeKind::Group(m) = &mut node.kind {
            m.item = item;
            m.tag_type = tag_type;
            m.tag = tag;
        }
        let at = e
            .scene
            .shader_tree
            .nodes
            .iter()
            .position(|x| x.id == g)
            .unwrap();
        e.scene.shader_tree.nodes[at] = node;
        add(&mut e, NewLayer::Material, Some(g));
        let kind = ProceduralKind::ALL[n % ProceduralKind::ALL.len()];
        add(&mut e, NewLayer::Procedural(kind), Some(g));
        let c = add(&mut e, NewLayer::Constant, Some(g));
        let at = e
            .scene
            .shader_tree
            .nodes
            .iter()
            .position(|x| x.id == c)
            .unwrap();
        if let NodeKind::Texture(t) = &mut e.scene.shader_tree.nodes[at].kind
            && let operandi_core::shader_tree::TextureSource::Constant { color, .. } = &mut t.source
        {
            *color = [0.2 * n as f64, 0.5, 0.1];
        }
    }
    for kind in ProceduralKind::ALL {
        add(&mut e, NewLayer::Procedural(*kind), None);
    }
    for layer in [
        NewLayer::Constant,
        NewLayer::ImageMap,
        NewLayer::Gradient,
        NewLayer::VertexMap,
        NewLayer::Variation,
        NewLayer::Process,
        NewLayer::Occlusion,
        NewLayer::Shader,
    ] {
        add(&mut e, layer, None);
    }
    // The image layer shows the picture; one gradient per input.
    use operandi_core::gradient::GradientInput;
    for input in GradientInput::ALL {
        let g = add(&mut e, NewLayer::Gradient, None);
        let at = e
            .scene
            .shader_tree
            .nodes
            .iter()
            .position(|x| x.id == g)
            .unwrap();
        if let NodeKind::Texture(t) = &mut e.scene.shader_tree.nodes[at].kind
            && let operandi_core::shader_tree::TextureSource::Gradient(g) = &mut t.source
        {
            g.input = *input;
        }
    }
    // Every texture half-covers what's below, with a blend mode and
    // effect of its own, so each one shows in the surface.
    use operandi_core::shader_tree::{BlendMode, TextureEffect};
    let mut k = 0;
    for node in &mut e.scene.shader_tree.nodes {
        if let NodeKind::Texture(t) = &mut node.kind {
            if let operandi_core::shader_tree::TextureSource::ImageMap { image, .. } = &mut t.source
            {
                *image = Some(9001);
            }
            if node.parent.is_none() {
                t.effect = TextureEffect::ALL[k % TextureEffect::ALL.len()];
            }
            node.layer.opacity = 0.5;
            node.layer.blend = BlendMode::ALL[k % BlendMode::ALL.len()];
            k += 1;
        }
    }
    e.scene.validate().unwrap();
    e
}

fn text(scene: &Scene) -> String {
    String::from_utf8(persistence::encode(scene).unwrap()).unwrap()
}

#[test]
fn a_scene_saves_every_surface_as_a_graph_and_reopens_identically() {
    let e = rich_scene();
    let first = text(&e.scene);
    assert!(first.contains("\"version\": 65"), "saved at version 65");
    let graphs = saved::graphs(&e.scene);
    // Masks by item, material, part and set give several surfaces.
    assert!(graphs.graphs.len() >= 5, "{} graphs", graphs.graphs.len());
    assert!(
        graphs
            .graphs
            .iter()
            .all(|g| g.materialx.contains("<operandi_surface ")),
        "every graph is the MaterialX form"
    );
    // Every polygon kind is bound: 2 items × (plain, Red, Lid, Top).
    assert_eq!(graphs.bindings.len(), 8);
    let opened = persistence::decode(first.as_bytes()).unwrap();
    assert!(
        opened.material_pins.is_empty(),
        "the layers agree with the graphs"
    );
    assert_eq!(saved::graphs(&opened), graphs);
    // Save -> open -> save gives the same file (AR-32's check).
    assert_eq!(text(&opened), first);
}

#[test]
fn items_with_the_same_material_share_one_graph_wherever_they_are() {
    let mut e = rich_scene();
    let before = saved::graphs(&e.scene);
    assert!(
        before.graphs.iter().any(|g| g.placed),
        "placement goes by reference"
    );
    // Three more copies of the first cube, moved and turned: the same
    // materials, seen from elsewhere.
    let first = e
        .scene
        .items
        .iter()
        .find(|i| i.is_mesh() && !i.mesh.faces.is_empty())
        .unwrap()
        .clone();
    for n in 1..=3 {
        let mut copy = first.clone();
        copy.id = e.scene.next_id;
        e.scene.next_id += 1;
        copy.name = format!("Copy {n}");
        copy.transform.translation = DVec3::new(5. * n as f64, -2., 1.);
        copy.transform.rotation = DVec3::new(0., 20. * n as f64, 5.);
        e.scene.items.push(copy);
    }
    e.scene.index = Default::default();
    e.scene.validate().unwrap();
    let after = saved::graphs(&e.scene);
    assert_eq!(after.graphs.len(), before.graphs.len(), "no new graphs");
    assert_eq!(after.bindings.len(), before.bindings.len() + 12);
    // And each copy still opens exactly as its layers give it.
    let file = text(&e.scene);
    let opened = persistence::decode(file.as_bytes()).unwrap();
    assert!(opened.material_pins.is_empty());
    assert_eq!(text(&opened), file);
}

#[test]
fn every_saved_graph_reads_back_as_the_surface_it_came_from() {
    let e = rich_scene();
    let graphs = saved::graphs(&e.scene);
    for g in &graphs.graphs {
        let p = tree::read(&g.materialx).unwrap();
        assert_eq!(
            tree::write(&p, &e.scene, &g.name),
            g.materialx,
            "{}",
            g.name
        );
    }
}

#[test]
fn older_scenes_upgrade_on_open_and_keep_their_look() {
    // The icon studio's 250 scenes were saved at version 44, before graphs.
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/icon-studio/icons");
    let mut count = 0;
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "opscene"))
        .collect();
    files.push(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/viewport-image-textures/image-layers.opscene"
        )
        .into(),
    );
    for path in files {
        let old = persistence::load(&path).unwrap();
        assert!(old.material_pins.is_empty());
        let upgraded = text(&old);
        assert!(
            upgraded.contains("\"material_graphs\""),
            "{} saves its graphs",
            path.display()
        );
        let again = persistence::decode(upgraded.as_bytes()).unwrap();
        assert!(again.material_pins.is_empty(), "{}", path.display());
        assert_eq!(
            saved::graphs(&again),
            saved::graphs(&old),
            "{}",
            path.display()
        );
        assert_eq!(text(&again), upgraded, "{}", path.display());
        count += 1;
    }
    assert!(count > 200, "{count} scenes");
}

/// The rich scene's file with every graph's roughness set to `r`, as a
/// build with different layer sums (or an outside edit) would save it.
fn with_saved_roughness(e: &Editor, r: f64) -> String {
    let mut doc: serde_json::Value = serde_json::from_str(&text(&e.scene)).unwrap();
    for g in doc["material_graphs"]["graphs"].as_array_mut().unwrap() {
        let mut p = tree::read(g["materialx"].as_str().unwrap()).unwrap();
        for (input, expr) in &mut p.inputs {
            if *input == Input::Roughness {
                *expr = Expr::scalar(r);
            }
        }
        g["materialx"] = tree::write(&p, &e.scene, "Edited").into();
    }
    serde_json::to_string_pretty(&doc).unwrap()
}

fn roughness(scene: &Scene, item: u64) -> Expr {
    let ancestors = scene.item_ancestors(item);
    let locators = scene.locator_frames(item);
    let ctx = Context {
        item,
        ancestors: &ancestors,
        material: "",
        part: None,
        sets: &[],
        locators: &locators,
    };
    let p = saved::surface(
        scene,
        &ctx,
        scene.shading_tree().evaluate(&ctx).principled(),
    );
    p.inputs
        .iter()
        .find(|(i, _)| *i == Input::Roughness)
        .unwrap()
        .1
        .clone()
}

#[test]
fn the_saved_graph_wins_until_its_layers_are_edited() {
    let e = rich_scene();
    // The moved cube (texture locators are items too, after it).
    let item = e
        .scene
        .items
        .iter()
        .rfind(|i| i.is_mesh() && !i.mesh.faces.is_empty())
        .unwrap()
        .id;
    let file = with_saved_roughness(&e, 0.123);
    let mut opened = persistence::decode(file.as_bytes()).unwrap();
    assert!(
        !opened.material_pins.is_empty(),
        "the edited graphs are pinned"
    );
    assert_eq!(roughness(&opened, item), Expr::scalar(0.123));
    // Saving keeps the graph in force, so it survives another round.
    let resaved = text(&opened);
    let again = persistence::decode(resaved.as_bytes()).unwrap();
    assert_eq!(roughness(&again, item), Expr::scalar(0.123));
    assert_eq!(text(&again), resaved);
    // Editing the layers that shade those polygons makes them the truth.
    let top = opened
        .shader_tree
        .nodes
        .iter()
        .position(|n| matches!(n.kind, NodeKind::Texture(_)))
        .unwrap();
    opened.shader_tree.nodes[top].layer.enable = !opened.shader_tree.nodes[top].layer.enable;
    let layers = {
        let ancestors = opened.item_ancestors(item);
        let locators = opened.locator_frames(item);
        let ctx = Context {
            item,
            ancestors: &ancestors,
            material: "",
            part: None,
            sets: &[],
            locators: &locators,
        };
        opened.shading_tree().evaluate(&ctx).principled()
    };
    let r = layers
        .inputs
        .iter()
        .find(|(i, _)| *i == Input::Roughness)
        .unwrap()
        .1
        .clone();
    assert_ne!(r, Expr::scalar(0.123));
    assert_eq!(roughness(&opened, item), r);
}

#[test]
fn files_with_broken_or_too_new_graphs_are_refused() {
    let e = rich_scene();
    let good = text(&e.scene);
    // A version 64 build never wrote graphs.
    let old = good.replace("\"version\": 65", "\"version\": 64");
    let err = persistence::decode(old.as_bytes()).unwrap_err();
    assert!(err.contains("version 65"), "{err}");
    // A graph that doesn't read is named, not dropped.
    let mut doc: serde_json::Value = serde_json::from_str(&good).unwrap();
    doc["material_graphs"]["graphs"][0]["materialx"] = "<materialx>".into();
    let err = persistence::decode(doc.to_string().as_bytes()).unwrap_err();
    assert!(err.contains("can't be read"), "{err}");
    // A binding to a graph that isn't there.
    let mut doc: serde_json::Value = serde_json::from_str(&good).unwrap();
    doc["material_graphs"]["bindings"][0]["graph"] = 999.into();
    let err = persistence::decode(doc.to_string().as_bytes()).unwrap_err();
    assert!(err.contains("graph 999"), "{err}");
}

#[test]
fn a_new_scene_saves_no_graphs() {
    let mut e = Editor::default();
    e.scene.new_mesh().unwrap();
    let t = text(&e.scene);
    assert!(!t.contains("material_graphs"));
    assert!(!t.contains("\"version\": 65"));
}

/// Opt-in probe: what saving the graphs costs on many textured items.
/// `cargo test --release --workspace -- material_graphs_saved::probe --ignored --nocapture`
#[test]
#[ignore]
fn probe_save_cost_with_many_items() {
    let base = rich_scene();
    let mesh_ids: Vec<u64> = base
        .scene
        .items
        .iter()
        .filter(|i| i.is_mesh() && !i.mesh.faces.is_empty())
        .map(|i| i.id)
        .collect();
    let template = base.scene.item(mesh_ids[1]).unwrap().clone();
    for count in [100usize, 1000] {
        let mut scene = base.scene.clone();
        for n in 0..count {
            let mut item = template.clone();
            item.id = scene.next_id;
            scene.next_id += 1;
            item.name = format!("Copy {n}");
            item.transform.translation = DVec3::new(n as f64 * 2., 0., 0.);
            scene.items.push(item);
        }
        scene.index = Default::default();
        let t = std::time::Instant::now();
        let graphs = saved::graphs(&scene);
        let graph_time = t.elapsed();
        let without = {
            let mut s = scene.clone();
            s.shader_tree = Default::default();
            s.materials.clear();
            persistence::encode(&s).unwrap().len()
        };
        let t = std::time::Instant::now();
        let with = persistence::encode(&scene).unwrap().len();
        let save_time = t.elapsed();
        eprintln!(
            "{count} items: {} graphs, {} bindings, graphs {graph_time:?}, save {save_time:?}, file {} KB (no materials {} KB)",
            graphs.graphs.len(),
            graphs.bindings.len(),
            with / 1024,
            without / 1024
        );
    }
}
