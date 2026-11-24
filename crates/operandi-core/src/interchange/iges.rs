//! IGES 5.3 export: every face as a rational B-spline surface (entity 128),
//! untrimmed. Tab patches are degree-2 Bézier patches (exact, cad.rs);
//! flat polygons are bilinear patches, a convex quad as one and anything
//! else as triangles (a bilinear patch with one side shrunk to a point).
//! IGES here is surfaces only, with no shells or solids: STEP is the
//! format for those.
//!
//! The format is public (the IGES 5.3 specification); nothing here follows
//! another application's files.
use super::cad::{CadFace, CadModel};
use super::step::real;
use crate::Result;
use glam::DVec3;
use std::fmt::Write;

/// A Hollerith string (`5Hhello`), printable ASCII only.
fn hollerith(s: &str) -> String {
    let s: String = s
        .chars()
        .map(|c| if (' '..='~').contains(&c) { c } else { '_' })
        .collect();
    // An empty string is left empty: the field's default.
    if s.is_empty() {
        return s;
    }
    format!("{}H{s}", s.len())
}

/// Fixed-column lines of one section: `width` columns of data, then the
/// section letter and sequence number (and, for P, the entity's directory
/// pointer before them).
struct Section {
    letter: char,
    lines: Vec<String>,
}
impl Section {
    fn push(&mut self, data: &str, pointer: Option<usize>) {
        let n = self.lines.len() + 1;
        let line = match pointer {
            Some(d) => format!("{data:<64} {d:>7}{}{n:>7}", self.letter),
            None => format!("{data:<72}{}{n:>7}", self.letter),
        };
        self.lines.push(line);
    }
    /// `tokens` joined by commas and ended by `;`, wrapped at `width`
    /// without splitting a token (except one longer than a whole line).
    fn stream(&mut self, tokens: &[String], width: usize, pointer: Option<usize>) -> usize {
        let start = self.lines.len();
        let mut line = String::new();
        let last = tokens.len().saturating_sub(1);
        for (i, t) in tokens.iter().enumerate() {
            let mut t = t.clone();
            t.push(if i == last { ';' } else { ',' });
            if line.len() + t.len() > width && !line.is_empty() {
                self.push(&line, pointer);
                line.clear();
            }
            while t.len() > width {
                let rest = t.split_off(width);
                self.push(&t, pointer);
                t = rest;
            }
            line.push_str(&t);
        }
        if !line.is_empty() {
            self.push(&line, pointer);
        }
        self.lines.len() - start
    }
}

/// Entity 128 parameters for a Bézier patch of degree (`du`, `dv`) with
/// `controls[i][j]` (`i` along u).
fn bspline_surface(controls: &[Vec<DVec3>]) -> Vec<String> {
    let (k1, k2) = (controls.len() - 1, controls[0].len() - 1);
    let mut p = vec![
        "128".to_owned(),
        k1.to_string(),
        k2.to_string(),
        k1.to_string(),
        k2.to_string(),
        "0".into(),
        "0".into(),
        "1".into(),
        "0".into(),
        "0".into(),
    ];
    // Bézier knots: degree + 1 zeros, then degree + 1 ones.
    for k in [k1, k2] {
        p.extend((0..=k).map(|_| "0.".to_owned()));
        p.extend((0..=k).map(|_| "1.".to_owned()));
    }
    p.extend((0..(k1 + 1) * (k2 + 1)).map(|_| "1.".to_owned()));
    for j in 0..=k2 {
        for row in controls {
            let c = row[j];
            p.extend([real(c.x), real(c.y), real(c.z)]);
        }
    }
    p.extend(["0.", "1.", "0.", "1."].map(String::from));
    p
}

/// Whether the quad's corners turn the same way at every corner.
fn convex(points: &[DVec3], normal: DVec3) -> bool {
    (0..4).all(|i| {
        let (a, b, c) = (points[i], points[(i + 1) % 4], points[(i + 2) % 4]);
        (b - a).cross(c - b).dot(normal) > 0.0
    })
}

