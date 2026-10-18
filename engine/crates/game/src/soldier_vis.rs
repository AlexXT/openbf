//! Animated soldier visuals: per-team skinned assets (baked by `bake
//! soldier`), per-unit animators driven by the sim's soldier state, and the
//! skinned draw list. Visual only — nothing here feeds back into the sim.
//!
//! **A rider is not drawn — but in a building turret that shows him** (card
//! VTURG, `analysis/re-turret-draw.md` 3 [O]): a soldier seated in a turret
//! whose seat has a pose key (`vehicle_class::SeatSpec::gunner_draw`) is
//! drawn at `L x node x O` (`viewer::turret_pose::GunnerHang`: the
//! `PilotPosition` hardpoint on its parent node as the turret is posed now -
//! an aimer node as the update before left it - in the building), posed by
//! the seat's clip (`Animator::seat`, the nine-pose numbers of
//! `turret_pose::nine_pose`), in every view, his own too, and without his
//! hand weapon. Hoth's laser towers have no pose key: nobody is drawn.
//!
//! **A vehicle's rider likewise** (card VRIDE, `analysis/re-vehicle-riders.md`
//! [O]): a seat with a pose key (BF2: and exposed) draws its soldier at
//! `O x hang` - the vehicle as drawn, the seat's
//! `vehicle_class::RiderHang` (the hardpoint's bind, or an
//! `AnimatedPilotPosition` node of the posed walker skin now) - posed by the
//! seat's clip: a hover's nine-pose pilot by the drive's smoothed throttle
//! and turn (`turret_pose::HoverNine`, kept per vehicle here), a vehicle
//! turret's nine-pose gunner by his gun's angles, every other seat's clip
//! played and looped. The speeder bikes, STAP, BARC, tauntaun, kaadu,
//! one-man AT-ST and the fighter tanks' gunners show their riders; the hover
//! tanks', AT-STs' and AATs' pilots have no pose key.
//!
//! **The muzzle flash** (card FXMF, `analysis/re-muzzle-flash.md` [O]) is
//! drawn here too, because it rides the held weapon: each `Shot` restarts
//! the unit's timer, and while t = (end − now) / `FlashLength` > 0 the
//! game's flash MODEL (`muzzleflash` / `com_sfx_muzzleflash`) is drawn at
//! `hp_weapons` × the weapon's fire point (its `FirePointName` node, baked
//! beside the weapon GLB; the `hp_weapons` origin when it has none), +Z along
//! the aim, scaled `FlashRadius`, tinted `FlashColor` with alpha
//! min(A, 2A·t) ([`core::flash`]), ADDITIVE in the particle list. Time runs
//! on the SIM clock (the 30 Hz tick), not the render's: every blaster's
//! 0.025-s flash is drawn for exactly the tick of its shot at full alpha, as
//! on the PS2 (§3; FEEL_NOTES). Only the guns a soldier holds flash — the
//! weapon loader gives none to sabres and thrown weapons (§5), and vehicle
//! and turret guns never reach this draw.

use crate::animator::{AnimInput, Animator, ClassAnim, Rig};
use crate::assets::{load_skinned_glb, SkinnedAsset};
use crate::viewer::turret_pose as tp;
use crate::walk::{TeamTemplate, WalkSim};
use glam::{Mat4, Vec3};
use render::{
    ParticleDraw, ParticleShape, Renderer, SkinnedDrawCmd, SkinnedMeshHandle, TextureHandle,
};
use std::collections::HashMap;
use std::path::Path;

/// A part's specular second pass, resolved for the loaded map: its flat
/// colour (`crate::assets::SpecPass::rgb_under` the map's sun) and its power
/// — `None` for a part that draws none (queue SPEC2).
pub type PartSpec = Option<([f32; 3], u32)>;

/// A part's [`PartSpec`] from the pass its GLB material carries and the sun
/// colour on the renderer — which must already be the map's
/// (`crate::viewer::upload_scene` puts it there).
pub fn part_spec(r: &Renderer, pass: Option<crate::assets::SpecPass>) -> PartSpec {
    pass.map(|p| (p.rgb_under(r.lighting.sun_color), p.power))
}

/// A skinned part's draws: `base`, then its specular second pass when it has
/// one and is textured — the add is weighted by the base texel's alpha, so an
/// untextured part draws none (as the world's, `crate::viewer::upload_scene`).
pub fn with_spec(base: SkinnedDrawCmd, spec: PartSpec) -> Vec<SkinnedDrawCmd> {
    let second = match (base.texture, spec) {
        (Some(_), Some((rgb, power))) => Some(base.specular(rgb, power)),
        _ => None,
    };
    std::iter::once(base).chain(second).collect()
}

pub struct SoldierAsset {
    pub asset: SkinnedAsset,
    /// Per skinned submesh: mesh, texture, the pass its GLB material asks
    /// for — `Opaque`, or `Blend` for a segment the disc composites
    /// (`render::SkinnedDrawCmd::layer`) — and its specular second pass.
    pub meshes: Vec<(
        SkinnedMeshHandle,
        Option<TextureHandle>,
        render::DrawLayer,
        PartSpec,
    )>,
    /// The soldier's animation sets over `asset` (`soldiers/<model>.sets.ron`);
    /// `None` when they are not baked: the soldier stands in his bind pose.
    pub rig: Option<Rig>,
    /// Skeleton index of `hp_weapons`: where the class's [`WeaponAsset`] is
    /// drawn.
    pub hp_weapons: Option<usize>,
}

/// A hand weapon model (static, drawn at the body's `hp_weapons`), cached by
/// its own `GeometryName` — apart from the body, because classes that share
/// a body hold different guns (card UWPN: BF1's clone trooper and
/// sharpshooter are both `rep_inf_trooper`).
#[derive(Default)]
pub struct WeaponAsset {
    /// Per weapon submesh: mesh, texture, and the pass its GLB material
    /// asks for. The weapon bake is NOT on ALPHA RULE v2 (design D6) — it
    /// still ORs the material flags per texture — but it writes that decision
    /// on the material, and the part is routed from it. A MASK part draws
    /// through the WORLD's mask pipeline (the disc's cutoff since queue AT);
    /// none of the 31 baked weapon GLBs has a MASK material. The last field
    /// is the part's specular second pass, as [`SoldierAsset::meshes`]'.
    /// Empty when the GLB is not baked (no weapon drawn).
    pub parts: Vec<(
        render::MeshHandle,
        Option<TextureHandle>,
        render::DrawLayer,
        PartSpec,
    )>,
    /// The weapon's `skel` nodes (`weapons/<model>.nodes.ron`, card FXMF):
    /// where its `FirePointName` puts the muzzle flash. Empty when the bake
    /// predates them or the model has none.
    pub nodes: manifest::WeaponNodes,
}

/// What one class draws: its body and the gun in its hand, each an index
/// into its own list. `None` = no model / no hand weapon.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClassDraw {
    pub body: Option<usize>,
    pub weapon: Option<usize>,
}

/// Which models two teams' classes draw, before anything is loaded: every
/// distinct body and every distinct hand weapon once, and each class's pair.
/// The body and the weapon are keyed apart (card UWPN) — a class sharing its
/// body with an earlier class still holds its OWN primary's model.
#[derive(Debug, Default)]
pub struct DrawPlan {
    /// Distinct soldier `GeometryName`s, in first-use order.
    pub bodies: Vec<String>,
    /// Distinct primary-weapon `GeometryName`s, in first-use order.
    pub weapons: Vec<String>,
    /// `per_class[team][class]`, index-aligned with `TeamTemplate::classes`.
    pub per_class: [Vec<ClassDraw>; 2],
}

impl DrawPlan {
    pub fn of(teams: &[TeamTemplate; 2]) -> Self {
        fn slot(names: &mut Vec<String>, name: Option<&str>) -> Option<usize> {
            let name = name?;
            Some(names.iter().position(|x| x == name).unwrap_or_else(|| {
                names.push(name.to_owned());
                names.len() - 1
            }))
        }
        let mut plan = Self::default();
        for (i, t) in teams.iter().enumerate() {
            for c in &t.classes {
                let draw = ClassDraw {
                    body: slot(&mut plan.bodies, c.geometry.as_deref()),
                    weapon: slot(&mut plan.weapons, c.weapon_geometry.as_deref()),
                };
                plan.per_class[i].push(draw);
            }
        }
        plan
    }

