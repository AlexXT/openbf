//! The PS2's capture icon rows through the port's rule (card CPBAR,
//! acceptance A; `analysis/re-capture-display.md` §5, §6, §10).
//!
//! `data/capture_display_ps2.csv` is the RE kit's fixture
//! (`engine/artifacts/lead_tmp/cpbar/fixture/`, made by `tools/mkfixture.py`):
//! 4,338 rows of 8 live scenarios, 3 in BF1 (Yavin 4 Arena) and 5 in BF2
//! (Mygeeto), each the post the player stands in read from RAM (owner, the
//! capturing team, units inside per team, the player's bit in the counting
//! mask, the meter's team, both meters and times) beside the HUD object's
//! own fields (drawn, split, top and bottom colour). Per row
//! `game::capture_feed::icon` under the PS2's rule must give the drawn flag,
//! the split within 0.01 and both colours (an alternating top as either of
//! its pair). The object is one update behind the post, so a row also
//! passes on the previous update's post state, as the kit's `replay.py`
//! does; a row within one update of a change of the post's state is
//! counted apart, not failed. Bar: 99 % per source, every other row named.

use game::capture_feed::{icon, PostState, Rules};
use ui::hud_capture::Side;

struct Row {
    game: String,
    src: String,
    t: f32,
    my: u8,
    owner: u8,
    counts: [u32; 3],
    mask: u32,
    charge: u8,
    neutral: f32,
    capture: f32,
    n: f32,
    cap_t: f32,
    vis: bool,
    frac: f32,
    top: String,
    bottom: String,
}

fn rows() -> Vec<Row> {
    let text = include_str!("data/capture_display_ps2.csv");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap().split(',').collect();
    let col = |name: &str| head.iter().position(|h| *h == name).unwrap();
    let c = [
        "game",
        "src",
        "t",
        "my",
        "owner",
        "c1",
        "c2",
        "c3",
        "mask",
        "charge_team",
        "neutral",
        "capture",
        "N",
        "T",
        "vis",
        "frac",
        "top",
        "bottom",
    ]
    .map(col);
    lines
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let num = |k: usize| f[c[k]].parse::<f32>().unwrap();
            Row {
                game: f[c[0]].into(),
                src: f[c[1]].into(),
                t: num(2),
                my: num(3) as u8,
                owner: num(4) as u8,
                counts: [num(5) as u32, num(6) as u32, num(7) as u32],
                mask: num(8) as u32,
                charge: num(9) as u8,
                neutral: num(10),
                capture: num(11),
                n: num(12),
                cap_t: num(13),
                vis: f[c[14]] == "1",
                frac: num(15),
                top: f[c[16]].into(),
                bottom: f[c[17]].into(),
            }
        })
        .collect()
}

/// The PS2's team table row as the player's team sees it (§3.2 [M]).
fn rgb(game: &str, s: Side) -> &'static str {
    match s {
        Side::Neutral => "c0c0c0",
        Side::Enemy => "c86464",
        Side::Own if game == "bf1" => "64c864",
        Side::Own => "0156d5",
    }
}

/// PS2 team index (0 = nobody) -> the port's.
fn team(t: u8) -> Option<u8> {
    t.checked_sub(1)
}

/// What the port's rule draws for a row's post state: `None`, or
/// (split, the possible tops, bottom).
fn expect(r: &Row) -> Option<(f32, Vec<&'static str>, &'static str)> {
    if r.mask & 1 == 0 {
        return None;
    }
    let post = PostState {
        owner: team(r.owner),
        capture: r.capture,
        capture_team: team(r.charge),
        neutral: r.neutral,
        capture_time: r.cap_t,
        neutralize_time: r.n,
    };
    let viewer = team(r.my).unwrap();
    // The top alternates by the battle clock: both halves of a second.
    let mut tops = Vec::new();
    let mut got = None;
    for clock in [0.1, 0.6] {
        let i = icon(&post, &r.counts, viewer, clock, Rules::PS2)?;
        let top = rgb(&r.game, i.top);
        if !tops.contains(&top) {
            tops.push(top);
        }
        got = Some((i.split, rgb(&r.game, i.bottom)));
    }
    let (split, bottom) = got?;
    Some((split, tops, bottom))
}

fn fits(e: &Option<(f32, Vec<&'static str>, &'static str)>, r: &Row) -> bool {
    match e {
        None => !r.vis,
        Some((split, tops, bottom)) => {
            r.vis
                && (split - r.frac).abs() <= 0.01
                && tops.contains(&r.top.as_str())
                && *bottom == r.bottom
        }
    }
}

