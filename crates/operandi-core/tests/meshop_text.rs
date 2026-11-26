//! Text as a Mesh Stack op (`prim.text.item`).
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use operandi_core::*;
use serde_json::json;

#[test]
fn text_op_adds_the_text_tools_letters() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "prim.text.item", json!({ "text": "Hi" }));
    let m = ok(&e, id);
    let p = TextParameters {
        text: "Hi".into(),
        ..Default::default()
    };
    let letters = text::text_geometry(&p).unwrap();
    assert!(!letters.faces.is_empty());
    assert_eq!(m.faces.len(), 6 + letters.faces.len());
    assert_eq!(&m.vertices[8..], &letters.vertices[..]);
}

#[test]
fn text_op_with_no_text_adds_nothing() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "prim.text.item", json!({}));
    assert_eq!(ok(&e, id), cube());
}
