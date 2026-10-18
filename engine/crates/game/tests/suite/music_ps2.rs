//! Card AMBM's data pair: the baked `Music` records and stream takes of BF1
//! Naboo and BF2 Utapau against the disc's numbers
//! (`analysis/re-ambience-music.md` 3.11) and the PS2's take timeline (3.12),
//! and a real take decoded from the disc's LVL.
//!
//! Skips cleanly when the map is not baked, the bake is older than the
//! `music` records, or the extracted disc is not there.
//! `OPENBF_SOUND_ROOT=<dir>` points it at a scratch bake.

use core::music::{Change, Entry, Player};
use game::mission::MissionDef;
use game::music::BattleMusic;
use game::sounds::SoundLib;
use std::path::PathBuf;

fn root() -> PathBuf {
    match std::env::var("OPENBF_SOUND_ROOT") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => game::assets::baked_root(),
    }
}

fn lib(map: &str, music: &str) -> Option<SoundLib> {
    let sl = SoundLib::load(&root().join(map))?;
    if sl.music(music).is_none() {
        eprintln!("SKIP: {map} is baked without Music records (rebake sounds)");
        return None;
    }
    Some(sl)
}

fn mission(name: &str) -> MissionDef {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../missions");
    MissionDef::load(&PathBuf::from(dir).join(name)).expect("mission")
}

#[test]
fn naboos_records_are_the_discs_and_its_takes_follow_the_ps2() {
    let Some(sl) = lib("bf1_nab1", "cis_nab_amb_middle") else {
        eprintln!("SKIP: bf1_nab1 not baked");
        return;
    };
    let times = |n: &str| {
        let d = sl.music(n).expect(n).def;
        (
            d.fade_in,
            d.fade_out,
            d.min_playback,
            d.min_inactive,
            d.priority,
        )
    };
    assert_eq!(times("cis_nab_amb_start"), (1.0, 1.0, 5.0, 90.0, 0.0));
    assert_eq!(times("cis_nab_amb_middle"), (1.0, 1.0, 5.0, 30.0, 0.0));
    assert_eq!(times("cis_nab_amb_end"), (1.0, 1.0, 5.0, 30.0, 0.0));
    assert_eq!(times("cis_nab_amb_victory"), (0.0, 1.0, 5.0, 60.0, 1.0));
    let mid = sl.music("cis_nab_amb_middle").unwrap();
    assert_eq!(
        mid.ids,
        [0x8233_8421, 0xB033_8D63, 0xAF33_8BD0, 0xB633_96D5]
    );
    assert!(!mid.def.looping && mid.def.max_playback.is_none());
    for (got, want) in mid.def.takes.iter().zip([28.30, 26.66, 26.12, 31.29]) {
        assert!((got - want).abs() < 0.01, "{:?}", mid.def.takes);
    }
    let start = sl.music("cis_nab_amb_start").unwrap();
    assert!((start.def.takes[0] - 53.9).abs() < 0.05 && start.def.takes.len() == 1);

    // The PS2's run: take AF338BD0 at clock 176.61, B63396D5 at 233.024,
    // 82338421 at 294.586 - the baked record through the port's player.
    let mut p = Player::new(vec![mid.def.clone()]);
    p.musics[0].take = 2;
    let mut e = Entry::new(0);
    let (dt, mut t, mut starts) = (1.0 / 60.0, 176.61f32, Vec::new());
    for _ in 0..(140.0 / dt) as usize {
        e.play(&mut p);
        for c in p.update(dt) {
            if let Change::Started { take, .. } = c {
                starts.push((t, mid.ids[take]));
            }
        }
        t += dt;
    }
    let ps2 = [
        (176.61f32, 0xAF33_8BD0u32),
        (233.024, 0xB633_96D5),
        (294.586, 0x8233_8421),
    ];
    assert_eq!(starts.len(), 3, "{starts:x?}");
    for ((at, id), (want, want_id)) in starts.iter().zip(ps2) {
        assert_eq!(*id, want_id, "{starts:x?}");
        assert!((at - want).abs() < 0.5, "{at} against the PS2's {want}");
    }
    println!("take starts {starts:x?} against the PS2's {ps2:x?}");
}

