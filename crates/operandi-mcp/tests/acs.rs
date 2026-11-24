//! Character rigs through the bridge (BEHAVIOR-ACS-FEATURES ACS-00): a
//! Biped rig made, its guides moved with symmetry and fitted to a mesh,
//! the context switched, the skeleton built and a mesh bound, each by
//! `command_run`, read back with `operandi.acs.query`.
use operandi_mcp::{Bridge, protocol};
use serde_json::{Value, json};

fn call(b: &mut Bridge, name: &str, arguments: Value) -> Result<Value, String> {
    let r = protocol::handle(
        b,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": {"name": name, "arguments": arguments}}),
    )
    .expect("a request gets a response");
    let result = &r["result"];
    if result["isError"] == json!(true) {
        Err(result["content"][0]["text"].as_str().unwrap().to_string())
    } else {
        Ok(result["structuredContent"].clone())
    }
}

fn run(b: &mut Bridge, command: &str, args: Value) {
    if let Err(e) = call(b, "command_run", json!({"command": command, "args": args})) {
        panic!("{command} {args}: {e}");
    }
}

fn rig(b: &mut Bridge) -> Value {
    let out = call(
        b,
        "command_run",
        json!({"command": "operandi.acs.query", "args": {}}),
    )
    .unwrap();
    let v: Value = serde_json::from_str(out["value"].as_str().unwrap()).unwrap();
    v["rigs"][0].clone()
}

fn part<'a>(list: &'a Value, role: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|p| p["role"] == role)
        .unwrap_or_else(|| panic!("no {role}"))
}

fn at(p: &Value) -> [f64; 3] {
    let a = p["position"].as_array().unwrap();
    [0, 1, 2].map(|i| a[i].as_f64().unwrap())
}

#[test]
fn acs_biped_guides_skeleton_and_bind_through_the_bridge() {
    let mut b = Bridge::new();
    run(&mut b, "operandi.acs.rigAdd", json!({"height": 1.8}));
    let r = rig(&mut b);
    assert_eq!(r["context"], "guide");
    assert_eq!(r["guides"].as_array().unwrap().len(), 25);
    let root = r["root"].as_u64().unwrap();

    // ACS-31: moving the left knee moves the right one, mirrored.
    run(
        &mut b,
        "operandi.acs.guideMove",
        json!({"guide": "leg.L.knee", "position": "0.15,0.5,0.04"}),
    );
    let r = rig(&mut b);
    let k = at(part(&r["guides"], "leg.R.knee"));
    assert!(
        (k[0] + 0.15).abs() < 1e-9 && (k[1] - 0.5).abs() < 1e-9,
        "{k:?}"
    );

    // ACS-11/22: options.
    run(
        &mut b,
        "operandi.acs.options",
        json!({"rig": root, "spine": 2, "upperArmTwist": 1}),
    );
    assert_eq!(rig(&mut b)["options"]["spine"], 2);

    // ACS-34: fit to a body-shaped box.
    run(&mut b, "mesh.new", json!({}));
    let added = call(&mut b, "primitive_add", json!({"shape": "cube"})).unwrap();
    let mesh = added["active"]["item"].as_u64().unwrap();
    for (name, v) in [
        ("scl.X", 0.6),
        ("scl.Y", 1.7),
        ("scl.Z", 0.3),
        ("pos.Y", 0.85),
    ] {
        run(
            &mut b,
            "transform.channel",
            json!({"name": name, "value": v, "item": mesh}),
        );
    }
    run(
        &mut b,
        "operandi.acs.guideFit",
        json!({"rig": root, "mesh": mesh}),
    );
    let r = rig(&mut b);
    let size = r["referenceSize"].as_f64().unwrap();
    assert!((size - 1.7).abs() < 1e-6, "{size}");

    // ACS-32/40: build; one vertebra fewer, one twist joint more a side.
    run(&mut b, "operandi.acs.build", json!({"rig": root}));
    let r = rig(&mut b);
    assert_eq!(r["joints"].as_array().unwrap().len(), 28 - 1 + 2);
    let hips = at(part(&r["joints"], "spine.C.hips"));
    let guide = at(part(&r["guides"], "spine.C.hips"));
    assert_eq!(hips, guide);

    // ACS-01/02: the context switch is read back.
    run(
        &mut b,
        "operandi.acs.context",
        json!({"rig": root, "context": "animate"}),
    );
    assert_eq!(rig(&mut b)["context"], "animate");

    // ACS-41: bind the mesh; the rig goes to Form.
    run(
        &mut b,
        "operandi.acs.bind",
        json!({"rig": root, "items": mesh.to_string(), "type": "distance"}),
    );
    let r = rig(&mut b);
    assert_eq!(r["context"], "form");
    assert_eq!(r["boundMeshes"], json!([mesh]));
}

#[test]
fn acs_refusals_through_the_bridge() {
    let mut b = Bridge::new();
    let e = call(
        &mut b,
        "command_run",
        json!({"command": "operandi.acs.build", "args": {}}),
    )
    .unwrap_err();
    assert!(e.contains("Add a Biped rig first"), "{e}");
    run(&mut b, "operandi.acs.rigAdd", json!({}));
    let e = call(
        &mut b,
        "command_run",
        json!({"command": "operandi.acs.bind", "args": {}}),
    )
    .unwrap_err();
    assert!(e.contains("Build the skeleton first"), "{e}");
}
