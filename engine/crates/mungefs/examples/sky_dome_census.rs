//! Census of every shipped sky dome: the elevation band each `DomeModel`
//! mesh covers, and the `DomeInfo` values that scale it.
//!
//! ```text
//! sky_dome_census <map.lvl> [map.lvl ...]
//! ```
//!
//! Answers "is there anything below the horizon on this map?" from the disc
//! rather than from a screenshot. Elevation is measured the way the engine
//! draws it: the first resolving dome sets a scale so its `max(|x|, |z|)` is
//! 1000 m (`game::viewer::scene::SKY_SHELL_RADIUS`), every shell shares that scale,
//! and a shell's `Offset` lifts it. See `analysis/re-sky-lighting.md` §6.

use mungefs::model::Model;
use mungefs::sky::Sky;
use std::collections::HashMap;

/// Radius the first dome is scaled to — must match `viewer::scene::SKY_SHELL_RADIUS`.
const SKY_SHELL_RADIUS: f32 = 1000.0;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: sky_dome_census <map.lvl> [map.lvl ...]");
        std::process::exit(2);
    }
    for path in &args {
        if let Err(e) = census(path) {
            eprintln!("{path}: {e}");
        }
    }
}

fn census(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let buf = std::fs::read(path)?;
    let chunks = mungefs::content_chunks(mungefs::open(&buf)?)?;
    let mut sky = None;
    let mut models: HashMap<String, Model> = HashMap::new();
    for c in &chunks {
        if &c.tag == b"sky_" {
            sky = Sky::parse(c).ok();
        } else if &c.tag == b"modl" {
            if let Ok(m) = Model::parse(c) {
                models.insert(m.name.to_ascii_lowercase(), m);
            }
        }
    }
    let label = std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let Some(sky) = sky else {
        println!("{label}\tno sky_ chunk");
        return Ok(());
    };
    let Some(dome) = sky.dome.as_ref() else {
        println!("{label}\tno DomeInfo");
        return Ok(());
    };
    println!(
        "{label}\tAmbient={:?} Intensity={:?} FogColor={:?} FogRange={:?}",
        dome.ambient, dome.intensity, sky.info.fog_color, sky.info.fog_range
    );
    let mut scale: Option<f32> = None;
    for m in &dome.models {
        let g = m.geometry.to_ascii_lowercase();
        if g.is_empty() {
            println!("  (placeholder entry with an empty Geometry)");
            continue;
        }
        let Some(model) = models.get(&g) else {
            println!("  {g}\tnot a modl in this LVL");
            continue;
        };
        let pts: Vec<[f32; 3]> = model
            .segments
            .iter()
            .flat_map(|s| s.positions.iter().copied())
            .collect();
        if pts.is_empty() {
            println!("  {g}\tno vertices");
            continue;
        }
        let r = pts
            .iter()
            .fold(0.0f32, |a, p| a.max(p[0].abs().max(p[2].abs())));
        let s = *scale.get_or_insert(if r > 1e-3 { SKY_SHELL_RADIUS / r } else { 1.0 });
        let off = m.offset.unwrap_or(0.0);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        let mut below = 0usize;
        for p in &pts {
            let (x, z) = (p[0] * s, p[2] * s);
            let y = p[1] * s + off;
            let el = y.atan2((x * x + z * z).sqrt().max(1e-6)).to_degrees();
            lo = lo.min(el);
            hi = hi.max(el);
            if el < 0.0 {
                below += 1;
            }
        }
        println!(
            "  {g}\tel {lo:.1}..{hi:.1} deg\tbelow horizon {below}/{} ({:.0} %)\toffset {off} movescale {:?}",
            pts.len(),
            100.0 * below as f32 / pts.len() as f32,
            m.movement_scale
        );
    }
    Ok(())
}
