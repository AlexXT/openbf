//! Terrain height query, transcribed from the decomp (BF1 FUN_00473710;
//! BF2 FUN_0055ebc0 is the same routine per audit A2/RE_LOG).
//!
//! The game does NOT bilinear-filter: each grid cell is split into two
//! triangles and the point is lerped on the triangle plane, with the split
//! diagonal ALTERNATING by z-row parity (`iz & 1`):
//! - even row: diagonal from (ix,iz+1) to (ix+1,iz)-side (xfrac vs zfrac);
//! - odd row: diagonal from (ix,iz) to (ix+1,iz+1) (xfrac vs 1-zfrac).
//!
//! Globals in the original: DAT_00564698/9c = world->grid offsets,
//! DAT_00564690 = gridSize, DAT_00564694 = unit, DAT_005647bc = 1/unit,
//! DAT_005647c0 = heightScale (folded into `heights` here — lerp is linear,
//! prescaling is bit-identical).

/// Row-major (z * size + x) grid of world-space heights.
pub struct Heightfield {
    /// Grid vertices per side.
    pub size: u32,
    /// World units per cell.
    pub unit: f32,
    /// Added to world x/z before the cell lookup (grid min corner negated).
    pub offset_x: f32,
    pub offset_z: f32,
    pub heights: Vec<f32>,
}

impl Heightfield {
    fn h(&self, ix: u32, iz: u32) -> f32 {
        // Clamp like the original's index math: the query clamps the world
        // coordinate to (size-1)*unit, but the +1 neighbors of the last cell
        // still need a guard when the coordinate sits exactly on the edge.
        let ix = ix.min(self.size - 1);
        let iz = iz.min(self.size - 1);
        self.heights[(iz * self.size + ix) as usize]
    }

    /// The clamped cell under (x, z) and the fractions inside it.
    fn cell(&self, x: f32, z: f32) -> (u32, u32, f32, f32) {
        let max = (self.size - 1) as f32 * self.unit;
        let fx = (x + self.offset_x).clamp(0.0, max);
        let fz = (z + self.offset_z).clamp(0.0, max);
        let inv = 1.0 / self.unit;
        let ix = (fx * inv) as u32;
        let iz = (fz * inv) as u32;
        (ix, iz, fx * inv - ix as f32, fz * inv - iz as f32)
    }

    /// The unit up-facing normal of the triangle [`Heightfield::height_at`]
    /// lerps on at (x, z) — BF1 FUN_00473e08, which picks the triangle by the
    /// same z-row parity split (analysis/re-foliage.md §6: the terrain foliage
    /// stands on it).
    pub fn normal_at(&self, x: f32, z: f32) -> [f32; 3] {
        let (ix, iz, xfrac, zfrac) = self.cell(x, z);
        let h = |dx: u32, dz: u32| self.h(ix + dx, iz + dz);
        // d(height)/d(fraction) along x and z, per triangle (see height_at).
        let (gx, gz) = if iz & 1 == 0 {
            if xfrac < zfrac {
                (h(1, 1) - h(0, 1), h(0, 1) - h(0, 0))
            } else {
                (h(1, 0) - h(0, 0), h(1, 1) - h(1, 0))
            }
        } else if xfrac < 1.0 - zfrac {
            (h(1, 0) - h(0, 0), h(0, 1) - h(0, 0))
        } else {
            (h(1, 1) - h(0, 1), h(1, 1) - h(1, 0))
        };
        let n = [-gx, self.unit, -gz];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        [n[0] / len, n[1] / len, n[2] / len]
    }

    /// World-space ground height at (x, z) — FUN_00473710 transcription.
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let (ix, iz, xfrac, zfrac) = self.cell(x, z);