#[test]
fn utapaus_records_are_the_discs() {
    let Some(sl) = lib("bf2_uta1", "cis_uta_amb_start") else {
        eprintln!("SKIP: bf2_uta1 not baked");
        return;
    };
    for (name, takes) in [
        ("cis_uta_amb_start", 7),
        // The property lists 7; its 9485E44C is in no stream of the disc
        // and is left out (what the PS2 does on drawing it is not read).
        ("cis_uta_amb_middle", 6),
        ("cis_uta_amb_end", 4),
    ] {
        let t = sl.music(name).expect(name);
        let d = &t.def;
        assert_eq!(
            (d.fade_in, d.fade_out, d.min_playback, d.min_inactive),
            (0.0, 1.5, 20.0, 15.0),
            "{name}"
        );
        assert_eq!((d.priority, d.looping), (1.0, true), "{name}");
        assert!(d.max_playback.is_some_and(|m| (150.0..=180.0).contains(&m)));
        assert_eq!(d.takes.len(), takes, "{name}");
        // Takes of 80 to 127 s: every one loops before MaxPlaybackTime.
        assert!(d.takes.iter().all(|s| (60.0..130.0).contains(s)), "{d:?}");
        // The stream is GLOBAL.LVL's stereo `cw_music`.
        assert!(t.takes.iter().all(|k| k.channels == 2));
    }
    let v = sl.music("cis_uta_amb_victory").unwrap().def;
    assert_eq!((v.priority, v.looping, v.takes.len()), (9.0, false, 1));
}

/// Both showcase missions find every Music they name, and the thresholds
/// are the PS2's RAM values.
#[test]
fn the_missions_tables_bind_to_the_bank() {
    for (map, file, music, thr) in [
        (
            "bf1_nab1",
            "bf1_nab1_con_cw.ron",
            "cis_nab_amb_start",
            [200, 198, 20],
        ),
        (
            "bf2_uta1",
            "bf2_uta1_con_cw.ron",
            "cis_uta_amb_start",
            [150, 120, 30],
        ),
    ] {
        let Some(sl) = lib(map, music) else {
            eprintln!("SKIP: {map} not baked");
            continue;
        };
        let m = mission(file);
        let mut b = BattleMusic::new(&m.music, m.source_game == "bf2", m.tickets());
        assert_eq!(b.bind(&sl), Vec::<String>::new(), "{map}");
        assert_eq!(b.ambient.thr, [thr; 2], "{map}");
        assert_eq!(b.player.musics.len(), 10, "{map}");
    }
}

/// A take off the disc: BF1 Naboo's `cis_nab_amb_start`, decoded block by
/// block - the whole length, not silent, and two channels that belong
/// together (a wrong block layout decorrelates them).
#[test]
fn a_take_decodes_from_the_discs_lvl() {
    let Some(sl) = lib("bf1_nab1", "cis_nab_amb_start") else {
        eprintln!("SKIP: bf1_nab1 not baked");
        return;
    };
    let take = sl.music("cis_nab_amb_start").unwrap().takes[0].clone();
    if !take.path.exists() {
        eprintln!("SKIP: no extracted disc at {}", take.path.display());
        return;
    }
    assert_eq!((take.channels, take.frequency), (2, 48_000));
    let mut r = audio::TakeReader::open(take.clone()).expect("open");
    let (mut n, mut blocks) = (0usize, 0usize);
    let (mut sl2, mut sr2, mut slr) = (0.0f64, 0.0f64, 0.0f64);
    loop {
        let b = r.next_block().expect("read");
        if b.is_empty() {
            break;
        }
        n += b.len();
        blocks += 1;
        for [l, r] in b {
            let (l, r) = (f64::from(l), f64::from(r));
            sl2 += l * l;
            sr2 += r * r;
            slr += l * r;
        }
    }
    assert_eq!(n, take.frames());
    assert_eq!(blocks, take.frames().div_ceil(r.block_frames()));
    let rms = (sl2 / n as f64).sqrt();
    let corr = slr / (sl2 * sr2).sqrt();
    println!(
        "{} frames, {:.1} s, rms {rms:.4}, left-right correlation {corr:.3}",
        n,
        take.secs()
    );
    assert!(rms > 0.01, "silent: rms {rms}");
    assert!(corr > 0.3, "left-right correlation {corr}");
}
