use super::*;
use manifest::EngineGraph;

type Points = &'static [(f32, f32)];

fn layer(
    delay: f32,
    speed: Points,
    acceleration: Points,
    on: [Points; 2],
    off: [Points; 2],
) -> EngineLayer {
    let g = |parameter: &str, target: &str, points: Points| EngineGraph {
        parameter: parameter.into(),
        target: target.into(),
        points: points.to_vec(),
    };
    EngineLayer {
        sound: 0,
        delay,
        deviation: 0.0,
        graphs: vec![
            g("speed", "Pitch", speed),
            g("acceleration", "Gain", acceleration),
            g("turnontime", "Gain", on[0]),
            g("turnofftime", "Gain", off[0]),
            g("turnontime", "Pitch", on[1]),
            g("turnofftime", "Pitch", off[1]),
        ],
    }
}

/// BF1 NAB.LVL `cis_hover_aat_engine_parameterized` (re-vehicle-sounds.md
/// §1; `lead_tmp/vsnd/param.py bf1 NAB --sub nab1cw aat_engine`).
fn aat() -> Vec<EngineLayer> {
    let on: [Points; 2] = [&[(0.0, 0.0), (2.0, 1.0)], &[(0.0, 0.5), (3.0, 1.0)]];
    let off: [Points; 2] = [
        &[(0.0, 1.0), (3.0, 0.0)],
        &[(0.0, 1.0), (2.0, 0.6), (3.0, 0.3)],
    ];
    vec![
        layer(
            0.0,
            &[(0.0, 0.8), (0.5, 0.9), (1.0, 1.0)],
            &[(-1.0, 1.0), (0.0, 0.7), (1.0, 1.0)],
            on,
            off,
        ),
        layer(
            0.0,
            &[(0.0, 0.85), (0.5, 0.9), (1.0, 1.0)],
            &[(-1.0, 0.6), (0.0, 0.6), (0.5, 0.9), (1.0, 1.0)],
            on,
            off,
        ),
        layer(
            0.0,
            &[(0.0, 0.6), (0.5, 0.8), (1.0, 1.0)],
            &[(-1.0, 0.8), (0.0, 0.8), (0.5, 0.8), (1.0, 1.0)],
            on,
            off,
        ),
    ]
}

/// BF2 UTA.LVL `atst_engine_parameterized`, in the record's order.
fn atst() -> Vec<EngineLayer> {
    let on: [Points; 2] = [&[(0.0, 0.0), (1.0, 1.0)], &[(0.0, 0.7), (1.0, 1.0)]];
    let off: [Points; 2] = [&[(0.0, 1.0), (1.0, 0.0)], &[(0.0, 1.0), (1.0, 0.1)]];
    let pitch: Points = &[(-0.1, 1.0), (0.0, 0.8), (0.5, 0.9), (1.0, 1.0)];
    vec![
        layer(0.8, pitch, &[(-0.1, 1.0), (0.0, 8.0), (1.0, 1.0)], on, off),
        layer(
            0.8,
            &[(0.0, 0.9), (1.0, 1.0)],
            &[(0.0, 1.0), (1.0, 0.8)],
            on,
            off,
        ),
        layer(0.8, pitch, &[(-0.1, 0.7), (0.0, 0.5), (1.0, 0.7)], on, off),
    ]
}

const DT: f32 = 1.0 / 30.0;
const HOVER: Drive = Drive::Hover {
    forward: 12.0,
    reverse: 10.0,
};

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

fn all(layers: &[EngineLayer], i: &Inputs) -> Vec<(f32, f32)> {
    layers.iter().map(|l| factors(l, i)).collect()
}

#[test]
fn a_graph_is_linear_between_its_points_and_flat_outside() {
    let g = [(0.0, 0.8), (0.5, 0.9), (1.0, 1.0)];
    for (x, y) in [
        (-3.0, 0.8),
        (0.0, 0.8),
        (0.25, 0.85),
        (0.5, 0.9),
        (0.9, 0.98),
        (1.0, 1.0),
        (7.0, 1.0),
    ] {
        assert!(near(graph(&g, x), y), "{x}: {}", graph(&g, x));
    }
    assert_eq!(graph(&[], 0.3), 1.0);
}