    /// The weapon model a class is drawn holding, if any.
    pub fn weapon_of(&self, team: usize, class: usize) -> Option<&str> {
        let w = self.per_class.get(team)?.get(class)?.weapon?;
        self.weapons.get(w).map(String::as_str)
    }
}

/// Triangles as (position, UV), three per triangle.
pub type Tris = Vec<(Vec3, [f32; 2])>;

/// The game's muzzle flash model (card FXMF), loaded once: its triangles in
/// model space, one list per sheet.
pub struct FlashModel {
    /// The model's name (`core::flash::BF1_MODEL` / `BF2_MODEL`).
    pub name: String,
    pub parts: Vec<(Option<TextureHandle>, Tris)>,
}

impl FlashModel {
    /// `common_<game>/models/<name>.glb` and its sheets beside it, or `None`
    /// (printed) when the common bake lacks it.
    pub fn load(r: &mut Renderer, assets_dir: &Path, game: &str) -> Option<Self> {
        let name = core::flash::MuzzleFlashLook::default_model(game == "bf2");
        let common = assets_dir.join(format!("common_{game}"));
        let glb = common.join("models").join(format!("{name}.glb"));
        let cpu = match crate::assets::load_glb(&glb) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("muzzle flash {name}: {e:#} (no flash drawn)");
                return None;
            }
        };
        let mut parts = Vec::new();
        for (start, count, material, _, _) in &cpu.submeshes {
            let tex = crate::viewer::upload_texture_file(
                r,
                &common.join("textures"),
                &material.to_ascii_lowercase(),
            )
            .unwrap_or_else(|e| {
                eprintln!("muzzle flash {name}: {e:#}");
                None
            });
            let tris = cpu.indices[*start as usize..(*start + *count) as usize]
                .iter()
                .map(|&i| {
                    let i = i as usize;
                    (
                        Vec3::from(cpu.positions[i]),
                        cpu.uvs.get(i).copied().unwrap_or([0.0; 2]),
                    )
                })
                .collect();
            parts.push((tex, tris));
        }
        Some(Self {
            name: name.to_owned(),
            parts,
        })
    }

    /// This model placed by `m` (`core::flash::matrix`), coloured `color`:
    /// one additive particle per sheet, keyed at `key`.
    pub fn draws(&self, m: Mat4, color: [u8; 4], key: Vec3) -> Vec<ParticleDraw> {
        self.parts
            .iter()
            .map(|(tex, tris)| ParticleDraw {
                shape: ParticleShape::Mesh {
                    tris: tris
                        .iter()
                        .map(|(p, uv)| (m.transform_point3(*p), *uv))
                        .collect(),
                },
                key,
                color,
                texture: *tex,
                additive: true,
            })
            .collect()
    }
}

#[derive(Default)]
pub struct UnitVisual {
    pub gen: u32,
    pub animator: Animator,
    /// Seconds since the unit died (death clip holds its last frame).
    pub dead_for: f32,
    /// The sim tick of the unit's last shot: its muzzle flash's clock.
    pub shot_tick: Option<u64>,
    /// The weapon class last seen in the unit's hands and the set it picks.
    set: Option<(usize, usize)>,
    /// Card VTURG: a seated gunner's world matrix this frame; `None` when he
    /// is not drawn in a seat.
    seat_model: Option<Mat4>,
    seat_clock: Option<SeatClock>,
}

/// What a seated gunner keeps from one sim tick to the next: the tick, the
/// look's yaw then, the turn's rate over its speed (what a full-circle
/// nine-pose seat's yaw number is made of, `turret_pose::nine_pose`) and the hang parent's matrix
/// of the update before and of this one (an aimer-node seat hangs from the
/// update before's, `analysis/re-turret-draw.md` 3.2).
#[derive(Clone, Copy, Debug)]
struct SeatClock {
    tick: u64,
    yaw: f32,
    stick: f32,
    prev: Mat4,
    cur: Mat4,
}

impl SeatClock {
    /// The clock of `slot` brought to `tick`: the look's yaw now, the turn
    /// speed `yaw_rate`, the hang parent's matrix `held`.
    fn advance(
        slot: &mut Option<SeatClock>,
        tick: u64,
        yaw: f32,
        yaw_rate: f32,
        held: Mat4,
    ) -> SeatClock {
        let c = slot.get_or_insert(SeatClock {
            tick,
            yaw,
            stick: 0.0,
            prev: held,
            cur: held,
        });
        if c.tick != tick {
            let turned = (yaw - c.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let secs = tick.saturating_sub(c.tick).max(1) as f32 * core::TICK_DT;
            c.stick = (turned / (secs * yaw_rate.max(1e-3))).clamp(-1.0, 1.0);
            (c.tick, c.yaw, c.prev, c.cur) = (tick, yaw, c.cur, held);
        }
        *c
    }
}

/// A seat its gunner is drawn in, as it stands this tick.
struct SeatNow<'a> {
    hang: tp::GunnerHang,
    model: Mat4,
    /// The rig's nodes now.
    now: Vec<Mat4>,
    /// The hang's parent is the seat's aimer node.
    lags: bool,
    seat: &'a crate::vehicle_class::SeatSpec,
    look: (f32, f32),
    yaw_rate: f32,
}

/// The seat of mount `m` when the PS2 draws its soldier there (card VTURG):
/// a building turret ([`crate::vehicle_class::SeatSpec::gunner_draw`]) whose
/// placed building has a rig holding the seat's `PilotPosition`. A
/// vehicle's seat is [`rider_now`]'s.
fn seat_now<'a>(
    turrets: &[tp::RiggedTurret],
    walk: &'a WalkSim,
    m: core::vehicle::Mount,
    bf2: bool,
) -> Option<SeatNow<'a>> {
    let v = walk.sim.vehicle(m.vehicle)?;
    if v.kind != core::vehicle::VehicleKind::Turret {
        return None;
    }
    let s = usize::from(m.seat);
    let seat = walk.vehicle_seats.get(v.class as usize)?.get(s)?;
    if seat.gunner_draw(bf2) != crate::vehicle_class::GunnerDraw::Drawn {
        return None;
    }
    let t = turrets.iter().find(|t| t.pos == v.pos)?;
    let hang = tp::GunnerHang::of(&t.rig, seat)?;
    let setup = walk
        .sim
        .seat_setup
        .get(v.class as usize)
        .and_then(Option::as_ref)
        .and_then(|x| x.guns.get(s))
        .and_then(Option::as_ref);
    // A bot's seat is armed on its first aim: until then the gun rests.
    let (look, _, aim) = setup.map_or(((0.0, 0.0), (0.0, 0.0), (0.0, 0.0)), |g| g.rest());
    let (look, aim) = v
        .guns
        .get(s)
        .and_then(Option::as_ref)
        .map_or((look, aim), |g| (g.look, g.aim));
    let spec = tp::PoseSpec::of(&t.rig, seat);
    Some(SeatNow {
        hang,
        model: t.model,
        now: tp::pose(&t.rig, &spec, look, aim, &[]),
        lags: hang.parent.is_some() && hang.parent == spec.aimer,
        seat,
        look,
        yaw_rate: setup.map_or(2.0, |g| g.yaw_speed()),
    })
}

/// A vehicle seat its rider is drawn in, as it stands this tick (card VRIDE).
struct RiderNow<'a> {
    /// His world matrix.
    model: Mat4,
    seat: &'a crate::vehicle_class::SeatSpec,
    look: (f32, f32),
    yaw_rate: f32,
    /// A hover pilot's numbers.
    hover: Option<tp::HoverNine>,
}

