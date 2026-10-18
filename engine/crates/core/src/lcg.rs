//! Inline gameplay LCG used by BOTH games for weapon spread, BF2 random
//! shot/salvo delays, and tumble/roll picks — NOT the SCE libc `rand()` in
//! `rng.rs` (the games use both; the sim carries both).
//!
//! Traced (lead-verified against the decomp bodies, 2026-08-30):
//! - BF2 `FUN_002a3e48` (uniform(min,max)) on global `DAT_0073d870`;
//! - BF1 shot spawn `FUN_002892a8` on global `DAT_0066eed0`.
//!
//! ```text
//! x = x * 0x19660D + 0x3C6EF35F            (mod 2^32)
//! one uniform in [0,1) consumes TWO steps x1, x2:
//! u = f32::from_bits((((x1 >> 16) | (x2 & 0xFFFF_0000)) >> 9) | 0x3F80_0000) - 1.0
//! ```
//! i.e. the high 16 bits of two successive states are packed into a 23-bit
//! mantissa. `uniform(a, b)` returns `a` WITHOUT stepping when `a == b`
//! (FUN_002a3e48's early return) — this keeps deterministic-delay weapons
//! from consuming random numbers.

#[derive(Clone, Debug)]
pub struct GameLcg {
    state: u32,
}

impl GameLcg {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// One raw LCG step; returns the new state.
    pub fn step(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(0x0019_660D)
            .wrapping_add(0x3C6E_F35F);
        self.state
    }

    /// The games' two-state → [0,1) float packing (exact).
    pub fn frac(x1: u32, x2: u32) -> f32 {
        let bits = (((x1 >> 16) | (x2 & 0xFFFF_0000)) >> 9) | 0x3F80_0000;
        f32::from_bits(bits) - 1.0
    }

    /// One uniform in [0,1): two steps.
    pub fn unit(&mut self) -> f32 {
        let x1 = self.step();
        let x2 = self.step();
        Self::frac(x1, x2)
    }

    /// BF2 FUN_002a3e48: `a + (b - a) * unit()`, no step when `a == b`.
    pub fn uniform(&mut self, a: f32, b: f32) -> f32 {
        if a == b {
            return a;
        }
        a + (b - a) * self.unit()
    }

    /// Triangular offset in ±`mag`: `mag * (U1 - U2)` with U1 drawn first
    /// (BF1 FUN_002892a8 order: four steps per axis).
    pub fn triangular(&mut self, mag: f32) -> f32 {
        let u1 = self.unit();
        let u2 = self.unit();
        mag * (u1 - u2)
    }

    /// Integer draw used by the CP spawn-node picker (BF1 FUN_0029df70 /
    /// BF2 FUN_00356998): `((r1 >> 16) | (r2 & 0xFFFF0000)) % n`.
    pub fn index(&mut self, n: u32) -> u32 {
        let r1 = self.step();
        let r2 = self.step();
        ((r1 >> 16) | (r2 & 0xFFFF_0000)) % n.max(1)
    }

    pub fn state(&self) -> u32 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference trace computed independently (Python, same formula as the
    /// decomp) for seed 0x12345678.
    #[test]
    fn matches_fun_002a3e48_reference() {
        let mut g = GameLcg::new(0x1234_5678);
        let x1 = g.step();
        let x2 = g.step();
        assert_eq!(x1, 0x7543_2777);
        assert_eq!(x2, 0xCD30_5E6A);
        let u = GameLcg::frac(x1, x2);
        assert_eq!(u.to_bits(), 0x3F4D_3074, "{u}"); // 0.801520586
        let x3 = g.step();
        let x4 = g.step();
        assert_eq!((x3, x4), (0x25DB_FAC1, 0x4B5C_952C));
        assert!((GameLcg::frac(x3, x4) - 0.294_374_7).abs() < 1e-7);

        // uniform(2,5) from a fresh seed = 2 + 3*u, bit-exact in f32.
        let mut g = GameLcg::new(0x1234_5678);
        let v = g.uniform(2.0, 5.0);
        assert_eq!(v.to_bits(), 0x408C_F22C, "{v}"); // 4.404562

        // Equal bounds: no step consumed.
        let before = g.state();
        assert_eq!(g.uniform(0.2, 0.2), 0.2);
        assert_eq!(g.state(), before);

        // Triangular from a fresh seed: 0.5 * (U1 - U2) with U1 = 0.8015206,
        // U2 = 0.2943747.
        let mut g = GameLcg::new(0x1234_5678);
        let t = g.triangular(0.5);
        assert!((t - 0.253_572_94).abs() < 1e-7, "{t}");
    }

    #[test]
    fn index_draw_is_bounded_and_deterministic() {
        let mut a = GameLcg::new(0x1234_5678);
        let x1 = a.step();
        let x2 = a.step();
        let mut b = GameLcg::new(0x1234_5678);
        assert_eq!(b.index(13), ((x1 >> 16) | (x2 & 0xFFFF_0000)) % 13);
        for n in 1..50 {
            assert!(b.index(n) < n);
        }
    }

    #[test]
    fn triangular_is_symmetric_and_bounded() {
        let mut g = GameLcg::new(7);
        let mut sum = 0.0f64;
        for _ in 0..20_000 {
            let t = g.triangular(0.5);
            assert!(t.abs() <= 0.5);
            sum += t as f64;
        }
        assert!((sum / 20_000.0).abs() < 0.01, "mean {}", sum / 20_000.0);
    }
}
