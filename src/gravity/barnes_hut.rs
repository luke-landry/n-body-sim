use crate::gravity::{Gravity, barnes_hut::octree::BarnesHutOctree};
use rayon::prelude::*;

pub mod octree;

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

fn compute_acceleration(g: f64, eps2: f64, m_j: f64, dx: f64, dy: f64, dz: f64) -> (f64, f64, f64) {
    let r2 = (dx * dx) + (dy * dy) + (dz * dz);
    let inv_r_softened = 1.0 / (r2 + eps2).sqrt();
    let inv_r_softened_cubed = inv_r_softened * inv_r_softened * inv_r_softened;
    let k = g * m_j * inv_r_softened_cubed;
    (k * dx, k * dy, k * dz)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use approx::assert_relative_eq;

//     // Direct Newtonian O(N^2) calculation for reference
//     fn direct_newtonian(
//         g: f64,
//         eps2: f64,
//         masses: &[f64],
//         rx: &[f64],
//         ry: &[f64],
//         rz: &[f64],
//         ax: &mut [f64],
//         ay: &mut [f64],
//         az: &mut [f64],
//     ) {
//         let n = masses.len();
//         for i in 0..n {
//             let mut sum_ax = 0.0;
//             let mut sum_ay = 0.0;
//             let mut sum_az = 0.0;
//             for j in 0..n {
//                 if i == j {
//                     continue;
//                 }
//                 let dx = rx[j] - rx[i];
//                 let dy = ry[j] - ry[i];
//                 let dz = rz[j] - rz[i];
//                 let (dax, day, daz) = compute_acceleration(g, eps2, masses[j], dx, dy, dz);
//                 sum_ax += dax;
//                 sum_ay += day;
//                 sum_az += daz;
//             }
//             ax[i] = sum_ax;
//             ay[i] = sum_ay;
//             az[i] = sum_az;
//         }
//     }

//     #[test]
//     fn test_barnes_hut_theta_zero_matches_newtonian() {
//         // Small N for easy debugging
//         let masses = vec![1.0, 2.0, 1.5];
//         let rx = vec![0.0, 1.0, 0.0];
//         let ry = vec![0.0, 0.0, 1.0];
//         let rz = vec![0.0, 0.0, 0.0];
//         let n = masses.len();

//         let g = 1.0;
//         let softening = 1e-3;
//         let eps2 = softening * softening;

//         // Newtonian reference
//         let mut ax_newton = vec![0.0; n];
//         let mut ay_newton = vec![0.0; n];
//         let mut az_newton = vec![0.0; n];
//         direct_newtonian(
//             g,
//             eps2,
//             &masses,
//             &rx,
//             &ry,
//             &rz,
//             &mut ax_newton,
//             &mut ay_newton,
//             &mut az_newton,
//         );

//         // Barnes-Hut with theta=0, max_leaf_size=1
//         let mut bh = BarnesHutGravity::new(g, softening, 0.0, n);
//         let mut ax_bh = vec![0.0; n];
//         let mut ay_bh = vec![0.0; n];
//         let mut az_bh = vec![0.0; n];
//         bh.calculate_accelerations(&masses, &rx, &ry, &rz, &mut ax_bh, &mut ay_bh, &mut az_bh);

//         // Compare
//         for i in 0..n {
//             assert_relative_eq!(ax_bh[i], ax_newton[i], epsilon = 1e-10);
//             assert_relative_eq!(ay_bh[i], ay_newton[i], epsilon = 1e-10);
//             assert_relative_eq!(az_bh[i], az_newton[i], epsilon = 1e-10);
//         }
//     }
// }
