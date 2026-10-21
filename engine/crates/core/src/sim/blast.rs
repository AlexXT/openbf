//! An explosion's area damage (card VDTH, `analysis/re-vehicle-death.md`
//! [O]): the ONE function every blast goes through — today a destroyed
//! vehicle's `ExplosionName` ([`Sim::destroy_vehicle`]), later any ordnance
//! that explodes. B1 FUN_002701a0 / B2 FUN_0030c8f8 (§2): gather at most 20
//! objects in an axis-aligned box, keep those a ray from the blast reaches
//! past buildings and terrain, and hand each the class's damage times the
//! falloff on its sphere (§3) through the bolt's TakeDamage (§4). A soldier
//! riding a vehicle takes none (§5). Push (§5) is card KNOCK's: the class
//! carries its numbers and [`Sim::explode`] marks where it goes.

use super::state::{Event, Sim};
use crate::soldier::Stance;
use crate::unit::UnitHandle;
use crate::vehicle::{VehicleHandle, VehicleKind};
use crate::weapon::{blast_falloff, ExplosionClass};
use glam::Vec3;

/// A blast gathers at most this many objects (the `20` passed to B1
/// FUN_00161c00 B1:194072 / B2 FUN_00161a98 B2:300525).
pub const BLAST_GATHER_MAX: usize = 20;

