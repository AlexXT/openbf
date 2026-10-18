//! Headless GPU referee for the DomeInfo frame filter (`render::FrameFilter`,
//! card BFLT, `analysis/re-glow.md` §3 `[O]`): the pass's output against the
//! model the PS2 was measured with, applied on the CPU to the same frame
//! rendered without it — `out = min(255, v + I/128 · max(0, box(v) − T))`
//! per channel in display bytes, the box `box_side(height)` pixels wide
//! starting one pixel up-left for a side of 2.
//!
//! Ignored by default — needs a real adapter. Run them through
//! `scripts/gpu_tests.sh` (serialised).

use glam::Mat4;
use render::{box_side, Camera, DrawCmd, FrameFilter, Renderer};

use crate::common;
use common::{upload_tex, view_quad};

/// 448 lines: one PS2 half-scale pixel is a 2×2 box, as on the PS2.
const W: u32 = 96;
const H: u32 = 448;

#[test]
#[ignore = "requires a GPU adapter (run with --ignored on the NVIDIA box)"]
fn the_pass_matches_the_ps2_model() {
    let mut r = Renderer::new_headless(W, H).expect("headless renderer");
    // A 16×16 noise sheet over the whole view: every level from dark to
    // bright, edges everywhere, so the box and the threshold both matter.
    let mut seed = 12345u32;
    let texels: Vec<u8> = (0..16 * 16 * 4)
        .map(|i| {
            if i % 4 == 3 {
                return 255;
            }
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (seed >> 16) as u8
        })
        .collect();
    let tex = upload_tex(&mut r, 16, 16, &texels, "noise");
    let quad = view_quad(&mut r, -3.0);
    let draws = [DrawCmd::textured(quad, tex, Mat4::IDENTITY)];
    let cam = Camera::default();

    r.render(&cam, &draws, [0.0, 0.0, 0.0, 1.0])
        .expect("render");
    let (off, w, h) = r.capture_rgba().expect("capture");
    let f = FrameFilter {
        threshold: 50.0,
        intensity: 50.0,
    };
    r.set_frame_filter(Some(f));
    r.render(&cam, &draws, [0.0, 0.0, 0.0, 1.0])
        .expect("render");
    let (on, _, _) = r.capture_rgba().expect("capture");

    let n = box_side(h) as i32;
    assert_eq!(n, 2);
    let at = |x: i32, y: i32, c: usize| {
        let x = x.clamp(0, w as i32 - 1) as u32;
        let y = y.clamp(0, h as i32 - 1) as u32;
        f32::from(off[((y * w + x) * 4) as usize + c])
    };
    let (mut worst, mut lifted) = (0.0f32, 0usize);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            for c in 0..3 {
                let mut sum = 0.0;
                for dy in 0..n {
                    for dx in 0..n {
                        sum += at(x - n / 2 + dx, y - n / 2 + dy, c);
                    }
                }
                let low = sum / (n * n) as f32;
                let v = at(x, y, c);
                let want = (v + f.intensity / 128.0 * (low - f.threshold).max(0.0)).min(255.0);
                let got = f32::from(on[((y as u32 * w + x as u32) * 4) as usize + c]);
                worst = worst.max((got - want).abs());
                lifted += usize::from(got > v);
            }
        }
    }
    assert!(worst <= 2.0, "the pass is {worst} levels off the model");
    assert!(
        lifted > (w * h) as usize,
        "the filter lifted only {lifted} channels"
    );

    // Off again: the frame is back to the unfiltered bytes exactly.
    r.set_frame_filter(None);
    r.render(&cam, &draws, [0.0, 0.0, 0.0, 1.0])
        .expect("render");
    assert_eq!(r.capture_rgba().expect("capture").0, off);
}