/// The AAT long after its turn-on: at rest the PS2's voices read pitch 0.80
/// / 0.85 / 0.60 and gain 0.63 / 0.54 / 0.72 = 0.9 x 0.7 / 0.6 / 0.8 (RAM,
/// §6.1); at full speed every pitch factor is 1.
#[test]
fn the_aat_at_rest_and_at_full_speed() {
    let rest = Inputs {
        health: 1.0,
        timer: 200.0,
        ..Inputs::default()
    };
    let got = all(&aat(), &rest);
    for (g, w) in got.iter().zip([(0.7, 0.8), (0.6, 0.85), (0.8, 0.6)]) {
        assert!(near(g.0, w.0) && near(g.1, w.1), "{got:?}");
    }
    let full = Inputs { speed: 1.0, ..rest };
    assert!(all(&aat(), &full).iter().all(|f| near(f.1, 1.0)));
    // A gain graph on a slope.
    let moving = Inputs {
        acceleration: 0.5,
        ..full
    };
    let got = all(&aat(), &moving);
    assert!(
        near(got[0].0, 0.85) && near(got[1].0, 0.9) && near(got[2].0, 0.8),
        "{got:?}"
    );
}

/// Graphs on one target multiply: 1 s into the turn-on the gain factor is
/// the acceleration graph x the fade (0.7 x 0.5), the pitch the speed graph
/// x the rise (0.8 x 2/3); the run-down reads the other pair.
#[test]
fn graphs_on_one_target_multiply() {
    let on = Inputs {
        timer: 1.0,
        ..Inputs::default()
    };
    let (g, p) = factors(&aat()[0], &on);
    assert!(
        near(g, 0.7 * 0.5) && near(p, 0.8 * (0.5 + 0.5 / 3.0)),
        "{g} {p}"
    );
    let off = Inputs {
        turning_off: true,
        ..on
    };
    let (g, p) = factors(&aat()[0], &off);
    assert!(near(g, 0.7 * (2.0 / 3.0)) && near(p, 0.8 * 0.8), "{g} {p}");
}

fn motion(piloted: bool, forward: f32, throttle: f32) -> Motion {
    Motion {
        piloted,
        forward,
        speed: forward.abs(),
        throttle,
        health: 1.0,
    }
}

/// A hover's speed is |forward speed| over `ForwardSpeed`, or over
/// `ReverseSpeed` whenever the throttle is not forward (§6.3: 12.024 m/s ->
/// 1.0020, 11.885 m/s coasting -> 1.1885, -9.729 -> 0.9728).
#[test]
fn a_hovers_speed_is_normalised_by_the_throttles_side() {
    let layers = aat();
    let mut e = Instance::default();
    for (forward, throttle, want) in [
        (12.024, 1.0, 1.002),
        (11.885, 0.0, 1.1885),
        (-9.729, -1.0, 0.9729),
    ] {
        let m = motion(true, forward, throttle);
        e.update(DT, HOVER, 3.0, &m, &layers, || 0.5);
        assert!((e.inputs.speed - want).abs() < 1e-3, "{}", e.inputs.speed);
    }
}

/// The filtered acceleration moves 1 % of the way per update: a step in
/// speed leaves x0.99 per update after it (§6.4: x0.860 = 0.99^15 in 0.5 s).
#[test]
fn the_acceleration_filter_keeps_99_percent_per_update() {
    let layers = aat();
    let mut e = Instance::default();
    let m = motion(true, 12.0, 1.0);
    e.update(DT, HOVER, 3.0, &m, &layers, || 0.5);
    let a0 = e.inputs.acceleration;
    // One update from rest to full speed: (1 - 0) / dt x 0.01.
    assert!(near(a0, 0.3), "{a0}");
    for _ in 0..15 {
        e.update(DT, HOVER, 3.0, &m, &layers, || 0.5);
    }
    let kept = e.inputs.acceleration / a0;
    assert!((kept - 0.99f32.powi(15)).abs() < 1e-4, "{kept}");
}