/// The seat of mount `m` on a vehicle (not a building turret) when the PS2
/// draws its soldier there (card VRIDE, `analysis/re-vehicle-riders.md` 1,
/// 2.1): `O x hang`, O the vehicle as `viewer::VehicleVisuals` draws it (a
/// walker with no rig turns with its legs).
fn rider_now<'a>(
    rigs: &[Option<std::sync::Arc<crate::viewer::walker_pose::WalkerRig>>],
    nines: &[Option<HoverClock>],
    walk: &'a WalkSim,
    m: core::vehicle::Mount,
    bf2: bool,
) -> Option<RiderNow<'a>> {
    use core::vehicle::VehicleKind;
    let v = walk.sim.vehicle(m.vehicle)?;
    if v.kind == VehicleKind::Turret {
        return None;
    }
    let s = usize::from(m.seat);
    let class = v.class as usize;
    let seat = walk.vehicle_seats.get(class)?.get(s)?;
    if seat.gunner_draw(bf2) != crate::vehicle_class::GunnerDraw::Drawn {
        return None;
    }
    let hang = walk.vehicle_riders.get(class)?.get(s)?;
    let walker = v.kind == VehicleKind::Walker;
    let rig = rigs
        .get(class)
        .and_then(Option::as_deref)
        .filter(|_| walker);
    let rot = if walker && rig.is_none() {
        core::vehicle::walker::legs_rot(v)
    } else {
        v.rot
    };
    let local = match hang {
        crate::vehicle_class::RiderHang::Bind(b) => *b,
        crate::vehicle_class::RiderHang::Node(name, b) => rig
            .and_then(|r| {
                let scale = walk
                    .sim
                    .walker_params
                    .get(class)
                    .and_then(Option::as_ref)
                    .map_or(1.0, |p| p.root_scale);
                r.node_now(&v.walker, scale, name)
            })
            .unwrap_or(*b),
    };
    let setup = walk
        .sim
        .seat_setup
        .get(class)
        .and_then(Option::as_ref)
        .and_then(|x| x.guns.get(s))
        .and_then(Option::as_ref);
    let look = v
        .guns
        .get(s)
        .and_then(Option::as_ref)
        .map(|g| g.look)
        .or_else(|| setup.map(|g| g.rest().0))
        .unwrap_or((0.0, 0.0));
    let pilot = seat.kind == crate::vehicle_class::SeatKind::Pilot;
    Some(RiderNow {
        model: Mat4::from_rotation_translation(rot, v.pos) * local,
        seat,
        look,
        yaw_rate: setup.map_or(2.0, |g| g.yaw_speed()),
        hover: nines
            .get(m.vehicle.index as usize)
            .copied()
            .flatten()
            .filter(|c| pilot && v.kind == VehicleKind::Hover && c.gen == v.gen)
            .map(|c| c.nine),
    })
}

/// Per-class soldier assets. `assets` is the BODY cache, one upload per
/// soldier geometry, and `weapons` the hand-weapon cache, one per weapon
/// geometry — so two classes (or two teams) that share a model share one
/// upload, and two classes that share a body still hold their own guns.
/// `per_class[team][class]` is the pair of indices `UnitSpec::class` resolves
/// to.
///
/// A class whose GLB is not baked falls back to its own team's FIRST class's
/// body (the fill trooper) rather than to the marker box — so a mixed roster
/// looks like a team, not like a team with holes — and keeps its own weapon.
/// The fallback is printed once per missing model; baking the rest
/// (`scripts/rebake.sh`) makes it stop.
#[derive(Default)]
pub struct SoldierVisuals {
    pub assets: Vec<SoldierAsset>,
    pub weapons: Vec<WeaponAsset>,
    /// `per_class[team][class]` -> indices into `assets` and `weapons`; a
    /// `None` body = marker box.
    pub per_class: [Vec<ClassDraw>; 2],
    /// `anim[team][class]`: what picks each class's animation sets.
    pub anim: [Vec<ClassAnim>; 2],
    pub units: Vec<UnitVisual>,
    /// The map's game's muzzle flash model (card FXMF); `None` = none baked.
    pub flash: Option<FlashModel>,
    /// The map's rigged armed buildings: where a seated gunner hangs (card
    /// VTURG; [`SoldierVisuals::set_turrets`]). Empty = no gunner is drawn.
    pub turrets: Vec<tp::RiggedTurret>,
    /// The map is a BF2 disc's (BF2 also hides a gunner on
    /// `IsPilotExposed = 0`).
    pub bf2: bool,
    /// Each vehicle class's walker rig, by `Vehicle::class` (card VRIDE;
    /// [`SoldierVisuals::set_walker_rigs`]): an `AnimatedPilotPosition`
    /// rider hangs from its node now. Empty = every walker drawn rigid.
    pub walker_rigs: Vec<Option<std::sync::Arc<crate::viewer::walker_pose::WalkerRig>>>,
    /// Per `Sim::vehicles` slot: a hover's rider numbers (card VRIDE).
    hover_nine: Vec<Option<HoverClock>>,
}

/// A hover's [`tp::HoverNine`] and the vehicle and tick it was stepped for.
#[derive(Clone, Copy, Debug)]
struct HoverClock {
    gen: u32,
    tick: u64,
    nine: tp::HoverNine,
}

/// Seconds a corpse stays drawn, counted from the death (`dead_for`), unless
/// the slot respawns first. PROVISIONAL and ours: BF1's stance table says
/// "dead/dying … after 4 s → corpse cleanup" (`analysis/re-stance-roll.md`,
/// stance 3); adopting it would move captures, not the sim.
const CORPSE_SECS: f32 = 6.0;

impl SoldierVisuals {
    /// Load every class model both teams name, from
    /// `assets_dir/side_<game>/soldiers/`. Classes whose GLB is missing take
    /// their team's first class's asset; a team with no model at all keeps
    /// the marker box.
    ///
    /// `game` is the MAP's source game — the same disc `game::roster` took
    /// the classes off, so the `GeometryName` a class carries is looked up in
    /// the side bake that holds it. Both discs bake all four sides, and the
    /// faction's OWN game is the wrong answer for a BF2 GCW map or a BF1 CW
    /// one: over the 322 class slots on such (map, faction) pairs the model
    /// resolves 309 times in the map's own side dir and 225 times in the
    /// faction's native one (the 13 that miss either way are the droideka's
    /// `cis_walk_droideka`, and the droideka is not a fieldable class).
    pub fn load(
        r: &mut Renderer,
        assets_dir: &Path,
        game: &str,
        teams: &[TeamTemplate; 2],
    ) -> Self {
        let plan = DrawPlan::of(teams);
        let mut cache: HashMap<String, Option<TextureHandle>> = HashMap::new();
        let bodies = plan
            .bodies
            .iter()
            .map(|geometry| Self::load_body(Some(&mut *r), assets_dir, game, geometry, &mut cache))
            .collect();
        let weapons: Vec<WeaponAsset> = plan
            .weapons
            .iter()
            .map(|wg| Self::load_weapon(r, assets_dir, game, wg, &mut cache))
            .collect();
        let flash = FlashModel::load(r, assets_dir, game);
        let mut s = Self::assemble(plan, teams, bodies, weapons, flash);
        s.bf2 = game.eq_ignore_ascii_case("bf2");
        s
    }

    /// The map's rigged armed buildings (`turret_pose::rigged_turrets`):
    /// a gunner seated in one is drawn at his seat (card VTURG).
    pub fn set_turrets(&mut self, turrets: Vec<tp::RiggedTurret>) {
        self.turrets = turrets;
    }

    /// Each vehicle class's walker rig (`viewer::walker_rigs`): a rider on
    /// an animated node rides it as it walks (card VRIDE).
    pub fn set_walker_rigs(
        &mut self,
        rigs: Vec<Option<std::sync::Arc<crate::viewer::walker_pose::WalkerRig>>>,
    ) {
        self.walker_rigs = rigs;
    }

    /// Bring every hover's rider numbers to this tick: one drive update of
    /// the tick's length per tick while it has a pilot, with his input
    /// (`tp::HoverNine::step`); a fresh vehicle starts at the PS2's -1.
    fn step_hover_nines(&mut self, walk: &WalkSim) {
        let tick = walk.sim.tick;
        let vs = &walk.sim.vehicles;
        self.hover_nine.resize(vs.len(), None);
        for (slot, v) in vs.iter().enumerate() {
            if !v.alive || v.kind != core::vehicle::VehicleKind::Hover {
                self.hover_nine[slot] = None;
                continue;
            }
            let fresh = HoverClock {
                gen: v.gen,
                tick,
                nine: tp::HoverNine::START,
            };
            let c = self.hover_nine[slot].get_or_insert(fresh);
            if c.gen != v.gen {
                *c = fresh;
            }
            let piloted = v.pilot_seat().is_some_and(|s| s.occupant.is_some());
            if piloted {
                let no_strafe = walk
                    .sim
                    .hover_params
                    .get(v.class as usize)
                    .and_then(Option::as_ref)
                    .is_some_and(|p| p.strafe_speed < 1e-4);
                let i = v.input;
                for _ in 0..tick.saturating_sub(c.tick).min(30) {
                    c.nine = c.nine.step(
                        [i.throttle, i.strafe, i.turn],
                        no_strafe,
                        self.bf2,
                        core::TICK_DT,
                    );
                }
            }
            c.tick = tick;
        }
    }

