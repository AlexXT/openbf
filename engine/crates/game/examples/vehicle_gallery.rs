//! Contact sheet of one side's baked vehicles (headless, card VBAKE):
//! `cargo run -p game --example vehicle_gallery -- <side_bfN/vehicles/<side>> <out.png> [--cell px]`
//!
//! One cell per geometry of `vehicles.ron`, in manifest order — each vehicle's
//! body and then its destruction chunks — drawn from its best LOD at a
//! three-quarter view framed on the model's bounds, textured from the side's
//! own `textures/`. The index (cell → geometry) is printed to stdout.

use glam::{Mat4, Vec3};
use render::{DrawCmd, Renderer};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dir = Path::new(&args[1]);
    let out = &args[2];
    let cell: u32 = args
        .iter()
        .position(|a| a == "--cell")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(320);
    let text = std::fs::read_to_string(dir.join("vehicles.ron"))?;
    let m: manifest::VehicleManifest = ron::from_str(&text)?;
    let (w, h) = (cell, cell * 3 / 4);
    let cols = 6u32;
    let rows = (m.geometries.len() as u32).div_ceil(cols);
    // The clear colour everywhere, so an empty trailing cell reads as empty.
    let mut sheet: Vec<u8> = [111u8, 118, 133, 255].repeat((w * cols * h * rows) as usize);
    let mut r = Renderer::new_headless(w, h)?;
    let tex_dir = dir.join("textures");
    for (i, g) in m.geometries.iter().enumerate() {
        let cpu = game::assets::load_glb(&dir.join("models").join(format!("{}.glb", g.model)))?;
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for p in &cpu.positions {
            lo = lo.min(Vec3::from(*p));
            hi = hi.max(Vec3::from(*p));
        }
        let (c, radius) = ((lo + hi) * 0.5, ((hi - lo).length() * 0.5).max(0.1));
        let dir3 = Vec3::new(0.62, 0.42, 0.66).normalize();
        // Far enough that the bounding sphere just fits the vertical FOV.
        let fov = render::Camera::default().fov_y;
        let dist = radius / (fov * 0.5).sin() * 1.05;
        let mut cam = game::viewer::look_at(c + dir3 * dist, c);
        cam.near = radius * 0.05;
        cam.far = dist + radius * 2.0;
        let mut draws = Vec::new();
        for (mesh, mat, layer, _) in game::viewer::upload_submeshes(&mut r, &cpu)? {
            let mut cmd = DrawCmd::flat(mesh, [0.8, 0.8, 0.8, 1.0], Mat4::IDENTITY);
            cmd.layer = layer;
            if let Some(name) = mat {
                if let Some(t) =
                    game::viewer::upload_texture_file(&mut r, &tex_dir, &name.to_ascii_lowercase())?
                {
                    cmd.texture = Some(t);
                    cmd.color = [1.0; 4];
                } else {
                    // Named but not baked: magenta, so a gap shows.
                    cmd.color = [1.0, 0.0, 1.0, 1.0];
                }
            }
            draws.push(cmd);
        }
        r.render_frame(&cam, &draws, &[], [0.16, 0.18, 0.22, 1.0], None)?;
        let (rgba, pw, ph) = r.capture_rgba()?;
        let (cx, cy) = (i as u32 % cols, i as u32 / cols);
        for y in 0..ph.min(h) {
            let src = &rgba[(y * pw * 4) as usize..((y * pw + pw.min(w)) * 4) as usize];
            let at = (((cy * h + y) * w * cols + cx * w) * 4) as usize;
            sheet[at..at + src.len()].copy_from_slice(src);
        }
        let size = hi - lo;
        println!(
            "{i:3} r{cy} c{cx}  {} ({}) {:.1} x {:.1} x {:.1} m",
            g.name, g.model, size.x, size.y, size.z
        );
    }
    image::save_buffer(out, &sheet, w * cols, h * rows, image::ColorType::Rgba8)?;
    println!("wrote {out}");
    Ok(())
}