/// On with a pilot, layers at once (delay 0); the run-down from the update
/// the seat empties, the stop at the first update whose timer has reached
/// `TurnOffTime` (PS2: timer 3.003, §6.5); a pilot inside the run-down
/// turns it on again from 0.
#[test]
fn on_with_a_pilot_and_off_after_the_run_down() {
    let layers = aat();
    let mut e = Instance::default();
    let (pilot, empty) = (motion(true, 0.0, 0.0), motion(false, 0.0, 0.0));
    e.update(DT, HOVER, 3.0, &empty, &layers, || 0.5);
    assert!(!e.on && e.started == [false; 3]);
    e.update(DT, HOVER, 3.0, &pilot, &layers, || 0.5);
    assert!(e.on && !e.turning_off && e.started == [true; 3]);
    assert_eq!(e.inputs.timer, 0.0);
    for _ in 0..90 {
        e.update(DT, HOVER, 3.0, &pilot, &layers, || 0.5);
    }
    // The run-down: timer from 0, the gain falling along the graph.
    e.update(DT, HOVER, 3.0, &empty, &layers, || 0.5);
    assert!(e.on && e.turning_off && e.inputs.timer == 0.0);
    let mut updates = 1;
    let mut last = e.inputs;
    while e.on {
        last = e.inputs;
        e.update(DT, HOVER, 3.0, &empty, &layers, || 0.5);
        updates += 1;
        assert!(updates < 200);
    }
    // 90 or 91 updates sent (float sums of 1 / 30), then the stop.
    assert!((91..=92).contains(&updates), "{updates}");
    assert!(
        last.timer < 3.0 && last.timer + DT >= 3.0 - 1e-4,
        "{last:?}"
    );
    assert!(factors(&layers[0], &last).0 < 0.02);
    assert!(e.started == [false; 3]);
    // Boarded again during a run-down.
    e.update(DT, HOVER, 3.0, &pilot, &layers, || 0.5);
    for _ in 0..30 {
        e.update(DT, HOVER, 3.0, &empty, &layers, || 0.5);
    }
    assert!(e.on && e.turning_off && e.inputs.timer > 0.9);
    e.update(DT, HOVER, 3.0, &pilot, &layers, || 0.5);
    assert!(e.on && !e.turning_off && e.inputs.timer == 0.0);
    // `TurnOffTime` 0 (the droideka's): off in the update the seat empties.
    e.update(DT, HOVER, 0.0, &empty, &layers, || 0.5);
    assert!(!e.on);
}

/// A walker sends m/s and `previous - speed / dt`: walking at 5.58 m/s the
/// pitch factors are 1 and each gain graph is read at its left end (BF2
/// RAM, §6.7); its layers start 0.8 s after the turn-on.
#[test]
fn a_walker_reads_its_gain_graphs_at_the_left_end() {
    let layers = atst();
    let mut e = Instance::default();
    let m = Motion {
        piloted: true,
        speed: 5.58,
        health: 1.0,
        ..Motion::default()
    };
    let mut first = None;
    for n in 0..120 {
        e.update(DT, Drive::Walker, 3.0, &m, &layers, || 0.5);
        if first.is_none() && e.started[0] {
            first = Some(n);
        }
    }
    assert!(e.started == [true; 3]);
    assert!((24..=25).contains(&first.unwrap()), "{first:?}");
    assert!(near(e.inputs.speed, 5.58) && e.inputs.acceleration < -100.0);
    let got = all(&layers, &e.inputs);
    for (g, w) in got.iter().zip([1.0, 1.0, 0.7]) {
        assert!(near(g.0, w) && near(g.1, 1.0), "{got:?}");
    }
    // Standing: acceleration 0, the graphs' own points at 0.
    let still = Motion { speed: 0.0, ..m };
    e.update(DT, Drive::Walker, 3.0, &still, &layers, || 0.5);
    e.update(DT, Drive::Walker, 3.0, &still, &layers, || 0.5);
    let got = all(&layers, &e.inputs);
    assert!(
        near(got[0].0, 8.0) && near(got[2].0, 0.5) && near(got[0].1, 0.8),
        "{got:?}"
    );
}

const VSND: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../artifacts/lead_tmp/vsnd");

