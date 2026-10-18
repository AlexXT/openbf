//! Which sky shell segments the V clamp catches, on every baked map dir, and
//! what the clamp costs in texture uploads.
//!
//! ```text
//! cargo run --release -p game --example sky_clamp_census [assets_dir]
//! ```
//!
//! It uploads each scene through the REAL path
//! ([`game::viewer::upload_scene`], headless) and prints that map's own
//! census — [`game::viewer::SceneGpu::sky_clamped`] and
//! [`game::viewer::SceneGpu::sky_clamp_extra_uploads`] — rather than
//! re-deriving the decision, so the table cannot drift from what renders.
//! This is the "what moved it" column of the golden sheets in
//! `engine/artifacts/sky_wrap/`.
//!
//! The rule itself is [`game::viewer::sky_wrap_v`] and its evidence is
//! `engine/artifacts/sky_wrap/dome_uv.txt` — PROVISIONAL, ours; the PS2's own
//! wrap mode is HOLD FOR FABLE.

use game::assets::{discover_maps, load_scene};
use game::viewer::upload_scene;
use render::Renderer;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let assets = std::env::args()
        .nth(1)
        .map_or_else(game::assets::baked_root, PathBuf::from);
    let mut dirs: Vec<PathBuf> = discover_maps(&assets)
        .into_iter()
        .map(|(_, _, d)| d)
        .collect();
    dirs.sort();

    let (mut segments, mut extra, mut maps_with_clamp) = (0usize, 0usize, 0usize);
    for dir in &dirs {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_owned();
        let mut scene = load_scene(dir)?;
        scene.apply_mode_layer("con");
        let scene = scene;
        let mut r = Renderer::new_headless(64, 64).map_err(|e| anyhow::anyhow!("renderer: {e}"))?;
        let gpu = upload_scene(&mut r, &scene)?;
        println!(
            "{name}: {} clamped segment(s), {} extra upload(s)",
            gpu.sky_clamped.len(),
            gpu.sky_clamp_extra_uploads
        );
        for line in &gpu.sky_clamped {
            println!("    {line}");
        }
        segments += gpu.sky_clamped.len();
        extra += gpu.sky_clamp_extra_uploads;
        maps_with_clamp += usize::from(!gpu.sky_clamped.is_empty());
    }
    println!(
        "\n{segments} clamped segments over {maps_with_clamp} of {} map dirs; \
         {extra} extra texture uploads in total",
        dirs.len()
    );
    Ok(())
}