#[test]
fn the_ps2_rows_through_the_ports_rule() {
    let rows = rows();
    assert_eq!(rows.len(), 4338);
    let mut by: Vec<(String, [usize; 4], Vec<String>)> = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        // The previous update's row: the last one of this source with an
        // earlier clock (rows can share one).
        let mut j = i;
        while j > 0 && rows[j - 1].src == r.src && rows[j - 1].t == r.t {
            j -= 1;
        }
        let prev = (j > 0 && rows[j - 1].src == r.src).then(|| &rows[j - 1]);
        let next = rows.get(i + 1).filter(|n| n.src == r.src);
        let e = expect(r);
        let ok = fits(&e, r);
        let lag = !ok && prev.is_some_and(|p| fits(&expect(p), r));
        let edge = !ok
            && !lag
            && [prev, next]
                .into_iter()
                .flatten()
                .any(|n| (n.t - r.t).abs() <= 0.07 && expect(n) != e);
        if by.last().is_none_or(|b| b.0 != r.src) {
            by.push((r.src.clone(), [0; 4], Vec::new()));
        }
        let b = by.last_mut().unwrap();
        b.1[0] += 1;
        if ok || lag {
            b.1[1] += 1;
        } else if edge {
            b.1[2] += 1;
        } else {
            b.1[3] += 1;
            b.2.push(format!(
                "t {} owner {} counts {:?} mask {} chg {} neu {} cap {} | vis {} frac {} {}/{} | port {:?}",
                r.t, r.owner, r.counts, r.mask, r.charge, r.neutral, r.capture, r.vis, r.frac,
                r.top, r.bottom, e
            ));
        }
    }
    let mut total = [0usize; 4];
    for (src, n, misses) in &by {
        println!(
            "{src:34} {:5} rows: {:5} fit ({:.2} %), at a change {:3}, MISS {}",
            n[0],
            n[1],
            100.0 * n[1] as f64 / n[0] as f64,
            n[2],
            n[3]
        );
        for m in misses {
            println!("    {m}");
        }
        for k in 0..4 {
            total[k] += n[k];
        }
        assert!(
            n[1] as f64 >= 0.99 * n[0] as f64,
            "{src}: {} of {}",
            n[1],
            n[0]
        );
    }
    println!(
        "ALL {} rows: {} fit, {} at a change, {} miss",
        total[0], total[1], total[2], total[3]
    );
    assert!(total[3] <= 3, "{} rows miss", total[3]);
}

mod common;

/// Acceptance C: the icon as the renderer draws it over the TAT1 golden
/// frame. Against the same frame without it, the changed pixels at 640 x 480
/// must be the PS2's box (x 533..603, y 212..272, within 2 px) and the
/// icon's solid pixels the PS2's colours; at 1280 x 720 it keeps its place at
/// the right edge, not stretched. `CPBAR_SHOTS=<dir>` writes the frames.
#[test]
#[ignore = "requires a GPU adapter + baked assets_baked/tat1"]
fn the_icon_on_screen() {
    use game::viewer::{camera_from, upload_scene, GOLDEN_CAM};
    use ui::hud_capture::{CaptureIcon, Mark};
    common::force_ps2_tier();
    let baked = game::assets::baked_root().join("tat1");
    let scene = game::assets::load_scene(&baked).expect("bake TAT1 first");
    // The PS2 strips' moments (sheets/bf2_*.png): an enemy post 0.414 down,
    // his capture at 0.760, his meter contested (the top red), and ours: his
    // own post attacked with the mark.
    let moments = [
        ("enemy_neutralize", 0.414, Side::Neutral, Side::Enemy, None),
        ("neutral_capture", 0.760, Side::Neutral, Side::Own, None),
        ("contest_my_meter", 0.697, Side::Enemy, Side::Own, None),
        (
            "ours_own_post_losing",
            0.300,
            Side::Neutral,
            Side::Own,
            Some(Mark::Down),
        ),
    ];
    let shots = std::env::var_os("CPBAR_SHOTS").map(std::path::PathBuf::from);
    for size in [[640u32, 480u32], [1280, 720]] {
        let mut r = render::Renderer::new_headless(size[0], size[1]).expect("headless renderer");
        let gpu = upload_scene(&mut r, &scene).expect("upload");
        let mut hud = game::hud_feed::HudView::for_renderer(&game::assets::baked_root(), &r)
            .expect("the BF2 HUD bake");
        let mut frame = |icon: Option<CaptureIcon>| {
            let m = ui::hud::HudModel {
                hp: 300.0,
                hp_max: 300.0,
                capture: icon,
                ..Default::default()
            };
            hud.update(&m, 0.0, size);
            hud.update(&m, 0.0, size);
            r.render_with_overlay(
                &camera_from(GOLDEN_CAM),
                &gpu.draws,
                gpu.clear,
                Some(&mut |dev, q, enc, view, sz| hud.paint(dev, q, enc, view, sz)),
            )
            .expect("render");
            r.capture_rgba().expect("capture").0
        };
        let bare = frame(None);
        for (name, split, top, bottom, mark) in moments {
            let px = frame(Some(CaptureIcon {
                split,
                top,
                bottom,
                mark,
            }));
            let (w, h) = (size[0] as usize, size[1] as usize);
            let mut b = [usize::MAX, usize::MAX, 0, 0];
            let mut colours = std::collections::BTreeMap::<[u8; 3], usize>::new();
            for y in 0..h {
                for x in 0..w {
                    let k = 4 * (y * w + x);
                    if px[k..k + 3] != bare[k..k + 3] {
                        b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
                        *colours.entry([px[k], px[k + 1], px[k + 2]]).or_default() += 1;
                    }
                }
            }
            let mut common: Vec<_> = colours.into_iter().collect();
            common.sort_by_key(|c| std::cmp::Reverse(c.1));
            println!(
                "{}x{} {name:22} changed box x {}..{} y {}..{}  colours {:?}",
                size[0],
                size[1],
                b[0],
                b[2] + 1,
                b[1],
                b[3] + 1,
                &common[..3.min(common.len())]
            );
            if let Some(dir) = &shots {
                std::fs::create_dir_all(dir).unwrap();
                let png = render::encode_png(&px, size[0], size[1]).unwrap();
                std::fs::write(dir.join(format!("{name}_{}x{}.png", size[0], size[1])), png)
                    .unwrap();
            }
            let tops: Vec<[u8; 3]> = common.iter().take(2).map(|c| c.0).collect();
            for want in [top.rgb(), bottom.rgb()] {
                assert!(tops.contains(&want), "{name}: {want:?} not among {tops:?}");
            }
            if size == [640, 480] && mark.is_none() {
                for (got, want) in [b[0], b[1], b[2] + 1, b[3] + 1]
                    .into_iter()
                    .zip([533, 212, 603, 272])
                {
                    assert!((got as i32 - want).abs() <= 2, "{name}: {b:?}");
                }
            }
        }
    }
}