/// The PS2's own numbers through the port's code (card VSND's referee): the
/// BAKED NAB1 records and [`factors`] against the multipliers in the PS2's
/// memory, per layer per sampled update of two live sessions
/// (`lead_tmp/vsnd/live{1,2}.csv`: stored speed, acceleration, health,
/// timer and flags of every hover instance), with the fixed rule of
/// `lead_tmp/vsnd/verify/strict_replay.py` - the timer sent is the sampled
/// one less the update's 1 / 30. Data-gated; `VSND_BAKED` names another
/// baked root.
#[test]
fn the_ps2s_multipliers_replay_through_the_baked_records() {
    let root = std::env::var_os("VSND_BAKED")
        .map_or_else(crate::assets::baked_root, std::path::PathBuf::from);
    let Some(lib) = crate::sounds::SoundLib::load(&root.join("bf1_nab1")) else {
        eprintln!("SKIP: bf1_nab1 sounds not baked");
        return;
    };
    let names = [
        "cis_hover_aat_engine_parameterized",
        "cis_hover_stap_engine_parameterized",
        "interceptorTank_engine_parameterized",
    ];
    let mut by_sound = HashMap::new();
    for (k, n) in names.iter().enumerate() {
        let Some(rec) = lib.engine(fnv1a_lowercase(n), &[fnv1a_lowercase("nab1cw")]) else {
            eprintln!("SKIP: {n} not in the bake (rebake sounds)");
            return;
        };
        for l in &rec.layers {
            by_sound.insert(format!("{:08X}", l.sound), (k, l.clone()));
        }
    }
    let (mut rows, mut ok) = ([0usize; 3], [0usize; 3]);
    for file in ["live1.csv", "live2.csv"] {
        let Ok(text) = pak::read_to_string(format!("{VSND}/{file}")) else {
            eprintln!("SKIP: {VSND}/{file} not present");
            return;
        };
        let mut lines = text.lines();
        let head: Vec<&str> = lines.next().unwrap().split(',').collect();
        let col = |n: &str| head.iter().position(|h| *h == n).unwrap();
        let (c_flags, c_timer, c_health, c_speed, c_accel, c_prop, c_g, c_p) = (
            col("flags"),
            col("timer"),
            col("health"),
            col("stored"),
            col("accel"),
            col("prop"),
            col("multG"),
            col("multP"),
        );
        for line in lines {
            let f: Vec<&str> = line.split(',').collect();
            let Some((k, l)) = by_sound.get(f[c_prop]) else {
                continue;
            };
            let flags = u32::from_str_radix(f[c_flags], 16).unwrap();
            if flags & 2 == 0 {
                continue;
            }
            // No hover winds up or down (flags 4, 8).
            assert_eq!(flags & 0xc, 0, "{line}");
            let num = |c: usize| f[c].parse::<f32>().unwrap();
            let i = Inputs {
                speed: num(c_speed),
                acceleration: num(c_accel),
                health: num(c_health),
                timer: (num(c_timer) - 1.0 / 30.0).max(0.0),
                turning_off: flags & 0x10 != 0,
            };
            let (g, p) = factors(l, &i);
            rows[*k] += 1;
            ok[*k] += usize::from((g - num(c_g)).abs().max((p - num(c_p)).abs()) <= 0.002);
        }
    }
    let (n, good) = (rows.iter().sum::<usize>(), ok.iter().sum::<usize>());
    eprintln!(
        "VSND replay: AAT {}/{}, STAP {}/{}, fighter tank {}/{}; {good} of {n} layer-ticks within 0.002",
        ok[0], rows[0], ok[1], rows[1], ok[2], rows[2]
    );
    assert_eq!(n, 19_419);
    assert!(good * 100 >= n * 99, "{good} of {n}");
}

/// BF2 Utapau's baked `atst_engine_parameterized`, layers in the layered
/// record's order: a walker at 5.58 m/s reads gain factors 0.7 / 1.0 / 1.0
/// and pitch factor 1 (RAM, §6.7). Data-gated like the replay.
#[test]
fn utapaus_baked_walker_record_reads_the_rams_factors() {
    let root = std::env::var_os("VSND_BAKED")
        .map_or_else(crate::assets::baked_root, std::path::PathBuf::from);
    let lib = crate::sounds::SoundLib::load(&root.join("bf2_uta1"));
    let rec = lib.as_ref().and_then(|l| {
        l.engine(
            fnv1a_lowercase("atst_engine_parameterized"),
            &[fnv1a_lowercase("uta1cw")],
        )
    });
    let Some(rec) = rec else {
        eprintln!("SKIP: bf2_uta1 engines not baked");
        return;
    };
    let i = Inputs {
        speed: 5.58,
        acceleration: 5.58 - 5.58 * 30.0,
        health: 1.0,
        timer: 50.0,
        turning_off: false,
    };
    let got = all(&rec.layers, &i);
    for (g, w) in got.iter().zip([0.7, 1.0, 1.0]) {
        assert!(near(g.0, w) && near(g.1, 1.0), "{got:?}");
    }
    assert!(rec.layers.iter().all(|l| l.delay == 0.8));
}
