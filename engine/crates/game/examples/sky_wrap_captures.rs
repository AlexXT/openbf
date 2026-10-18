//! Ground-level sky captures: four bearings from the map's own ground stand,
//! stitched 2x2, plus the seam instrument that turns "there is a hard line
//! across the sky" into a row number and a size.
//!
//! ```text
//! cargo run --release -p game --example sky_wrap_captures -- \
//!     <out_dir> [assets_dir] [map ...]
//! ```
//!
//! With no map names it captures [`MAPS`], the eight the sky-wrap build asked
//! for. Trailing names override that list — the fog build uses it to capture
//! the maps that GAIN fog (`bf1_bes1`, `bf1_geo1`, `tat1`, ...), which are not
//! the maps a sky-wrap question was about.
//!
//! The stand is [`game::gallery::ground_view`]'s — the same spot the golden
//! gallery's ground panel uses — with the eye at soldier height above it
//! (1.7 m), so these are the view a player has rather than the orbital camera
//! the map sheet used to open with. Each panel looks along one of the four
//! world bearings at a fixed +6 deg pitch, which puts the horizon below the
//! middle of the frame and the dome's bottom ring and the parallax shells in
//! shot. The map's OWN lighting is used (fog on) and the frame clears to
//! [`game::viewer::clear_color`], exactly as the game does.
//!
//! **The seam instrument.** For every panel it reports the largest
//! ROW-TO-ROW jump in mean brightness, over columns 10 %..90 % (the frame's
//! edges catch geometry the camera only clips), together with the row it sits
//! on. Two bands are printed:
//!
//! * `sky45` — the top 45 % of rows, the lead's band in
//!   `artifacts/briefs/brief_sky_wrap.md` D4. At 640x360, fov_y 70 deg and
//!   pitch +6 deg that band is elevation +9.5 deg .. +41 deg, i.e. sky well
//!   above the horizon;
//! * `full` — every row. Added by the build because the V-wrap stripe this
//!   change is about sits at the dome's BOTTOM ring, which on these maps is
//!   at −3 deg .. 0 deg elevation (`artifacts/sky_wrap/dome_uv.txt`) and
//!   therefore lands near row 225 of 360 — outside `sky45` entirely. Both are
//!   printed; neither is tuned.
//!
//! Nothing here renders differently from the game: it is a camera and a
//! measurement over the existing pipeline.

use game::assets::load_scene;
use game::gallery::ground_view;
use game::viewer::{clear_color, upload_scene};
use glam::Vec3;
use render::{Camera, Renderer};
use std::path::{Path, PathBuf};

/// One panel, and the sheet is 2x2 of them.
const PANEL: (u32, u32) = (640, 360);
/// Eye height over the ground stand — a standing soldier's.
const EYE_HEIGHT: f32 = 1.7;
/// Panel pitch, degrees. Positive looks UP; see the module doc.
const PITCH_DEG: f32 = 6.0;
/// The maps the maintainer asked for: both Hoths and both Kaminos (the reported hard
/// line), Naboo (the `sky_water` shell whose v reaches 1.13), both Kashyyyks
/// (domes that stop AT the horizon) and Mustafar (v to 1.077).
pub const MAPS: [&str; 8] = [
    "bf1_hot1", "bf2_hot1", "bf1_kam1", "bf2_kam1", "bf1_nab1", "bf1_kas2", "bf2_kas2", "bf2_mus1",
];

/// Mean brightness (mean of R, G and B, 0..255) of each row, over the middle
/// 80 % of columns.
fn row_means(rgba: &[u8], w: u32, h: u32) -> Vec<f64> {
    let (x0, x1) = (w / 10, w - w / 10);
    let n = f64::from(x1 - x0) * 3.0;
    (0..h)
        .map(|y| {
            let mut sum = 0.0f64;
            for x in x0..x1 {
                let i = ((y * w + x) * 4) as usize;
                sum += f64::from(rgba[i]) + f64::from(rgba[i + 1]) + f64::from(rgba[i + 2]);
            }
            sum / n
        })
        .collect()
}