/// The IGES file of `model` (coordinates already in millimetres, Z up).
pub(super) fn write(model: &CadModel, file_name: &str, stamp: &str) -> Result<Vec<u8>> {
    let info = super::file_info();
    let mut surfaces: Vec<Vec<String>> = Vec::new();
    let mut max = 0.0f64;
    let mut track = |p: DVec3| max = max.max(p.abs().max_element());
    for shell in &model.shells {
        for face in &shell.faces {
            match face {
                CadFace::Patch(p) => {
                    p.grid.iter().flatten().for_each(|&c| track(c));
                    let controls: Vec<Vec<DVec3>> = p.grid.iter().map(|r| r.to_vec()).collect();
                    surfaces.push(bspline_surface(&controls));
                }
                CadFace::Polygon { points, normal, .. } => {
                    points.iter().for_each(|&c| track(c));
                    // A bilinear patch c[i][j]: corners 0, 1 along u, 3, 2
                    // above them, so the normal matches the polygon's.
                    let quad = |a: DVec3, b: DVec3, c: DVec3, d: DVec3| {
                        bspline_surface(&[vec![a, d], vec![b, c]])
                    };
                    if points.len() == 4 && convex(points, *normal) {
                        surfaces.push(quad(points[0], points[1], points[2], points[3]));
                    } else {
                        for i in 1..points.len() - 1 {
                            let (a, b, c) = (points[0], points[i], points[i + 1]);
                            surfaces.push(quad(a, b, c, c));
                        }
                    }
                }
            }
        }
    }
    let stamp = stamp.replace(['-', ':'], "").replace('T', ".");
    let mut start = Section {
        letter: 'S',
        lines: Vec::new(),
    };
    start.push(
        &format!(
            "{}: exact subdivision patches as B-spline surfaces",
            model.name
        )
        .chars()
        .filter(|c| (' '..='~').contains(c))
        .take(72)
        .collect::<String>(),
        None,
    );
    let mut global = Section {
        letter: 'G',
        lines: Vec::new(),
    };
    let g = [
        "1H,".to_owned(),
        "1H;".into(),
        hollerith(&model.name),
        hollerith(file_name),
        hollerith("Operandi"),
        hollerith(env!("CARGO_PKG_VERSION")),
        "32".into(),
        "38".into(),
        "6".into(),
        "308".into(),
        "15".into(),
        hollerith(&model.name),
        "1.".into(),
        "2".into(),
        "2HMM".into(),
        "1".into(),
        "1.".into(),
        hollerith(&stamp),
        real(model.tolerance),
        real(max.max(1.0)),
        hollerith(&info.author),
        hollerith(&info.copyright),
        "11".into(),
        "0".into(),
        hollerith(&stamp),
    ];
    global.stream(&g, 72, None);
    let mut directory = Section {
        letter: 'D',
        lines: Vec::new(),
    };
    let mut parameters = Section {
        letter: 'P',
        lines: Vec::new(),
    };
    for surface in &surfaces {
        let d = directory.lines.len() + 1;
        let first = parameters.lines.len() + 1;
        let count = parameters.stream(surface, 64, Some(d));
        let mut a = String::new();
        let _ = write!(
            a,
            "{:>8}{first:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}",
            128, 0, 0, 0, 0, 0, 0, "00000000"
        );
        directory.push(&a, None);
        let mut b = String::new();
        let _ = write!(
            b,
            "{:>8}{:>8}{:>8}{count:>8}{:>8}{:>8}{:>8}{:>8}{:>8}",
            128, 0, 0, 0, "", "", "", 0
        );
        directory.push(&b, None);
    }
    let mut out = String::new();
    for s in [&start, &global, &directory, &parameters] {
        for line in &s.lines {
            out.push_str(line);
            out.push('\n');
        }
    }
    let end = format!(
        "S{:>7}G{:>7}D{:>7}P{:>7}",
        start.lines.len(),
        global.lines.len(),
        directory.lines.len(),
        parameters.lines.len()
    );
    let _ = writeln!(out, "{end:<72}T{:>7}", 1);
    Ok(out.into_bytes())
}
