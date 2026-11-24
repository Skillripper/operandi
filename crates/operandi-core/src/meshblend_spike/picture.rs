//! Pictures for the B0 comparison page: a small software renderer (flat
//! shaded polygons with their edges drawn on top), so both prototypes are
//! drawn exactly the same way, without a window or graphics card.
//!
//! Surfaces are grey (a slightly different grey per source), strips cyan,
//! corner patches pink, open edges red.
use crate::Mesh;
use glam::DVec3;
use std::collections::HashMap;

/// An orthographic camera: what it looks at and how much it shows.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub centre: DVec3,
    pub forward: DVec3,
    pub right: DVec3,
    pub up: DVec3,
    /// World units across the picture's height.
    pub height: f64,
}

impl View {
    /// Looking at `mesh` from the front, right and above, with a margin.
    pub fn fit(mesh: &Mesh, from: DVec3) -> Self {
        let forward = (-from).normalize();
        let right = forward.cross(DVec3::Y).normalize();
        let up = right.cross(forward);
        let (lo, hi) = mesh.vertices.iter().fold(
            (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)),
            |(lo, hi), p| (lo.min(*p), hi.max(*p)),
        );
        let centre = (lo + hi) * 0.5;
        let (mut w, mut h) = (0f64, 0f64);
        for p in &mesh.vertices {
            let d = *p - centre;
            w = w.max(d.dot(right).abs());
            h = h.max(d.dot(up).abs());
        }
        Self {
            centre,
            forward,
            right,
            up,
            height: (2. * h).max(2. * w * 0.75) * 1.12,
        }
    }

    /// The same direction, closer in on `at`.
    pub fn zoom(self, at: DVec3, height: f64) -> Self {
        Self {
            centre: at,
            height,
            ..self
        }
    }
}

const BACKGROUND: [f64; 3] = [38., 40., 44.];
const STRIP: [f64; 3] = [95., 200., 228.];
const CORNER: [f64; 3] = [232., 140., 196.];
const SOURCES: [[f64; 3]; 4] = [
    [196., 196., 200.],
    [214., 204., 188.],
    [186., 200., 214.],
    [204., 214., 190.],
];

/// Draw `mesh` (per polygon: `kind` 0 surface / 1 strip / 2 corner and
/// its source) at `w × h` pixels. Returns RGB bytes.
pub fn render(mesh: &Mesh, kind: &[u8], source: &[u32], view: View, w: usize, h: usize) -> Vec<u8> {
    const SS: usize = 3;
    let (sw, sh) = (w * SS, h * SS);
    let scale = sh as f64 / view.height;
    let project = |p: DVec3| {
        let d = p - view.centre;
        (
            sw as f64 * 0.5 + d.dot(view.right) * scale,
            sh as f64 * 0.5 - d.dot(view.up) * scale,
            d.dot(view.forward),
        )
    };
    let mut z = vec![f64::INFINITY; sw * sh];
    let mut rgb = vec![BACKGROUND; sw * sh];
    let light = (-view.forward * 0.7 + view.up * 0.5 - view.right * 0.35).normalize();
    let screen: Vec<(f64, f64, f64)> = mesh.vertices.iter().map(|&p| project(p)).collect();
    for (f, face) in mesh.faces.iter().enumerate() {
        let v = &face.vertices;
        if v.len() < 3 {
            continue;
        }
        let p: Vec<DVec3> = v.iter().map(|&i| mesh.vertices[i as usize]).collect();
        let mut n = DVec3::ZERO;
        for i in 0..p.len() {
            n += p[i].cross(p[(i + 1) % p.len()]);
        }
        let n = n.normalize_or_zero();
        let base = match kind.get(f).copied().unwrap_or(0) {
            1 => STRIP,
            2 => CORNER,
            _ => SOURCES[source.get(f).copied().unwrap_or(0) as usize % SOURCES.len()],
        };
        let facing = -n.dot(view.forward);
        let colour = if facing < 0. {
            // Seen from behind: dark red, so a flipped polygon shows.
            [120., 30., 30.]
        } else {
            let shade = 0.38 + 0.62 * n.dot(light).max(0.) * 0.85 + 0.15 * facing;
            base.map(|c| (c * shade).min(255.))
        };
        for k in 1..v.len() - 1 {
            let t = [
                screen[v[0] as usize],
                screen[v[k] as usize],
                screen[v[k + 1] as usize],
            ];
            fill(&mut z, &mut rgb, sw, sh, t, colour);
        }
    }
    // Edges on top, where visible.
    let bias = view.height * 0.004;
    let mut uses: HashMap<(u32, u32), u32> = HashMap::new();
    for face in mesh.faces.iter() {
        let v = &face.vertices;
        for i in 0..v.len() {
            *uses
                .entry(crate::topology::edge_key(v[i], v[(i + 1) % v.len()]))
                .or_default() += 1;
        }
    }
    let mut edges: Vec<_> = uses.into_iter().collect();
    edges.sort();
    for ((a, b), n) in edges {
        let (open, colour, thick) = if n != 2 {
            (true, [255., 40., 40.], 3)
        } else {
            (false, [24., 26., 30.], 1)
        };
        line(
            &z,
            &mut rgb,
            sw,
            sh,
            screen[a as usize],
            screen[b as usize],
            colour,
            if open { 1e9 } else { bias },
            thick,
            if open { 1. } else { 0.55 },
        );
    }
    // Downsample.
    let mut out = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let mut s = [0.; 3];
            for dy in 0..SS {
                for dx in 0..SS {
                    let c = rgb[(y * SS + dy) * sw + x * SS + dx];
                    for k in 0..3 {
                        s[k] += c[k];
                    }
                }
            }
            for c in s {
                out.push((c / (SS * SS) as f64).round().clamp(0., 255.) as u8);
            }
        }
    }
    out
}

