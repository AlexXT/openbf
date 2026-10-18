//! In-game UI. The HUD is BF2's own, interpreted from its baked tree
//! ([`hud_tree`] the elements and their events, [`hud_draw`] their layout into
//! triangles for `render::hud`; [`hud_map`], [`hud_squad`] and
//! [`hud_capture`] the minimap, the squad-order cross and the capture
//! display, drawn from code as the PS2 draws them), fed by a
//! [`hud::HudModel`]; the menus, the
//! spawn screen and the HUD lines BF2's tree does not carry yet are egui.
//! Both halves are pure functions of the model so they can be tested headless,
//! independently of the renderer; [`overlay`] is the egui-wgpu / egui-winit
//! glue that paints egui into the game's frame through
//! `render::Renderer::render_with_overlay`.
//!
//! Layout rule (PLAN §Presentation, widescreen-native): every element is
//! anchored to an edge/corner in logical points — nothing is authored for a
//! fixed aspect or stretched.

pub use egui;

pub mod hud;
pub mod hud_capture;
pub mod hud_draw;
pub mod hud_map;
pub mod hud_squad;
pub mod hud_tree;
pub mod loading;
pub mod menu;
pub mod overlay;
pub mod shell;
pub mod spawn;
