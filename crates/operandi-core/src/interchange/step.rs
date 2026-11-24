//! STEP (ISO 10303-21, AP214 "automotive design" schema) export: each
//! visible mesh as a boundary representation of exact surfaces (cad.rs).
//!
//! Tab patches become `B_SPLINE_SURFACE_WITH_KNOTS` faces (degree 2, one
//! span each: a Bézier patch), bounded by the quadratic `B_SPLINE_CURVE`
//! boundaries they share with their neighbours. Flat polygons become
//! `PLANE` faces bounded by `LINE` edges (non-planar ones are split into
//! triangles first). Faces that share edges form one shell; a shell where
//! every edge has two faces is closed and written as a solid
//! (`MANIFOLD_SOLID_BREP`), any other as a surface model.
//!
//! The format is public (ISO 10303-21 and the AP214 schema); nothing here
//! follows another application's files.
use super::cad::{CadFace, CadModel};
use crate::Result;
use glam::DVec3;
use std::collections::HashMap;
use std::fmt::Write;

/// A vertex: its shell's numbering space and its number there.
type VertexKey = (usize, u32);

/// Entity writer: numbers entities as they are added. STEP allows
/// references to entities written later, so each piece is written once.
struct Out {
    text: String,
    next: u64,
    points: HashMap<[u64; 3], u64>,
}
impl Out {
    fn add(&mut self, body: impl AsRef<str>) -> u64 {
        let id = self.next;
        self.next += 1;
        let _ = writeln!(self.text, "#{id}={};", body.as_ref());
        id
    }
    /// A `CARTESIAN_POINT`, shared by every user of the same position.
    fn point(&mut self, p: DVec3) -> u64 {
        let key = [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()];
        if let Some(&id) = self.points.get(&key) {
            return id;
        }
        let id = self.add(format!("CARTESIAN_POINT('',{})", triple(p)));
        self.points.insert(key, id);
        id
    }
    fn direction(&mut self, d: DVec3) -> u64 {
        self.add(format!("DIRECTION('',{})", triple(d)))
    }
}

/// A STEP real: always with a decimal point, exponent as `E`.
pub(super) fn real(v: f64) -> String {
    let v = if v == 0.0 { 0.0 } else { v };
    let s = format!("{v:?}");
    match s.split_once('e') {
        Some((m, e)) if m.contains('.') => format!("{m}E{e}"),
        Some((m, e)) => format!("{m}.E{e}"),
        None => s,
    }
}

fn triple(p: DVec3) -> String {
    format!("({},{},{})", real(p.x), real(p.y), real(p.z))
}

/// A STEP string: quotes doubled, backslashes doubled, anything outside
/// printable ASCII as `\X2\` UTF-16 hex.
pub(super) fn text(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\'' => out.push_str("''"),
            '\\' => out.push_str("\\\\"),
            ' '..='~' => out.push(c),
            _ => {
                out.push_str("\\X2\\");
                let mut units = [0u16; 2];
                for u in c.encode_utf16(&mut units) {
                    let _ = write!(out, "{u:04X}");
                }
                out.push_str("\\X0\\");
            }
        }
    }
    out.push('\'');
    out
}