    /// The animators without a renderer (a census, a test): every body's
    /// skeleton, clips and sets; nothing to draw.
    pub fn load_cpu(assets_dir: &Path, game: &str, teams: &[TeamTemplate; 2]) -> Self {
        let plan = DrawPlan::of(teams);
        let mut cache = HashMap::new();
        let bodies = plan
            .bodies
            .iter()
            .map(|geometry| Self::load_body(None, assets_dir, game, geometry, &mut cache))
            .collect();
        let mut s = Self::assemble(plan, teams, bodies, Vec::new(), None);
        s.bf2 = game.eq_ignore_ascii_case("bf2");
        s
    }

    /// `bodies` per `plan.bodies` entry, `None` for one that is not baked.
    fn assemble(
        plan: DrawPlan,
        teams: &[TeamTemplate; 2],
        bodies: Vec<Option<SoldierAsset>>,
        weapons: Vec<WeaponAsset>,
        flash: Option<FlashModel>,
    ) -> Self {
        // A body that is not baked leaves no asset: plan index -> `assets`.
        let mut assets: Vec<SoldierAsset> = Vec::new();
        let bodies: Vec<Option<usize>> = bodies
            .into_iter()
            .map(|a| {
                assets.push(a?);
                Some(assets.len() - 1)
            })
            .collect();
        let mut per_class = plan.per_class;
        for (i, t) in teams.iter().enumerate() {
            for c in per_class[i].iter_mut() {
                c.body = c.body.and_then(|b| bodies[b]);
            }
            // A class with no model of its own borrows the team's FIRST
            // class's; only a team whose first class has none keeps the box.
            let fallback = per_class[i].first().and_then(|c| c.body);
            for (k, c) in per_class[i].iter_mut().enumerate() {
                if c.body.is_none() && fallback.is_some() {
                    println!(
                        "soldier {}: no baked model — drawing {} instead",
                        t.classes[k].name, t.classes[0].name
                    );
                    c.body = fallback;
                }
            }
        }
        Self {
            assets,
            weapons,
            per_class,
            anim: teams
                .each_ref()
                .map(|t| t.classes.iter().map(|c| c.anim.clone()).collect()),
            units: Vec::new(),
            flash,
            turrets: Vec::new(),
            bf2: false,
            walker_rigs: Vec::new(),
            hover_nine: Vec::new(),
        }
    }

    /// A part's texture from `side/textures/`, uploaded once per name.
    fn texture(
        r: &mut Renderer,
        side: &Path,
        material: Option<&str>,
        cache: &mut HashMap<String, Option<TextureHandle>>,
    ) -> Option<TextureHandle> {
        let key = material?.to_ascii_lowercase();
        if !cache.contains_key(&key) {
            let h = crate::viewer::upload_texture_file(r, &side.join("textures"), &key)
                .unwrap_or_else(|e| {
                    eprintln!("{e:#}");
                    None
                });
            cache.insert(key.clone(), h);
        }
        cache[&key]
    }