/// The largest |mean[y+1] − mean[y]| with `y` in `rows`, as (row, jump).
/// The row reported is the UPPER of the pair, so "row 124" means the step
/// happens between 124 and 125.
fn worst_jump(means: &[f64], rows: std::ops::Range<usize>) -> (usize, f64) {
    let mut best = (0usize, 0.0f64);
    for y in rows {
        if y + 1 >= means.len() {
            break;
        }
        let d = (means[y + 1] - means[y]).abs();
        if d > best.1 {
            best = (y, d);
        }
    }
    best
}

/// Four equal panels into one 2x2 sheet: bearings 0 and 90 on the top row,
/// 180 and 270 on the bottom. (`gallery::stitch` is row-wise only.)
fn grid_2x2(panels: &[(Vec<u8>, u32, u32)]) -> (Vec<u8>, u32, u32) {
    let (pw, ph) = (panels[0].1, panels[0].2);
    let (w, h) = (pw * 2, ph * 2);
    let mut out = vec![0u8; (w * h * 4) as usize];
    for (i, (px, _, _)) in panels.iter().enumerate() {
        let (cx, cy) = ((i as u32 % 2) * pw, (i as u32 / 2) * ph);
        for y in 0..ph {
            let src = (y * pw * 4) as usize;
            let dst = (((cy + y) * w + cx) * 4) as usize;
            out[dst..dst + (pw * 4) as usize].copy_from_slice(&px[src..src + (pw * 4) as usize]);
        }
    }
    (out, w, h)
}

fn capture(map_dir: &Path, out: &Path) -> anyhow::Result<()> {
    let mut scene = load_scene(map_dir)?;
    scene.apply_mode_layer("con");
    let scene = scene;
    let name = map_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_owned();

    let (w, h) = PANEL;
    let mut r = Renderer::new_headless(w, h).map_err(|e| anyhow::anyhow!("renderer: {e}"))?;
    let gpu = upload_scene(&mut r, &scene)?;
    let (_, stand) = ground_view(&scene);
    let eye = stand + Vec3::Y * EYE_HEIGHT;
    let clear = clear_color(&scene);

    println!(
        "{name}: stand ({:.1}, {:.1}, {:.1})  eye ({:.1}, {:.1}, {:.1})",
        stand.x, stand.y, stand.z, eye.x, eye.y, eye.z
    );
    let mut panels = Vec::new();
    for k in 0..4u32 {
        let cam = Camera {
            position: eye,
            yaw: (k as f32 * 90.0).to_radians(),
            pitch: PITCH_DEG.to_radians(),
            ..Camera::default()
        };
        r.render(&cam, &gpu.draws, clear)
            .map_err(|e| anyhow::anyhow!("{name} yaw {}: {e}", k * 90))?;
        let (px, pw, ph) = r
            .capture_rgba()
            .map_err(|e| anyhow::anyhow!("capture: {e}"))?;
        let means = row_means(&px, pw, ph);
        let band = (ph as usize) * 45 / 100;
        let (sky_row, sky_jump) = worst_jump(&means, 0..band);
        let (all_row, all_jump) = worst_jump(&means, 0..ph as usize);
        println!(
            "  yaw {:>3}  sky45 row {sky_row:>3} jump {sky_jump:6.2}   full row {all_row:>3} jump {all_jump:6.2}",
            k * 90
        );
        panels.push((px, pw, ph));
    }
    let (sheet, sw, sh) = grid_2x2(&panels);
    std::fs::create_dir_all(out)?;
    std::fs::write(
        out.join(format!("{name}.png")),
        render::encode_png(&sheet, sw, sh).map_err(|e| anyhow::anyhow!("png: {e}"))?,
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let out = PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow::anyhow!("usage: sky_wrap_captures <out_dir> [assets_dir]"))?,
    );
    let assets = args
        .next()
        .map_or_else(game::assets::baked_root, PathBuf::from);
    let maps: Vec<String> = args.collect();
    let maps: Vec<&str> = if maps.is_empty() {
        MAPS.to_vec()
    } else {
        maps.iter().map(String::as_str).collect()
    };
    for map in &maps {
        capture(&assets.join(map), &out)?;
    }
    println!("wrote {} sheets to {}", maps.len(), out.display());
    Ok(())
}