/// The STEP file of `model` (coordinates already in millimetres, Z up).
pub(super) fn write(model: &CadModel, file_name: &str, stamp: &str) -> Result<Vec<u8>> {
    let info = super::file_info();
    let mut out = Out {
        text: String::new(),
        next: 1,
        points: HashMap::new(),
    };
    // Product structure and units (AP214's minimum for a part).
    let app = out.add("APPLICATION_CONTEXT('automotive design')");
    out.add(format!(
        "APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#{app})"
    ));
    let product_context = out.add(format!("PRODUCT_CONTEXT('',#{app},'mechanical')"));
    let definition_context = out.add(format!(
        "PRODUCT_DEFINITION_CONTEXT('part definition',#{app},'design')"
    ));
    let name = text(&model.name);
    let product = out.add(format!("PRODUCT({name},{name},'',(#{product_context}))"));
    out.add(format!(
        "PRODUCT_RELATED_PRODUCT_CATEGORY('part',$,(#{product}))"
    ));
    let formation = out.add(format!("PRODUCT_DEFINITION_FORMATION('','',#{product})"));
    let definition = out.add(format!(
        "PRODUCT_DEFINITION('design','',#{formation},#{definition_context})"
    ));
    let shape = out.add(format!("PRODUCT_DEFINITION_SHAPE('','',#{definition})"));
    let mm = out.add("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.))");
    let radian = out.add("(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.))");
    let steradian = out.add("(NAMED_UNIT(*)SI_UNIT($,.STERADIAN.)SOLID_ANGLE_UNIT())");
    let accuracy = out.add(format!(
        "UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE({}),#{mm},'distance_accuracy_value','confusion accuracy')",
        real(model.tolerance)
    ));
    let context = out.add(format!(
        "(GEOMETRIC_REPRESENTATION_CONTEXT(3)GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{accuracy}))GLOBAL_UNIT_ASSIGNED_CONTEXT((#{mm},#{radian},#{steradian}))REPRESENTATION_CONTEXT('Context3D','3D Context'))"
    ));
    let origin = out.point(DVec3::ZERO);
    let z = out.direction(DVec3::Z);
    let x = out.direction(DVec3::X);
    let axis = out.add(format!("AXIS2_PLACEMENT_3D('',#{origin},#{z},#{x})"));

    let mut solids = Vec::new();
    let mut surfaces = Vec::new();
    let mut vertices: HashMap<VertexKey, u64> = HashMap::new();
    // Edge by its (low, high) vertex keys: entity, written low → high.
    let mut edges: HashMap<(VertexKey, VertexKey), u64> = HashMap::new();
    for shell in &model.shells {
        let mut faces = Vec::with_capacity(shell.faces.len());
        for face in &shell.faces {
            // Each boundary edge as (from, to, curve controls from → to).
            let (surface, sides) = match face {
                CadFace::Patch(p) => {
                    let rows: Vec<String> = p
                        .grid
                        .iter()
                        .map(|row| {
                            let ids: Vec<String> =
                                row.iter().map(|&c| format!("#{}", out.point(c))).collect();
                            format!("({})", ids.join(","))
                        })
                        .collect();
                    let surface = out.add(format!(
                        "B_SPLINE_SURFACE_WITH_KNOTS('',2,2,({}),.UNSPECIFIED.,.F.,.F.,.F.,(3,3),(3,3),(0.,1.),(0.,1.),.UNSPECIFIED.)",
                        rows.join(",")
                    ));
                    let sides: Vec<_> = (0..4)
                        .map(|i| {
                            (
                                (shell.space, p.corners[i]),
                                (shell.space, p.corners[(i + 1) % 4]),
                                p.side(i).to_vec(),
                            )
                        })
                        .collect();
                    (surface, sides)
                }
                CadFace::Polygon {
                    points,
                    keys,
                    normal,
                } => {
                    let o = out.point(points[0]);
                    let n = out.direction(*normal);
                    let along = (points[1] - points[0]).reject_from_normalized(*normal);
                    let r = out.direction(along.normalize_or(normal.any_orthonormal_vector()));
                    let placement = out.add(format!("AXIS2_PLACEMENT_3D('',#{o},#{n},#{r})"));
                    let surface = out.add(format!("PLANE('',#{placement})"));
                    let k = points.len();
                    let sides: Vec<_> = (0..k)
                        .map(|i| {
                            (
                                (shell.space, keys[i]),
                                (shell.space, keys[(i + 1) % k]),
                                vec![points[i], points[(i + 1) % k]],
                            )
                        })
                        .collect();
                    (surface, sides)
                }
            };
            let mut oriented = Vec::with_capacity(sides.len());
            for (from, to, controls) in sides {
                let forward = from < to;
                let (lo, hi) = if forward { (from, to) } else { (to, from) };
                let edge = match edges.get(&(lo, hi)) {
                    Some(&e) => e,
                    None => {
                        let mut controls = controls;
                        if !forward {
                            controls.reverse();
                        }
                        let mut vertex = |key, p: DVec3, out: &mut Out| {
                            *vertices.entry(key).or_insert_with(|| {
                                let pt = out.point(p);
                                out.add(format!("VERTEX_POINT('',#{pt})"))
                            })
                        };
                        let start = vertex(lo, controls[0], &mut out);
                        let end = vertex(hi, *controls.last().unwrap(), &mut out);
                        let curve = if controls.len() == 2 {
                            let p = out.point(controls[0]);
                            let d = controls[1] - controls[0];
                            let dir = out.direction(d.normalize_or_zero());
                            let v = out.add(format!("VECTOR('',#{dir},{})", real(d.length())));
                            out.add(format!("LINE('',#{p},#{v})"))
                        } else {
                            let ids: Vec<String> = controls
                                .iter()
                                .map(|&c| format!("#{}", out.point(c)))
                                .collect();
                            out.add(format!(
                                "B_SPLINE_CURVE_WITH_KNOTS('',2,({}),.UNSPECIFIED.,.F.,.F.,(3,3),(0.,1.),.UNSPECIFIED.)",
                                ids.join(",")
                            ))
                        };
                        let e = out.add(format!("EDGE_CURVE('',#{start},#{end},#{curve},.T.)"));
                        edges.insert((lo, hi), e);
                        e
                    }
                };
                let sense = if forward { ".T." } else { ".F." };
                oriented.push(format!(
                    "#{}",
                    out.add(format!("ORIENTED_EDGE('',*,*,#{edge},{sense})"))
                ));
            }
            let lp = out.add(format!("EDGE_LOOP('',({}))", oriented.join(",")));
            let bound = out.add(format!("FACE_OUTER_BOUND('',#{lp},.T.)"));
            faces.push(format!(
                "#{}",
                out.add(format!("ADVANCED_FACE('',(#{bound}),#{surface},.T.)"))
            ));
        }
        let name = text(&shell.name);
        if shell.closed {
            let s = out.add(format!("CLOSED_SHELL('',({}))", faces.join(",")));
            solids.push(format!(
                "#{}",
                out.add(format!("MANIFOLD_SOLID_BREP({name},#{s})"))
            ));
        } else {
            let s = out.add(format!("OPEN_SHELL('',({}))", faces.join(",")));
            surfaces.push(format!(
                "#{}",
                out.add(format!("SHELL_BASED_SURFACE_MODEL({name},(#{s}))"))
            ));
        }
    }
    // Solids go in an advanced B-rep representation and open surfaces in a
    // surface one; with both, the surfaces hang off the solids' one.
    let representation = |out: &mut Out, kind: &str, items: &[String]| {
        let mut list = vec![format!("#{axis}")];
        list.extend(items.iter().cloned());
        out.add(format!("{kind}({name},({}),#{context})", list.join(",")))
    };
    let main = match (solids.is_empty(), surfaces.is_empty()) {
        (false, true) => representation(&mut out, "ADVANCED_BREP_SHAPE_REPRESENTATION", &solids),
        (true, _) => representation(&mut out, "MANIFOLD_SURFACE_SHAPE_REPRESENTATION", &surfaces),
        (false, false) => {
            let main = representation(&mut out, "ADVANCED_BREP_SHAPE_REPRESENTATION", &solids);
            let more = representation(&mut out, "MANIFOLD_SURFACE_SHAPE_REPRESENTATION", &surfaces);
            out.add(format!(
                "SHAPE_REPRESENTATION_RELATIONSHIP('','',#{main},#{more})"
            ));
            main
        }
    };
    out.add(format!("SHAPE_DEFINITION_REPRESENTATION(#{shape},#{main})"));

    let author = text(&info.author);
    let mut file = String::new();
    file.push_str("ISO-10303-21;\nHEADER;\n");
    let _ = writeln!(
        file,
        "FILE_DESCRIPTION(({}),'2;1');",
        text("Operandi export: exact subdivision patches and flat polygons")
    );
    let _ = writeln!(
        file,
        "FILE_NAME({},{},({author}),({}),'Operandi','Operandi','');",
        text(file_name),
        text(stamp),
        text(&info.copyright),
    );
    file.push_str(
        "FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));\nENDSEC;\nDATA;\n",
    );
    file.push_str(&out.text);
    file.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
    Ok(file.into_bytes())
}