    /// One skinned soldier model, or `None` if it is not baked. Without a
    /// renderer it has no meshes.
    fn load_body(
        r: Option<&mut Renderer>,
        assets_dir: &Path,
        game: &str,
        geometry: &str,
        cache: &mut HashMap<String, Option<TextureHandle>>,
    ) -> Option<SoldierAsset> {
        // The side bake knows no map, so a BF1 part's sun-coloured
        // specular pass resolves against the lighting the map's
        // `upload_scene` already put on the renderer: every caller loads
        // the soldiers after the scene.
        let side = assets_dir.join(format!("side_{game}"));
        let glb = side.join("soldiers").join(format!("{geometry}.glb"));
        let asset = match load_skinned_glb(&glb) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("soldier {geometry}: {e:#}");
                return None;
            }
        };
        let mut meshes = Vec::new();
        if let Some(r) = r {
            match crate::viewer::upload_skinned(r, &asset.mesh) {
                Ok(parts) => {
                    for (k, (mesh, mat, layer)) in parts.into_iter().enumerate() {
                        let tex = Self::texture(r, &side, mat, cache);
                        meshes.push((mesh, tex, layer, part_spec(r, asset.mesh.spec_pass(k))));
                    }
                }
                Err(e) => {
                    eprintln!("soldier {geometry}: {e:#}");
                    return None;
                }
            }
        }
        let hp_weapons = asset.skeleton.index_of("hp_weapons");
        let rig = Self::load_rig(&side, geometry, &asset);
        println!(
            "soldier {geometry}: {} bones, {} clips, {} parts",
            asset.skeleton.len(),
            asset.clips.len(),
            meshes.len()
        );
        Some(SoldierAsset {
            asset,
            meshes,
            rig,
            hp_weapons,
        })
    }

    /// A baked soldier's animation sets over its skeleton and clips.
    pub fn load_rig(side: &Path, geometry: &str, asset: &SkinnedAsset) -> Option<Rig> {
        let path = manifest::anim_sets::path(side, geometry);
        let sets = pak::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|t| ron::from_str(&t).map_err(|e| e.to_string()));
        match sets {
            Ok(sets) => Some(Rig::new(sets, &asset.skeleton, &asset.clips)),
            Err(e) => {
                eprintln!("soldier {geometry}: {}: {e} (not animated)", path.display());
                None
            }
        }
    }

    /// One hand weapon: a static GLB under weapons/, textures beside, and its
    /// nodes. A model that is not baked gives no parts (no weapon drawn).
    fn load_weapon(
        r: &mut Renderer,
        assets_dir: &Path,
        game: &str,
        wg: &str,
        cache: &mut HashMap<String, Option<TextureHandle>>,
    ) -> WeaponAsset {
        let side = assets_dir.join(format!("side_{game}"));
        let mut parts = Vec::new();
        let wpath = side.join("weapons").join(format!("{wg}.glb"));
        match crate::assets::load_glb(&wpath) {
            Ok(cpu) => match crate::viewer::upload_submeshes(r, &cpu) {
                Ok(up) => {
                    for (k, (mesh, mat, layer, _)) in up.into_iter().enumerate() {
                        let tex = Self::texture(r, &side, mat, cache);
                        parts.push((mesh, tex, layer, part_spec(r, cpu.spec_pass(k))));
                    }
                }
                Err(e) => eprintln!("weapon {wg}: {e:#}"),
            },
            Err(e) => eprintln!("weapon {wg}: {e:#} (no weapon drawn)"),
        }
        let nodes = pak::read_to_string(manifest::weapon_nodes_path(&side, wg))
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        println!("weapon {wg}: {} parts", parts.len());
        WeaponAsset { parts, nodes }
    }

    /// What a unit's class draws: its own pair, or the team's first-class
    /// body fallback that `load` resolved for it.
    fn class_draw(&self, team: u8, class: u8) -> ClassDraw {
        let row = &self.per_class[usize::from(team.min(1))];
        let i = usize::from(class).min(row.len().saturating_sub(1));
        row.get(i).copied().unwrap_or_default()
    }

    /// Index into `assets` for a unit's class.
    fn asset_index(&self, team: u8, class: u8) -> Option<usize> {
        self.class_draw(team, class).body
    }

    /// The asset a unit draws with.
    fn asset_for(&self, team: u8, class: u8) -> Option<&SoldierAsset> {
        self.assets.get(self.asset_index(team, class)?)
    }

    /// The hand weapon a unit's class holds, if it has one.
    fn weapon_for(&self, team: u8, class: u8) -> Option<&WeaponAsset> {
        self.weapons.get(self.class_draw(team, class).weapon?)
    }

    pub fn has_model(&self, team: u8) -> bool {
        self.per_class[usize::from(team.min(1))]
            .iter()
            .any(|c| c.body.is_some())
    }

    /// The tick the sim just stepped: who fired (the muzzle flash's clock).
    /// [`SoldierVisuals::update`] reads the last tick's shots itself; a frame
    /// that steps several ticks calls this after each
    /// (`WalkSim::advance_with`) so an earlier tick's shot still flashes.
    pub fn note_shots(&mut self, walk: &WalkSim) {
        for e in &walk.sim.events {
            let core::sim::Event::Shot { unit, .. } = e else {
                continue;
            };
            let (Some(v), Some(u)) = (
                self.units.get_mut(unit.index as usize),
                walk.sim.units.get(unit.index as usize),
            ) else {
                continue;
            };
            if v.gen == u.gen {
                v.shot_tick = Some(walk.sim.tick);
            }
        }
    }

    /// Advance every unit's animator from the sim state (`dt` = real seconds).
    pub fn update(&mut self, walk: &WalkSim, dt: f32) {
        self.update_with(walk, dt, None);
    }

    /// [`SoldierVisuals::update`], counting what each animator holds.
    pub fn update_with(
        &mut self,
        walk: &WalkSim,
        dt: f32,
        mut census: Option<&mut crate::animator::census::Census>,
    ) {
        let n = walk.sim.units.len();
        if self.units.len() < n {
            self.units.resize_with(n, UnitVisual::default);
        }
        self.step_hover_nines(walk);
        let fired: Vec<u32> = walk
            .sim
            .events
            .iter()
            .filter_map(|e| match e {
                core::sim::Event::Shot { unit, .. } => Some(unit.index),
                _ => None,
            })
            .collect();
        for (i, u) in walk.sim.units.iter().enumerate() {
            // Split the borrows by FIELD: the asset lives in `self.assets`,
            // the animator in `self.units`.
            let Some(ai) = self.asset_index(u.team, u.spec.class) else {
                continue;
            };
            let sa = &self.assets[ai];
            let v = &mut self.units[i];
            if v.gen != u.gen {
                *v = UnitVisual {
                    gen: u.gen,
                    ..Default::default()
                };
            }
            if !u.alive {
                v.dead_for += dt;
            }
            if fired.contains(&(i as u32)) {
                v.shot_tick = Some(walk.sim.tick);
            }
            // Seated in a turret that shows him (card VTURG) or on a
            // vehicle that does (card VRIDE): hung at the seat's hardpoint,
            // posed by the seat's clip.
            v.seat_model = None;
            let tick = walk.sim.tick;
            // A full-circle seat's yaw number: BF2 the yaw rate over the
            // turn speed (B2:189542-189546), BF1 the raw stick
            // (B1:137715-137719, T+0x78), which turns the yaw against its
            // sign on the PS2 as the port's `rx` does: -rate / speed.
            let bf2 = self.bf2;
            let turn = |c: SeatClock| if bf2 { c.stick } else { -c.stick };
            let rider = u
                .mount
                .filter(|_| u.alive)
                .and_then(|m| rider_now(&self.walker_rigs, &self.hover_nine, walk, m, self.bf2));
            if let Some(r) = rider {
                let c = SeatClock::advance(
                    &mut v.seat_clock,
                    tick,
                    r.look.0,
                    r.yaw_rate,
                    Mat4::IDENTITY,
                );
                v.seat_model = Some(r.model);
                let Some(pose) = &r.seat.pilot_pose else {
                    continue;
                };
                if let Some(clip) = crate::animator::find_clip(&sa.asset.clips, &pose.clip()) {
                    let nine = pose.nine.then(|| match r.hover {
                        Some(h) => h.pose(),
                        None => tp::nine_pose(
                            -r.look.1,
                            r.look.0,
                            turn(c),
                            r.seat.pitch_limits,
                            r.seat.yaw_limits,
                        ),
                    });
                    v.animator
                        .seat(&sa.asset.clips, &sa.asset.skeleton, clip, nine, dt);
                }
                continue;
            }
            let seated = u
                .mount
                .filter(|_| u.alive)
                .and_then(|m| seat_now(&self.turrets, walk, m, self.bf2));
            let Some(s) = seated else {
                v.seat_clock = None;
                let Some(rig) = &sa.rig else { continue };
                Self::step_on_foot(
                    walk,
                    &self.anim,
                    i,
                    u,
                    sa,
                    rig,
                    v,
                    &fired,
                    dt,
                    census.as_deref_mut(),
                );
                continue;
            };
            let held = s.hang.parent.map_or(Mat4::IDENTITY, |p| s.now[p]);
            let c = SeatClock::advance(&mut v.seat_clock, tick, s.look.0, s.yaw_rate, held);
            let mut now = s.now;
            if let (true, Some(p)) = (s.lags, s.hang.parent) {
                now[p] = c.prev;
            }
            v.seat_model = Some(s.hang.matrix(s.model, &now));
            let Some(pose) = &s.seat.pilot_pose else {
                continue;
            };
            if let Some(clip) = crate::animator::find_clip(&sa.asset.clips, &pose.clip()) {
                let nine = pose.nine.then(|| {
                    tp::nine_pose(
                        -s.look.1,
                        s.look.0,
                        turn(c),
                        s.seat.pitch_limits,
                        s.seat.yaw_limits,
                    )
                });
                v.animator
                    .seat(&sa.asset.clips, &sa.asset.skeleton, clip, nine, dt);
            }
        }
    }

    /// One unit's animator on foot (not in a seat that shows him).
    #[allow(clippy::too_many_arguments)]
    fn step_on_foot(
        walk: &WalkSim,
        anim: &[Vec<ClassAnim>; 2],
        i: usize,
        u: &core::unit::Unit,
        sa: &SoldierAsset,
        rig: &Rig,
        v: &mut UnitVisual,
        fired: &[u32],
        dt: f32,
        census: Option<&mut crate::animator::census::Census>,
    ) {
        // The weapon in hand (channel 1's while it has the soldier) picks
        // the set.
        let held = (u.active_channel == 1)
            .then(|| u.held.get(usize::from(u.cur1?)))
            .flatten();
        let (wc, weapon) = held.map_or((u.weapon_class, &u.weapon), |h| (h.class, &h.state));
        let class = walk.sim.weapon_classes.get(wc);
        let set = match v.set {
            Some((c, set)) if c == wc => set,
            _ => {
                let team = &anim[usize::from(u.team.min(1))];
                let anim = team.get(usize::from(u.spec.class).min(team.len().saturating_sub(1)));
                let bank = anim.zip(class).map_or("rifle", |(a, w)| a.bank_of(&w.name));
                let set = rig.weapon_set(anim.map_or("human", |a| &a.family), bank);
                v.set = Some((wc, set));
                set
            }
        };
        let s = &u.soldier;
        let input = AnimInput {
            stance: s.stance,
            vel: crate::animator::body_vel(s.vel, s.yaw),
            yaw: s.yaw,
            max_speed: u.consts.max_speed,
            dead: !u.alive,
            set,
            fired: u.mount.is_none() && fired.contains(&(i as u32)),
            reloading: weapon.reloading(),
            reload_time: class.map_or(0.0, |c| c.reload_time),
            // PROVISIONAL and ours (FEEL_NOTES; the PS2's two soldier
            // flags are unread): he fires, holds the trigger (a charge
            // too), or has a target (a bot's; the player's aim magnet).
            alert: u.mount.is_none()
                && u.alive
                && (fired.contains(&(i as u32))
                    || weapon.fire_cmd > 0
                    || u.ai.target.is_some()
                    || u.spec.player_slot.is_some_and(|p| {
                        walk.sim
                            .magnet
                            .get(usize::from(p))
                            .is_some_and(Option::is_some)
                    })),
            // The body faces the aim's yaw; the animator takes the legs'
            // twist off it.
            aim: (0.0, s.pitch),
        };
        v.animator
            .step(rig, &sa.asset.clips, &sa.asset.skeleton, &input, dt);
        if let Some(c) = census {
            if u.alive && u.mount.is_none() {
                c.note(rig, &sa.asset.clips, &v.animator, &input);
                c.note_aim(rig, &sa.asset.skeleton, sa.hp_weapons, &v.animator, &input);
            }
        }
    }

    /// Units to draw this frame: (unit index, asset, model matrix, palette).
    fn posed(&self, walk: &WalkSim) -> Vec<(usize, &SoldierAsset, Mat4, Vec<Mat4>)> {
        let mut out = Vec::new();
        for (i, u) in walk.sim.units.iter().enumerate() {
            let Some(sa) = self.asset_for(u.team, u.spec.class) else {
                continue;
            };
            let Some(v) = self.units.get(i) else { continue };
            if v.gen != u.gen || (!u.alive && v.dead_for > CORPSE_SECS) {
                continue;
            }
            if !u.alive && !v.animator.posed() {
                continue;
            }
            // A rider is hidden but in a seat that shows him (cards VTURG,
            // VRIDE), who is drawn in every view, his own too.
            let model = match (u.mount, v.seat_model) {
                (Some(_), Some(m)) => m,
                (Some(_), None) => continue,
                // In the scope view the player's own body is not drawn.
                (None, _) if i == walk.player.index as usize && walk.player_body_hidden() => {
                    continue
                }
                (None, _) => model_of(&u.soldier),
            };
            let pose = v.animator.pose(&sa.asset.skeleton);
            let palette = sa.asset.skeleton.palette(&pose.locals());
            out.push((i, sa, model, palette));
        }
        out
    }

    /// Every unit drawn this frame and its model matrix (a test's and a
    /// census's read of who is drawn where).
    pub fn drawn(&self, walk: &WalkSim) -> Vec<(usize, Mat4)> {
        self.posed(walk)
            .into_iter()
            .map(|(i, _, m, _)| (i, m))
            .collect()
    }

    /// The hand weapon of every drawn unit, at the posed `hp_weapons` bone:
    /// `unit × palette[hp] × bind_global[hp]` (palette = global × inverse bind).
    pub fn weapon_draws(&self, walk: &WalkSim) -> Vec<render::DrawCmd> {
        let mut out = Vec::new();
        for (i, sa, model, palette) in self.posed(walk) {
            let Some(hp) = sa.hp_weapons else { continue };
            let u = &walk.sim.units[i];
            // A seated gunner holds no hand weapon (B1:114261-114266,
            // B2:155344-155345).
            if u.mount.is_some() {
                continue;
            }
            let Some(weapon) = self.weapon_for(u.team, u.spec.class) else {
                continue;
            };
            let m = model * palette[hp] * sa.asset.skeleton.global_bind[hp];
            for (mesh, tex, layer, spec) in &weapon.parts {
                // The weapon is a non-static draw with no shadow volume: the
                // soldier's own character ambient on BF1 (re-model-lighting §12).
                let light = render::LightPath::Character;
                out.push(
                    match tex {
                        Some(t) if *layer == render::DrawLayer::Mask => {
                            render::DrawCmd::textured_masked(*mesh, *t, m)
                        }
                        Some(t) => render::DrawCmd::textured(*mesh, *t, m),
                        None => render::DrawCmd::flat(*mesh, [0.3, 0.3, 0.32, 1.0], m),
                    }
                    .with_light(light),
                );
                // The second pass rides only a textured draw: its add is
                // weighted by the base texel's alpha (as the world's).
                if let (Some(t), Some((rgb, power))) = (tex, spec) {
                    out.push(
                        render::DrawCmd::specular(*mesh, *t, *rgb, *power, m).with_light(light),
                    );
                }
            }
        }
        out
    }

    /// Every drawn unit's weapon fire point in the world, by unit index:
    /// `hp_weapons` × its weapon's `FirePointName` node (the `hp_weapons`
    /// origin when the weapon model has no such node, §4.1). `None` for a
    /// unit drawn without a model or a hand node. Where the muzzle flash
    /// sits and the drawn bolt starts (`crate::bolt_vis`).
    pub fn fire_points(&self, walk: &WalkSim) -> Vec<Option<Vec3>> {
        let mut out = vec![None; walk.sim.units.len()];
        for (i, sa, model, palette) in self.posed(walk) {
            if let (Some(hp), None) = (sa.hp_weapons, walk.sim.units[i].mount) {
                let u = &walk.sim.units[i];
                let name = walk
                    .sim
                    .weapon_classes
                    .get(u.weapon_class)
                    .and_then(|w| w.flash.as_ref())
                    .map_or(core::flash::DEFAULT_FIRE_POINT, |f| f.fire_point.as_str());
                let hand = model * palette[hp] * sa.asset.skeleton.global_bind[hp];
                out[i] = Some(match self.weapon_for(u.team, u.spec.class) {
                    Some(w) => fire_point(hand, &w.nodes, name),
                    None => hand.transform_point3(Vec3::ZERO),
                });
            }
        }
        out
    }

    /// The muzzle flashes to draw this frame (see the module doc): one per
    /// drawn unit whose weapon flashes and whose last shot's t is still > 0
    /// on the sim clock.
    pub fn flash_draws(&self, walk: &WalkSim) -> Vec<ParticleDraw> {
        let Some(model) = &self.flash else {
            return Vec::new();
        };
        let now = walk.sim.tick as f64 * f64::from(core::TICK_DT);
        let points = self.fire_points(walk);
        let mut out = Vec::new();
        for (i, u) in walk.sim.units.iter().enumerate() {
            let (Some(v), Some(Some(p))) = (self.units.get(i), points.get(i)) else {
                continue;
            };
            let Some(shot) = v.shot_tick else { continue };
            let Some(look) = walk
                .sim
                .weapon_classes
                .get(u.weapon_class)
                .and_then(|w| w.flash.as_ref())
                .filter(|f| f.draws() && f.model.as_deref() == Some(model.name.as_str()))
            else {
                continue;
            };
            let t = look.t(shot as f64 * f64::from(core::TICK_DT), now);
            let Some(color) = look.color_at(t) else {
                continue;
            };
            let m = core::flash::matrix(u.aim_frame().0, look.radius, *p);
            out.extend(model.draws(m, color, *p));
        }
        out
    }

    /// Skinned draws for every unit with a model (alive, or a fresh corpse),
    /// each part's specular second pass beside its base.
    /// Soldier models face +Z; sim yaw 0 faces −Z, hence the extra π.
    pub fn draws(&self, walk: &WalkSim) -> Vec<SkinnedDrawCmd> {
        let mut out = Vec::new();
        for (_, sa, model, palette) in self.posed(walk) {
            for (mesh, tex, layer, spec) in &sa.meshes {
                let base = SkinnedDrawCmd {
                    mesh: *mesh,
                    texture: *tex,
                    color: [1.0; 4],
                    model,
                    palette: palette.clone(),
                    layer: *layer,
                    // BF2's local lights reach the soldier (card BPNT).
                    lit: Some(crate::point_light::soldier_sphere(
                        model.transform_point3(glam::Vec3::ZERO),
                    )),
                };
                out.extend(with_spec(base, *spec));
            }
        }
        out
    }
}