        if iz & 1 == 0 {
            if xfrac < zfrac {
                // triangle (ix,iz) (ix,iz+1) (ix+1,iz+1), base at (ix,iz+1)
                let base = self.h(ix, iz + 1);
                base + (self.h(ix + 1, iz + 1) - base) * xfrac
                    + (self.h(ix, iz) - base) * (1.0 - zfrac)
            } else {
                // triangle (ix,iz) (ix+1,iz) (ix+1,iz+1), base at (ix+1,iz)
                let base = self.h(ix + 1, iz);
                base + (self.h(ix + 1, iz + 1) - base) * zfrac
                    + (self.h(ix, iz) - base) * (1.0 - xfrac)
            }
        } else if xfrac < 1.0 - zfrac {
            // triangle (ix,iz) (ix+1,iz) (ix,iz+1), base at (ix,iz)
            let base = self.h(ix, iz);
            base + (self.h(ix + 1, iz) - base) * xfrac + (self.h(ix, iz + 1) - base) * zfrac
        } else {
            // triangle (ix+1,iz) (ix,iz+1) (ix+1,iz+1), base at (ix+1,iz+1)
            let base = self.h(ix + 1, iz + 1);
            base + (self.h(ix, iz + 1) - base) * (1.0 - xfrac)
                + (self.h(ix + 1, iz) - base) * (1.0 - zfrac)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 3x3 grid, unit 1, min corner at world (0,0): heights = x + 10*z.
    fn hf() -> Heightfield {
        Heightfield {
            size: 3,
            unit: 1.0,
            offset_x: 0.0,
            offset_z: 0.0,
            heights: (0..9)
                .map(|i| (i % 3) as f32 + 10.0 * (i / 3) as f32)
                .collect(),
        }
    }

    #[test]
    fn exact_grid_points_and_planar_interior() {
        let f = hf();
        assert_eq!(f.height_at(0.0, 0.0), 0.0);
        assert_eq!(f.height_at(2.0, 0.0), 2.0);
        assert_eq!(f.height_at(1.0, 2.0), 21.0);
        // The test surface is a plane, so every triangle reproduces it exactly
        // regardless of the diagonal — checks the lerp arithmetic.
        for &(x, z, want) in &[(0.25, 0.75, 7.75), (1.5, 0.5, 6.5), (0.5, 1.5, 15.5)] {
            let got = f.height_at(x, z);
            assert!(
                (got - want).abs() < 1e-5,
                "({x},{z}): got {got}, want {want}"
            );
        }
    }

    #[test]
    fn diagonal_alternates_with_row_parity() {
        // Non-planar cell: spike one corner and probe both sides of the split.
        let mut f = hf();
        f.heights[4] = 100.0; // spike vertex (x=1, z=1)

        // Even row (iz=0), cell (1,0), sampled near corner (2,0) with
        // xfrac > zfrac: that triangle is (1,0)(2,0)(2,1) — the spike at
        // (1,1) is not one of its vertices and must not leak in.
        let got = f.height_at(1.99, 0.01);
        assert!((2.0..3.0).contains(&got), "spike must not leak: {got}");
        // Odd row (iz=1), cell (1,1): near corner (1,1) with xfrac<1-zfrac the
        // base IS the spike vertex.
        let got = f.height_at(1.01, 1.01);
        assert!(
            got > 95.0,
            "odd-row triangle starts at the spiked base: {got}"
        );
    }

    /// The normal is the slope `height_at` itself has on each of the four
    /// triangles of a non-planar grid: a finite difference inside the
    /// triangle, turned into (−∂h/∂x, 1, −∂h/∂z) and normalised, agrees.
    #[test]
    fn normal_is_the_height_query_s_own_triangle() {
        let mut f = hf();
        f.heights[4] = 3.0;
        f.heights[8] = -4.0;
        // One probe well inside each triangle kind: even row both halves
        // (cell (0,0)), odd row both halves (cell (1,1)).
        for &(x, z) in &[(0.2, 0.7), (0.7, 0.2), (1.2, 1.3), (1.8, 1.7)] {
            let e = 1e-3;
            let gx = (f.height_at(x + e, z) - f.height_at(x - e, z)) / (2.0 * e);
            let gz = (f.height_at(x, z + e) - f.height_at(x, z - e)) / (2.0 * e);
            let l = (gx * gx + 1.0 + gz * gz).sqrt();
            let want = [-gx / l, 1.0 / l, -gz / l];
            let got = f.normal_at(x, z);
            for k in 0..3 {
                assert!(
                    (got[k] - want[k]).abs() < 1e-3,
                    "({x},{z}): {got:?} vs {want:?}"
                );
            }
        }
        // A plane's normal everywhere: h = x + 10 z.
        let n = hf().normal_at(0.5, 0.5);
        let l = (1.0f32 + 1.0 + 100.0).sqrt();
        assert!((n[0] + 1.0 / l).abs() < 1e-6 && (n[1] - 1.0 / l).abs() < 1e-6);
        assert!((n[2] + 10.0 / l).abs() < 1e-6);
    }

    #[test]
    fn clamps_outside_the_grid() {
        let f = hf();
        assert_eq!(f.height_at(-5.0, -5.0), 0.0);
        assert_eq!(f.height_at(50.0, 50.0), 22.0);
    }
}
