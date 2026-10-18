//! `mungefs::sanm` against the BF2 disc's own `INGAME.LVL` (card ANIMS,
//! `analysis/re-soldier-anim-layers.md` §6.4; gitignored, SKIPs when absent).

use mungefs::sanm::{find_sanm, stored_hundredths, AimType, Layer};

#[test]
fn ingame_lvl_configures_192_soldier_animations() {
    let path = format!(
        "{}/../../../extracted/bf2/lvl/DATA/_LVL_PS2/INGAME.LVL",
        env!("CARGO_MANIFEST_DIR")
    );
    let Ok(buf) = std::fs::read(&path) else {
        eprintln!("SKIP: extracted BF2 INGAME.LVL not present");
        return;
    };
    let sanm = find_sanm(mungefs::open(&buf).unwrap())
        .unwrap()
        .expect("INGAME.LVL has a sanm chunk");
    assert_eq!(sanm.animations.len(), 192);

    let shoot = sanm.get("stand_shoot", Layer::Upper).unwrap();
    assert_eq!(shoot.name, "stand_shoot_upper");
    assert_eq!(shoot.aim, AimType::Torso);
    assert_eq!((shoot.blend_in, shoot.blend_out), (0.02, 0.02));
    assert_eq!(stored_hundredths(shoot.blend_in), 1);
    assert!(!shoot.looping);

    let run = sanm.get("stand_runforward", Layer::Lower).unwrap();
    assert_eq!(run.name, "stand_runforward_lower");
    assert!(run.looping);
    assert_eq!(run.sync.as_deref(), Some("forward"));
    assert_eq!((run.blend_in, run.blend_out), (0.15, 0.15));
    assert_eq!(
        run.blend_to,
        vec![("standalert_runforward".to_string(), 0.0)]
    );
    assert_eq!(run.aim, AimType::None);

    // A bare name configures both layers.
    let sprint = sanm.get("sprint", Layer::Upper).unwrap();
    assert_eq!(sprint, sanm.get("sprint", Layer::Lower).unwrap());
    assert!(sprint.looping);

    // The census of the chunk (lead_tmp/anim/sanm_bf2.txt).
    let n = |f: &dyn Fn(&mungefs::sanm::SoldierAnimation) -> bool| {
        sanm.animations.iter().filter(|a| f(a)).count()
    };
    assert_eq!(n(&|a| a.looping), 115);
    assert_eq!(n(&|a| a.aim == AimType::Head), 28);
    assert_eq!(n(&|a| a.aim == AimType::Torso), 70);
    assert_eq!(n(&|a| a.sync.is_some()), 89);
    assert_eq!(
        sanm.animations
            .iter()
            .map(|a| a.blend_to.len())
            .sum::<usize>(),
        43
    );
}