/// The world point of the weapon node `name` under the hand matrix `hand`
/// (`hp_weapons` in the world); the hand's origin when the weapon has no
/// such node — what the PS2's class+0x30 is then (§4.1).
pub fn fire_point(hand: Mat4, nodes: &manifest::WeaponNodes, name: &str) -> Vec3 {
    let local = nodes
        .iter()
        .find(|n| n.name.eq_ignore_ascii_case(name))
        .map_or(Vec3::ZERO, |n| Vec3::from(n.pos));
    hand.transform_point3(local)
}

/// A soldier's model matrix: at his feet, turned to his yaw (the model faces
/// +Z, sim yaw 0 faces -Z; `animator::travel` is measured in this frame).
fn model_of(s: &core::soldier::Soldier) -> Mat4 {
    Mat4::from_translation(s.pos) * Mat4::from_rotation_y(s.yaw + std::f32::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(classes: &[(&str, Option<&str>, Option<&str>)]) -> TeamTemplate {
        let mut t = TeamTemplate::default_trooper();
        t.classes = classes
            .iter()
            .map(|(name, geometry, weapon)| crate::walk::ClassVisual {
                name: (*name).into(),
                label: (*name).into(),
                geometry: geometry.map(str::to_owned),
                weapon_geometry: weapon.map(str::to_owned),
                weapon_hud: Default::default(),
                held_hud: Vec::new(),
                foley_class: 0,
                cries: Default::default(),
                anim: Default::default(),
            })
            .collect();
        t
    }

    /// Card UWPN: BF1's Clone Wars rosters. The sharpshooter shares the
    /// trooper's body and the pilot and assassin droids the assault droid's;
    /// each still draws its own primary's model.
    #[test]
    fn classes_sharing_a_body_hold_their_own_guns() {
        let rep = team(&[
            (
                "rep_inf_clone_trooper",
                Some("rep_inf_trooper"),
                Some("rep_weap_inf_rifle"),
            ),
            (
                "rep_inf_clone_pilot",
                Some("rep_inf_pilot"),
                Some("rep_weap_inf_rifle"),
            ),
            (
                "rep_inf_clone_sharpshooter",
                Some("rep_inf_trooper"),
                Some("rep_weap_inf_sniperrifle"),
            ),
        ]);
        let cis = team(&[
            ("cis_inf_battledroid", Some("cis_inf_sbdroid"), None),
            (
                "cis_inf_assault",
                Some("cis_inf_bdroid"),
                Some("cis_weap_inf_launcher"),
            ),
            (
                "cis_inf_pilotdroid",
                Some("cis_inf_bdroid"),
                Some("com_weap_inf_torpedo"),
            ),
            (
                "cis_inf_assassindroid",
                Some("cis_inf_bdroid"),
                Some("cis_weap_inf_sniperrifle"),
            ),
        ]);
        let plan = DrawPlan::of(&[rep, cis]);
        // One upload per distinct model.
        assert_eq!(plan.bodies.len(), 4, "{:?}", plan.bodies);
        assert_eq!(plan.weapons.len(), 5, "{:?}", plan.weapons);
        let [r, c] = &plan.per_class;
        // Same body, different gun.
        assert_eq!(r[0].body, r[2].body);
        assert_ne!(r[0].weapon, r[2].weapon);
        assert_eq!(plan.weapon_of(0, 2), Some("rep_weap_inf_sniperrifle"));
        assert_eq!(c[1].body, c[2].body);
        assert_eq!(c[1].body, c[3].body);
        assert_eq!(plan.weapon_of(1, 1), Some("cis_weap_inf_launcher"));
        assert_eq!(plan.weapon_of(1, 2), Some("com_weap_inf_torpedo"));
        assert_eq!(plan.weapon_of(1, 3), Some("cis_weap_inf_sniperrifle"));
        // Different body, same gun: one weapon upload.
        assert_ne!(r[0].body, r[1].body);
        assert_eq!(r[0].weapon, r[1].weapon);
        // The super battle droid's wrist blaster has no model: none drawn.
        assert_eq!(c[0].weapon, None);
    }

    #[test]
    fn the_fire_point_is_the_node_under_the_hand() {
        let nodes = vec![manifest::ManifestHardpoint {
            name: "hp_fire".into(),
            pos: [0.0, 0.0533, 1.097],
            axes: None,
        }];
        let hand = Mat4::from_translation(Vec3::new(10.0, 1.0, 0.0))
            * Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let p = fire_point(hand, &nodes, "HP_FIRE");
        assert!(p.abs_diff_eq(Vec3::new(11.097, 1.0533, 0.0), 1e-5), "{p}");
        // No such node: the hand's origin.
        assert_eq!(
            fire_point(hand, &nodes, "hp_other"),
            Vec3::new(10.0, 1.0, 0.0)
        );
    }

    /// Card ROLLS: a roll to a unit's side - the player's and a bot's, through
    /// the input this module fills from the sim - turns the DRAWN body so its
    /// dive points along the velocity, on a real soldier of each game.
    #[test]
    fn a_side_roll_turns_the_drawn_body_onto_the_velocity() {
        use core::soldier::Stance;
        let root = crate::assets::baked_root();
        for game in ["bf1", "bf2"] {
            let geometry = "rep_inf_trooper";
            if !root
                .join(format!("side_{game}/soldiers/{geometry}.glb"))
                .exists()
            {
                eprintln!("skip: no {game} {geometry} bake");
                continue;
            }
            let t = team(&[("trooper", Some(geometry), None)]);
            let mut vis = SoldierVisuals::load_cpu(&root, game, &[t.clone(), t]);
            let mut walk = WalkSim::new(7, Vec3::ZERO, None);
            let bot = walk.sim.spawn(&core::unit::UnitSpec {
                controller: core::unit::Controller::Ai,
                ..core::unit::UnitSpec::trooper(1, Vec3::new(6.0, 0.0, -10.0))
            });
            let player = walk.player.index as usize;
            let cases = [
                (player, 0.7f32, 1.0f32),
                (bot.expect("spawned").index as usize, -2.0, -1.0),
            ];
            for &(i, yaw, side) in &cases {
                let s = &mut walk.sim.units[i].soldier;
                s.yaw = yaw;
                s.stance = Stance::Roll;
                // His left (side +1) or right: the sim's `right` negated.
                let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
                s.vel = -right * side * 10.5;
            }
            for n in 1..=26 {
                vis.update(&walk, core::TICK_DT);
                for &(i, _, side) in &cases {
                    let a = &vis.units[i].animator;
                    let tw = a.twist().to_degrees();
                    assert!(tw * side > 0.0, "{game} unit {i} update {n}: {tw}");
                    if n >= 9 {
                        assert!(
                            (tw - 90.0 * side).abs() < 15.0,
                            "{game} unit {i} update {n}: {tw}"
                        );
                    }
                }
            }
            let drawn = vis.posed(&walk);
            assert_eq!(drawn.len(), 2, "{game}: both drawn");
            for (i, sa, model, palette) in drawn {
                let skel = &sa.asset.skeleton;
                let b = skel.index_of("DummyRoot").expect("a DummyRoot");
                assert_eq!(skel.parents[b], None, "{game}: the root has no parent");
                // The drawn root (no parent: its global is its local) carries
                // the kept pose's model +Z to where the dive points.
                let kept = vis.units[i].animator.kept_pose(skel).0[b].rotation;
                let v = kept.inverse() * Vec3::Z;
                let dive = (model * palette[b] * skel.global_bind[b]).transform_vector3(v);
                let heading = |v: Vec3| v.x.atan2(v.z);
                let vel = walk.sim.units[i].soldier.vel;
                let off = (heading(dive) - heading(vel) + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                assert!(
                    off.to_degrees().abs() < 2.0,
                    "{game} unit {i}: {}",
                    off.to_degrees()
                );
            }
        }
    }

    /// Card ROLLS acceptance, the player's half: the pad's roll in each
    /// direction through the real sim (BF1 and BF2 ground rules, a real
    /// trooper of each game), travel less twist per update beside the PS2's
    /// rows (re-roll-picture 4): within 15 degrees from the 9th update, 2 at
    /// the end, a forward roll within 1 of 0; the strafe before a side roll
    /// twists the legs +-71.6 (card ANIMM: BF1 on the backward clip, BF2 the
    /// forward), so BF1's first update reads the PS2's -31.2 / +31.1, and
    /// 0.5 s after the roll the twist is back within 10 of the strafe's.
    /// `--nocapture` prints the rows.
    #[test]
    fn the_players_rolls_turn_the_body_as_the_ps2s() {
        use core::heightfield::Heightfield;
        use core::sim::{buttons, PadState};
        use core::soldier::{GroundRules, Stance};
        let root = crate::assets::baked_root();
        let flat = || Heightfield {
            size: 2,
            unit: 4000.0,
            offset_x: 2000.0,
            offset_z: 2000.0,
            heights: vec![0.0; 4],
        };
        let mut rolls = 0;
        for (game, rules) in [("bf1", GroundRules::Bf1), ("bf2", GroundRules::Bf2)] {
            let geometry = "rep_inf_trooper";
            if !root
                .join(format!("side_{game}/soldiers/{geometry}.glb"))
                .exists()
            {
                eprintln!("skip: no {game} {geometry} bake");
                continue;
            }
            let t = team(&[("trooper", Some(geometry), None)]);
            // (name, stick x, stick y): the pad's +y is ahead, -x his left.
            for (name, lx, ly) in [
                ("forward", 0i16, 32767i16),
                ("left", -32767, 0),
                ("right", 32767, 0),
                ("forward-left", -23170, 23170),
            ] {
                let mut vis = SoldierVisuals::load_cpu(&root, game, &[t.clone(), t.clone()]);
                let mut w = WalkSim::new(1, Vec3::ZERO, Some(flat()));
                w.sim.ground_rules = rules;
                let p = w.player.index as usize;
                let stick = PadState {
                    lx,
                    ly,
                    ..Default::default()
                };
                let press = PadState {
                    buttons: buttons::ROLL_STANCE,
                    ..stick
                };
                for _ in 0..19 {
                    w.step_tick(&[stick]);
                    vis.update(&w, core::TICK_DT);
                }
                let strafe = vis.units[p].animator.twist().to_degrees();
                let want_strafe = match (game, name) {
                    ("bf1", "left") | ("bf2", "right") => -71.6,
                    ("bf1", "right") | ("bf2", "left") => 71.6,
                    (_, "forward") => 0.0,
                    // Inside the forward window: the travel itself (the
                    // walk's, under the roll's 45).
                    _ => strafe.clamp(1.0, 44.0),
                };
                assert!(
                    (strafe - want_strafe).abs() < 1.5,
                    "{game} {name}: strafe twist {strafe}"
                );
                let mut rows = Vec::new();
                let mut after = None;
                for k in 0..90 {
                    w.step_tick(&[if k == 0 { press } else { stick }]);
                    vis.update(&w, core::TICK_DT);
                    let a = &vis.units[p].animator;
                    let r = a.roll();
                    if w.sim.units[p].soldier.stance == Stance::Roll {
                        assert!(r.updates > 0);
                        rows.push((a.travel().to_degrees(), a.twist().to_degrees()));
                    } else if r.after.is_some_and(|t| (t - 0.5).abs() < 1e-3) {
                        after = Some(a.twist().to_degrees());
                    }
                }
                assert!(
                    rows.len() >= 20,
                    "{game} {name}: {} roll updates",
                    rows.len()
                );
                rolls += 1;
                let line: Vec<String> = rows
                    .iter()
                    .map(|(t, tw)| format!("{:.1}", t - tw))
                    .collect();
                println!(
                    "{game} {name}: travel {:.1}, travel - twist by update {}",
                    rows[0].0,
                    line.join(" ")
                );
                let (travel, _) = rows[0];
                let want = match name {
                    "forward" => 0.0,
                    "left" => 90.0,
                    "right" => -90.0,
                    // The strafe is slower than the run (PS2: +37.9).
                    _ => 37.9,
                };
                assert!(
                    (travel - want).abs() < 1.0,
                    "{game} {name}: travel {travel}"
                );
                for (n, (t, tw)) in rows.iter().enumerate() {
                    if name == "forward" {
                        assert!(tw.abs() < 1.0, "{game} forward update {}: {tw}", n + 1);
                    }
                    if n + 1 >= 9 {
                        assert!(
                            (t - tw).abs() < 15.0,
                            "{game} {name} update {}: {}",
                            n + 1,
                            t - tw
                        );
                    }
                }
                if game == "bf1" && matches!(name, "left" | "right") {
                    let first = rows[0].1;
                    let ps2 = if name == "left" { -31.2 } else { 31.1 };
                    assert!((first - ps2).abs() < 1.0, "{game} {name}: update 1 {first}");
                }
                let (t, tw) = rows[rows.len() - 1];
                assert!((t - tw).abs() < 2.0, "{game} {name} at the end: {}", t - tw);
                let after = after.expect("0.5 s after the roll");
                println!(
                    "{game} {name}: {} updates, 0.5 s after: twist {after:.2}",
                    rows.len()
                );
                // The fixture's class is the default one (9 m/s ahead): BF1's
                // forward-left run is 8.06 m/s, over its chooser's 8 m/s idle
                // line, so his legs stand and turn on the spot after the roll.
                if !(game == "bf1" && name == "forward-left") {
                    assert!(
                        (after - strafe).abs() < 10.0,
                        "{game} {name}: {after} after, {strafe} before"
                    );
                }
            }
        }
        if rolls > 0 {
            assert_eq!(rolls, 8);
        }
    }

    /// Card ANIMM acceptance 2: the feet keep to the ground. A real trooper
    /// of each game, driven at three speeds per stance and direction: the
    /// ground the legs' clip covers a second (its root speed, the bake's
    /// PS2 measure, over a cycle's length, times the cycles a second the
    /// animator plays) against his speed, with the plain clock (main's) and
    /// with the PS2's rate. The rate leaves 5 % of the gap to the clip's own
    /// speed (`0.95 v / v_clip + 0.05`), so it is under 5 % of the speed
    /// wherever he moves at more than half the clip's speed. `--nocapture`
    /// prints the table.
    #[test]
    fn the_feet_keep_to_the_ground() {
        use crate::animator::{AnimInput, Animator};
        use core::soldier::Stance;
        let root = crate::assets::baked_root();
        let mut rows = 0;
        for game in ["bf1", "bf2"] {
            let geometry = "rep_inf_trooper";
            if !root
                .join(format!("side_{game}/soldiers/{geometry}.glb"))
                .exists()
            {
                eprintln!("skip: no {game} {geometry} bake");
                continue;
            }
            let t = team(&[("trooper", Some(geometry), None)]);
            let vis = SoldierVisuals::load_cpu(&root, game, &[t.clone(), t]);
            let sa = &vis.assets[0];
            let (Some(rig), clips, skel) = (&sa.rig, &sa.asset.clips, &sa.asset.skeleton) else {
                panic!("{game}: no rig");
            };
            for (stance, ahead, speeds) in [
                (Stance::Stand, true, [2.5, 4.5, 6.5]),
                (Stance::Stand, false, [2.0, 3.5, 5.0]),
                (Stance::Crouch, true, [1.2, 2.0, 3.0]),
                (Stance::Prone, true, [0.6, 1.0, 1.5]),
            ] {
                // BF2's rifle set has no prone walk.
                if game == "bf2" && stance == Stance::Prone {
                    continue;
                }
                for speed in speeds {
                    let input = AnimInput {
                        stance,
                        vel: glam::Vec2::new(0.0, if ahead { speed } else { -speed }),
                        yaw: 0.0,
                        max_speed: 7.0,
                        dead: false,
                        set: 0,
                        fired: false,
                        reloading: false,
                        reload_time: 0.0,
                        alert: false,
                        aim: (0.0, 0.0),
                    };
                    let mut a = Animator::new();
                    let dt = 1.0 / 30.0;
                    for _ in 0..60 {
                        a.step(rig, clips, skel, &input, dt);
                    }
                    let c = &clips[a.lower_clip().expect("a legs clip")];
                    // The clips are baked in place: the ground a cycle covers
                    // is the PS2's root speed over the cycle's length.
                    let denom = if game == "bf1" {
                        c.frames
                    } else {
                        c.frames - 1
                    } as f32;
                    let v_clip = rig
                        .clip_speed(a.lower_clip().unwrap())
                        .unwrap_or_else(|| panic!("{game}: no root speed for {}", c.name))[1]
                        .abs();
                    let travel = v_clip * denom / 30.0;
                    // Cycles a second: 30 updates' phase steps (each under
                    // one cycle).
                    let mut cycles = 0.0;
                    for _ in 0..30 {
                        let t0 = a.lower_phase();
                        a.step(rig, clips, skel, &input, dt);
                        cycles += (a.lower_phase() - t0).rem_euclid(1.0);
                    }
                    let plain = 30.0 / denom;
                    let feet = cycles * travel;
                    let before = plain * travel;
                    let err = |f: f32| 100.0 * (f - speed).abs() / speed;
                    println!(
                        "{game} {stance:?} {} {speed:.1} m/s: {} clip {:.2} m/s | plain {before:.2} ({:.1} %) | rate {feet:.2} ({:.1} %)",
                        if ahead { "ahead" } else { "back" },
                        c.name,
                        before,
                        err(before),
                        err(feet),
                    );
                    // The PS2's own residual, and under 5 % from half the
                    // clip's speed up.
                    assert!(
                        (feet - speed).abs() <= 0.05 * (before - speed).abs() + 0.01 * speed,
                        "{game} {stance:?} {speed}: {feet} against {speed}"
                    );
                    if speed >= before / 2.0 {
                        assert!(err(feet) < 5.0, "{game} {stance:?} {speed}: {}", err(feet));
                    }
                    rows += 1;
                }
            }
        }
        if rows > 0 {
            assert_eq!(rows, 21);
        }
    }
}
