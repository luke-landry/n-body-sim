pub mod octree;

use crate::gravity::{Gravity, barnes_hut::octree::BarnesHutOctree, newton::compute_acceleration};
use rayon::prelude::*;

pub struct BarnesHutGravity {
    g_constant: f64,
    softening_factor: f64,
    octree: BarnesHutOctree,
}

impl BarnesHutGravity {
    pub fn new(g_constant: f64, softening_factor: f64, theta: f64, n: usize) -> Self {
        BarnesHutGravity {
            g_constant: g_constant,
            softening_factor: softening_factor,
            octree: BarnesHutOctree::new(n, theta, 1), // TODO configurable max_leaf_size
        }
    }
}

impl Gravity for BarnesHutGravity {
    fn calculate_accelerations(
        &mut self,
        masses: &[f64],
        rx: &[f64],
        ry: &[f64],
        rz: &[f64],
        ax: &mut [f64],
        ay: &mut [f64],
        az: &mut [f64],
    ) {
        let g = self.g_constant;
        let eps2 = self.softening_factor * self.softening_factor;
        let acceleration_function = |m: f64, dx: f64, dy: f64, dz: f64| -> (f64, f64, f64) {
            compute_acceleration(g, eps2, m, dx, dy, dz)
        };

        self.octree.build(masses, rx, ry, rz);
        ax.par_iter_mut()
            .zip(ay.par_iter_mut())
            .zip(az.par_iter_mut())
            .enumerate()
            .for_each(|(i, ((ax_i, ay_i), az_i))| {
                (*ax_i, *ay_i, *az_i) = self
                    .octree
                    .compute_acceleration_for_body(i, acceleration_function);
            });
    }
}