fn fill(
    z: &mut [f64],
    rgb: &mut [[f64; 3]],
    w: usize,
    h: usize,
    t: [(f64, f64, f64); 3],
    colour: [f64; 3],
) {
    let (x0, y0) = (
        t.iter()
            .map(|p| p.0)
            .fold(f64::INFINITY, f64::min)
            .floor()
            .max(0.) as usize,
        t.iter()
            .map(|p| p.1)
            .fold(f64::INFINITY, f64::min)
            .floor()
            .max(0.) as usize,
    );
    let (x1, y1) = (
        (t.iter()
            .map(|p| p.0)
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as usize)
            .min(w - 1),
        (t.iter()
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as usize)
            .min(h - 1),
    );
    let area = (t[1].0 - t[0].0) * (t[2].1 - t[0].1) - (t[2].0 - t[0].0) * (t[1].1 - t[0].1);
    if area.abs() < 1e-12 {
        return;
    }
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
            let e = |a: (f64, f64, f64), b: (f64, f64, f64)| {
                ((b.0 - a.0) * (py - a.1) - (px - a.0) * (b.1 - a.1)) / area
            };
            let (w0, w1, w2) = (e(t[1], t[2]), e(t[2], t[0]), e(t[0], t[1]));
            if w0 < 0. || w1 < 0. || w2 < 0. {
                continue;
            }
            let d = w0 * t[0].2 + w1 * t[1].2 + w2 * t[2].2;
            let i = y * w + x;
            if d < z[i] {
                z[i] = d;
                rgb[i] = colour;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn line(
    z: &[f64],
    rgb: &mut [[f64; 3]],
    w: usize,
    h: usize,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    colour: [f64; 3],
    bias: f64,
    thick: i64,
    strength: f64,
) {
    let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let steps = (len.ceil() as usize).max(1);
    for s in 0..=steps {
        let t = s as f64 / steps as f64;
        let (x, y, d) = (
            a.0 + (b.0 - a.0) * t,
            a.1 + (b.1 - a.1) * t,
            a.2 + (b.2 - a.2) * t,
        );
        for oy in -(thick / 2)..=(thick / 2) {
            for ox in -(thick / 2)..=(thick / 2) {
                let (xi, yi) = (x as i64 + ox, y as i64 + oy);
                if xi < 0 || yi < 0 || xi >= w as i64 || yi >= h as i64 {
                    continue;
                }
                let i = yi as usize * w + xi as usize;
                if d <= z[i] + bias {
                    for k in 0..3 {
                        rgb[i][k] = rgb[i][k] * (1. - strength) + colour[k] * strength;
                    }
                }
            }
        }
    }
}

/// Write RGB bytes as a PNG.
pub fn save_png(path: &std::path::Path, w: usize, h: usize, rgb: &[u8]) -> crate::Result<()> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgb).map_err(|e| e.to_string())
}