/// A soldier's collision sphere as a blast measures it — `(height of the
/// centre above the feet, radius w)` — per stance, measured in BF1 RAM over
/// 2,652 soldiers (§3 [V-RAM]; the code that writes them was not found [H]).
/// PROVISIONAL: sprint, jump, fall and roll take the standing sphere.
pub fn soldier_blast_sphere(stance: Stance) -> (f32, f32) {
    match stance {
        Stance::Crouch => (0.75, 1.102_27),
        Stance::Prone => (0.35, 1.622_066),
        _ => (1.0, 1.462_635),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Victim {
    Unit(UnitHandle),
    Vehicle(VehicleHandle),
}

impl Sim {
    /// Detonate `e` at `pos` (§2). `attacker` is the unit credited with the
    /// damage and kills (none for an empty vehicle's blast), `team` the
    /// blast's team for the friendly-fire rule — a victim of that team takes
    /// `friendly_fire_scale`, and a blast whose attacker is not a human
    /// player cannot hurt a human player of that team at all (§4, B1
    /// FUN_001bc190).
    ///
    /// The gather (§2 step 3): live units, then live vehicles, in slot order —
    /// the PS2 walks its collision layers' lists [H: the order] — whose box
    /// (sphere centre ± w, FUN_00168830) overlaps the blast's (pos ± r, r =
    /// [`ExplosionClass::gather_radius`]); the first [`BLAST_GATHER_MAX`],
    /// handled from the end of the list. Riders are gathered (they fill the
    /// cap) and then skipped unless their mount is a building turret (§5).
    /// LOS (§2 step 4): the ray to the sphere centre — the PS2's target point
    /// for most classes; the soldier's own FUN_0020c680 [H] — must clear the
    /// terrain and the world's ORDNANCE subset (PROVISIONAL: the port's
    /// stand-in for the building layer; units and vehicles never block — the
    /// hulls are skipped, [`crate::collision::HullFilter::Static`]).
    pub fn explode(
        &mut self,
        e: &ExplosionClass,
        pos: Vec3,
        attacker: Option<UnitHandle>,
        team: Option<u8>,
    ) {
        self.explode_from(e, pos, attacker, team, None);
    }

    /// [`Sim::explode`] from a dying vehicle (`source`, card RIDK): the
    /// gather leaves out the vehicle itself and its riders, who die after
    /// the blast ([`Sim::destroy_vehicle`]).
    pub(super) fn explode_from(
        &mut self,
        e: &ExplosionClass,
        pos: Vec3,
        attacker: Option<UnitHandle>,
        team: Option<u8>,
        source: Option<VehicleHandle>,
    ) {
        let r = e.gather_radius();
        if r <= 0.0 {
            return;
        }
        let overlaps = |c: Vec3, w: f32| {
            let d = (c - pos).abs();
            d.x <= r + w && d.y <= r + w && d.z <= r + w
        };
        let mut gathered: Vec<(Victim, Vec3, f32)> = Vec::new();
        for (i, u) in self.units.iter().enumerate() {
            if !u.alive || source.is_some_and(|v| u.mount.is_some_and(|m| m.vehicle == v)) {
                continue;
            }
            let (lift, w) = soldier_blast_sphere(u.soldier.stance);
            let c = u.soldier.pos + Vec3::Y * lift;
            if overlaps(c, w) {
                gathered.push((Victim::Unit(self.handle(i)), c, w));
            }
        }
        for (h, v) in self.alive_vehicles() {
            if source == Some(h) {
                continue;
            }
            let (c, w) = match self.blast_setup.get(v.class as usize) {
                Some(Some(b)) => (v.pos + v.rot * b.sphere_centre, b.sphere_radius),
                _ => (v.pos, 0.0),
            };
            if overlaps(c, w) {
                gathered.push((Victim::Vehicle(h), c, w));
            }
        }
        gathered.truncate(BLAST_GATHER_MAX);
        let attacker_is_player = attacker
            .and_then(|a| self.units.get(a.index as usize).filter(|u| u.gen == a.gen))
            .is_some_and(|u| u.is_player);
        let desc = e.desc();
        for &(victim, c, w) in gathered.iter().rev() {
            if !crate::perception::los_past(self, pos, c, crate::collision::HullFilter::Static) {
                continue;
            }
            // Card KNOCK: the push (`vt+0x84`, §5) goes here, on the same
            // gather and LOS, when `e.push_outer > 0`.
            if e.damage_outer <= 0.0 {
                continue;
            }
            let f = blast_falloff((c - pos).length(), w, e.damage_inner, e.damage_outer);
            if f <= 0.0 {
                continue;
            }
            match victim {
                Victim::Unit(h) => self.blast_unit(h, &desc, f, attacker, attacker_is_player, team),
                Victim::Vehicle(h) => {
                    self.damage_vehicle_by(h, &desc, f, attacker, team);
                }
            }
        }
    }

    /// One soldier's splash (B1 FUN_00200e68 → FUN_00221600, §3-§5): riders
    /// of anything but a building turret skip; TakeDamage with the falloff
    /// as the multiplier; D12's human damage layer as for a bolt; the kill
    /// through [`Sim::unit_killed`].
    fn blast_unit(
        &mut self,
        h: UnitHandle,
        desc: &crate::weapon::DamageDesc,
        f: f32,
        attacker: Option<UnitHandle>,
        attacker_is_player: bool,
        team: Option<u8>,
    ) {
        // A rider died in an earlier chained blast, or was killed above.
        let Some(v) = self.unit(h) else {
            return;
        };
        if let Some(m) = v.mount {
            if self
                .vehicle(m.vehicle)
                .is_some_and(|x| x.kind != VehicleKind::Turret)
            {
                return;
            }
        }
        let (victim_team, victim_is_player) = (v.team, v.is_player);
        let friendly = team == Some(victim_team);
        if victim_is_player && (self.cheats.invincible || (friendly && !attacker_is_player)) {
            return;
        }
        // D12 (OURS, PROVISIONAL, as in `resolve_bolt`): a bot's damage into
        // a human enemy carries the tier's multiplier.
        let mult = if self.human_damage_on && victim_is_player && !attacker_is_player && !friendly {
            team.map_or(self.difficulty, |t| self.difficulty_of(t))
                .tier()
                .human_damage
        } else {
            1.0
        };
        let mult = mult * self.buff_scale(attacker, h);
        let (ff, now) = (self.friendly_fire_scale, self.tick);
        let u = self.unit_mut(h).unwrap();
        let damage = u.health.take_damage(desc, f * mult, friendly, ff);
        let (killed, health_left) = (!u.health.alive(), u.health.cur);
        if let Some(a) = attacker {
            u.last_hit = Some((a, now));
            self.events.push(Event::Hit {
                shooter: a,
                victim: h,
                damage,
                age: 0.0,
                health_left,
            });
        }
        if killed {
            self.unit_killed(h, attacker);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::tests::flat_ground;
    use crate::sim::DeathCause;
    use crate::unit::UnitSpec;
    use crate::vehicle::{BlastSetup, SeatKind, VehicleSpec};
    use crate::weapon::HealthType;

    /// `imp_walk_atst_exp` (§7): Damage 50, DamageRadius 20, Push 13 / 7.
    fn atst_exp() -> ExplosionClass {
        ExplosionClass {
            damage: 50.0,
            damage_outer: 20.0,
            push: 13.0,
            push_outer: 7.0,
            ..Default::default()
        }
    }

    /// The AAT / fighter-tank class (§7): 100 / 4, VehicleScale 0.
    fn tank_exp() -> ExplosionClass {
        let mut e = ExplosionClass {
            damage: 100.0,
            damage_outer: 4.0,
            ..Default::default()
        };
        e.scales[HealthType::Vehicle as usize] = 0.0;
        e
    }

    fn bike(pos: Vec3) -> VehicleSpec {
        VehicleSpec {
            kind: VehicleKind::Hover,
            class: 0,
            home_team: None,
            neutral_when_empty: true,
            spawn_pad: None,
            pos,
            yaw: 0.0,
            max_health: 500.0,
            health_type: HealthType::Vehicle,
            seats: vec![SeatKind::Pilot],
        }
    }

    fn sim() -> Sim {
        let mut s = Sim::empty(3);
        s.heightfield = Some(flat_ground());
        s
    }

    /// §3's falloff on d' = d − w/2 with the disc's usual Inner 0: full
    /// damage within w/2 of the centre, linear to 0 at Outer + w/2.
    #[test]
    fn falloff_is_linear_on_the_sphere_surface_distance() {
        assert_eq!(blast_falloff(0.5, 1.0, 0.0, 20.0), 1.0 - 0.0 / 20.0);
        assert_eq!(blast_falloff(0.4, 1.0, 0.0, 20.0), 1.0);
        assert_eq!(blast_falloff(10.5, 1.0, 0.0, 20.0), 0.5);
        assert_eq!(blast_falloff(20.5, 1.0, 0.0, 20.0), 0.0);
        // The rep fighter tank's 4.5–6.5: flat to 4.5, then 2 m of ramp.
        assert_eq!(blast_falloff(4.0, 0.0, 4.5, 6.5), 1.0);
        assert_eq!(blast_falloff(5.5, 0.0, 4.5, 6.5), 0.5);
        assert_eq!(blast_falloff(6.5, 0.0, 4.5, 6.5), 0.0);
    }

    /// A standing trooper 10 m from an AT-ST's blast: centre 1 m up, w
    /// 1.4626 → d' = √101 − 0.7313, f = (20 − d')/20, damage 50·f.
    #[test]
    fn a_soldier_takes_the_falloff_of_its_stance_sphere() {
        let mut s = sim();
        let t = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(10.0, 0.0, 0.0)))
            .unwrap();
        let hp = s.unit(t).unwrap().health.cur;
        s.explode(&atst_exp(), Vec3::ZERO, None, Some(0));
        let d = 101f32.sqrt() - 1.462_635 * 0.5;
        let want = 50.0 * (20.0 - d) / 20.0;
        let got = hp - s.unit(t).unwrap().health.cur;
        assert!((got - want).abs() < 1e-3, "{got} vs {want}");
        // Prone, the same spot: centre 0.35 up, w 1.6221 — closer, more.
        s.units[t.index as usize].health.cur = hp;
        s.units[t.index as usize].soldier.stance = Stance::Prone;
        s.explode(&atst_exp(), Vec3::ZERO, None, Some(0));
        let d = (100.0f32 + 0.35 * 0.35).sqrt() - 1.622_066 * 0.5;
        let want = 50.0 * (20.0 - d) / 20.0;
        let got = hp - s.unit(t).unwrap().health.cur;
        assert!((got - want).abs() < 1e-3, "{got} vs {want}");
    }

    /// Terrain blocks a blast: a trooper behind a ridge takes nothing.
    #[test]
    fn terrain_blocks_the_blast() {
        let mut s = Sim::empty(3);
        // A 30 m wall of terrain between x = 2 and x = 6 (4 m cells).
        let mut hf = flat_ground();
        hf.size = 1001;
        hf.unit = 4.0;
        hf.heights = vec![0.0; 1001 * 1001];
        for z in 0..1001 {
            hf.heights[z * 1001 + 501] = 30.0;
        }
        s.heightfield = Some(hf);
        let far = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(10.0, 0.0, 0.0)))
            .unwrap();
        let near = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(-3.0, 0.0, 0.0)))
            .unwrap();
        let hp = s.unit(far).unwrap().health.cur;
        s.explode(&atst_exp(), Vec3::new(-1.0, 0.5, 0.0), None, Some(0));
        assert_eq!(s.unit(far).unwrap().health.cur, hp, "behind the ridge");
        assert!(s.unit(near).unwrap().health.cur < hp, "in the open");
    }

    /// Riders of a vehicle take no splash (§5); the vehicle's hull does,
    /// through its HealthType's scale — 0 for the tank class (§7).
    #[test]
    fn riders_skip_and_the_hull_takes_its_scale() {
        let mut s = sim();
        s.blast_setup = vec![Some(BlastSetup {
            explosion: None,
            sphere_centre: Vec3::new(0.0, 1.0, 0.0),
            sphere_radius: 3.1467,
        })];
        let v = s.spawn_vehicle(&bike(Vec3::new(3.0, 0.0, 0.0)));
        let rider = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(3.0, 0.0, 0.0)))
            .unwrap();
        assert!(s.seat_unit(rider, v, 0));
        let hp = s.unit(rider).unwrap().health.cur;
        s.explode(&atst_exp(), Vec3::ZERO, None, Some(0));
        assert_eq!(s.unit(rider).unwrap().health.cur, hp, "a rider is spared");
        // d = √10, w 3.1467: d' = 1.589, f = 0.9206; 500 − 46.03.
        let d = 10f32.sqrt() - 3.1467 * 0.5;
        let want = 500.0 - 50.0 * (20.0 - d) / 20.0;
        let got = s.vehicle(v).unwrap().health.cur;
        assert!((got - want).abs() < 1e-3, "{got} vs {want}");
        s.explode(&tank_exp(), Vec3::ZERO, None, Some(0));
        assert_eq!(s.vehicle(v).unwrap().health.cur, got, "VehicleScale 0");
    }

    /// At most 20 are gathered, handled from the end of the list: of 25
    /// troopers in slot order, the last five are out of the cap.
    #[test]
    fn the_gather_stops_at_twenty() {
        let mut s = sim();
        let hs: Vec<_> = (0..25)
            .map(|i| {
                let a = i as f32 * 0.25;
                s.spawn(&UnitSpec::trooper(
                    1,
                    Vec3::new(a.cos() * 2.0, 0.0, a.sin() * 2.0),
                ))
                .unwrap()
            })
            .collect();
        let hp = s.unit(hs[0]).unwrap().health.cur;
        s.explode(&atst_exp(), Vec3::ZERO, None, Some(0));
        let hit = hs
            .iter()
            .filter(|&&h| s.unit(h).unwrap().health.cur < hp)
            .count();
        assert_eq!(hit, BLAST_GATHER_MAX);
        assert!(s.unit(hs[24]).unwrap().health.cur == hp);
    }

    /// §4's team rules: a blast of team 0 whose attacker is a bot spares a
    /// human of team 0 entirely, scales a bot of team 0 by the friendly-fire
    /// scale, and hits team 1 in full; a kill credits the attacker.
    #[test]
    fn team_rules_and_the_kill_credit() {
        let mut s = sim();
        s.friendly_fire_scale = 0.5;
        let bot = s
            .spawn(&UnitSpec::trooper(0, Vec3::new(0.0, 0.0, 30.0)))
            .unwrap();
        let human = s
            .spawn(&UnitSpec::player(0, Vec3::new(1.0, 0.0, 0.0)))
            .unwrap();
        let mate = s
            .spawn(&UnitSpec::trooper(0, Vec3::new(-1.0, 0.0, 0.0)))
            .unwrap();
        let foe = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(0.0, 0.0, 1.0)))
            .unwrap();
        let hp = s.unit(mate).unwrap().health.cur;
        s.explode(&atst_exp(), Vec3::ZERO, Some(bot), Some(0));
        assert_eq!(
            s.unit(human).unwrap().health.cur,
            hp,
            "AI blast, human mate"
        );
        // d = √2, w/2 = 0.7313 → f = (20 − 0.6829)/20.
        let f = (20.0 - (2f32.sqrt() - 1.462_635 * 0.5)) / 20.0;
        let mate_hit = hp - s.unit(mate).unwrap().health.cur;
        assert!((mate_hit - 25.0 * f).abs() < 1e-3, "{mate_hit}");
        let foe_hit = hp - s.unit(foe).unwrap().health.cur;
        assert!((foe_hit - 50.0 * f).abs() < 1e-3, "{foe_hit}");
        // Enough blasts kill the foe: the kill is the bot's.
        while s.unit(foe).is_some() {
            s.explode(&atst_exp(), Vec3::ZERO, Some(bot), Some(0));
        }
        assert!(s.events.contains(&Event::Kill {
            shooter: bot,
            victim: foe
        }));
        assert!(s.unit(bot).unwrap().kills >= 1);
    }

    /// The whole chain: a destroyed vehicle's class explosion hits a trooper
    /// beside it — its pilot dies with it (and is credited), the blast is
    /// the pilot's on the vehicle's team; an empty vehicle's blast kills
    /// with no attacker ([`DeathCause::Blast`]).
    #[test]
    fn a_destroyed_vehicle_detonates_its_class() {
        let mut s = sim();
        s.blast_setup = vec![Some(BlastSetup {
            explosion: Some(ExplosionClass {
                damage: 1000.0,
                damage_outer: 10.0,
                ..Default::default()
            }),
            sphere_centre: Vec3::ZERO,
            sphere_radius: 3.0,
        })];
        let v = s.spawn_vehicle(&bike(Vec3::ZERO));
        let pilot = s.spawn(&UnitSpec::trooper(0, Vec3::ZERO)).unwrap();
        assert!(s.seat_unit(pilot, v, 0));
        let foe = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(4.0, 0.0, 0.0)))
            .unwrap();
        s.destroy_vehicle(v, None);
        assert!(s.unit(foe).is_none(), "the blast killed the bystander");
        assert!(s.events.contains(&Event::Kill {
            shooter: pilot,
            victim: foe
        }));

        let v = s.spawn_vehicle(&bike(Vec3::ZERO));
        let other = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(4.0, 0.0, 0.0)))
            .unwrap();
        s.destroy_vehicle(v, None);
        assert!(s.events.contains(&Event::EnvDeath {
            unit: other,
            cause: DeathCause::Blast
        }));
    }
}
