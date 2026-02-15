use crate::gravity::Gravity;
use rayon::prelude::*;
// This module uses the similarly named barnes_hut crate by
// David-OConnor for the Barnes-Hut algorithm implementation
use barnes_hut::{self, BhConfig, BodyModel, Cube, Tree};

pub struct BarnesHutGravity {
    g_constant: f64,
    softening_factor: f64,
    bh_config: BhConfig,
}

impl BarnesHutGravity {
    pub fn new(g_constant: f64, softening_factor: f64, theta: f64) -> Self {
        BarnesHutGravity {
            g_constant: g_constant,
            softening_factor: softening_factor,
            bh_config: BhConfig {
                θ: theta,
                ..BhConfig::default()
            },
        }
    }
}

struct BodyRef<'a> {
    masses: &'a [f64],
    pos_x: &'a [f64],
    pos_y: &'a [f64],
    pos_z: &'a [f64],
    index: usize,
}

impl<'a> barnes_hut::BodyModel for BodyRef<'a> {
    fn posit(&self) -> lin_alg::f64::Vec3 {
        lin_alg::f64::Vec3::new(
            self.pos_x[self.index],
            self.pos_y[self.index],
            self.pos_z[self.index],
        )
    }

    fn mass(&self) -> f64 {
        self.masses[self.index]
    }
}

impl Gravity for BarnesHutGravity {
    fn calculate_accelerations(
        &self,
        masses: &[f64],
        rx: &[f64],
        ry: &[f64],
        rz: &[f64],
        ax: &mut [f64],
        ay: &mut [f64],
        az: &mut [f64],
    ) {
        // The barnes_hut crate expects an AoS format, so we create a vector of
        // BodyRef wrappers that reference the original SoA data
        let bodies: Vec<BodyRef> = (0..masses.len())
            .map(|i| BodyRef {
                masses: masses,
                pos_x: rx,
                pos_y: ry,
                pos_z: rz,
                index: i,
            })
            .collect();

        // bounding box for the tree, with 0 padding and no z-offset
        let bb = Cube::from_bodies(&bodies, 0.0, false)
            .expect("barnes_hut: Cube::from_bodies: bodies must not be empty");

        let tree = Tree::new(&bodies, &bb, &self.bh_config);

        let eps2 = self.softening_factor * self.softening_factor;

        // barnes_hut takes a function that computes the acceleration contribution from a source body
        // on a target body, given the relative position and mass of the source body
        let acc_fn = |acc_dir: lin_alg::f64::Vec3, m_j, dist| -> lin_alg::f64::Vec3 {
            compute_acceleration(self.g_constant, eps2, m_j, acc_dir, dist)
        };

        ax.par_iter_mut()
            .zip(ay.par_iter_mut())
            .zip(az.par_iter_mut())
            .enumerate()
            .for_each(|(i, ((ax_i, ay_i), az_i))| {
                let accel =
                    barnes_hut::run_bh(bodies[i].posit(), i, &tree, &self.bh_config, &acc_fn);
                *ax_i = accel.x;
                *ay_i = accel.y;
                *az_i = accel.z;
            });
    }
}

// Computes acceleration contribution from a source body j on a target body i,
// given the relative position (dx, dy, dz) and mass m_j of the source body with
// the formulas:
//      a_ix += k*∆x
//      a_iy += k*∆y
//      a_iz += k*∆z
// where
//      k = (G * m_j) / (r^2 + ε^2)^(3/2)
fn compute_acceleration(
    g: f64,
    eps2: f64,
    m_j: f64,
    acc_dir: lin_alg::f64::Vec3,
    dist: f64,
) -> lin_alg::f64::Vec3 {
    let r2 = dist * dist;
    let inv_r_softened = 1.0 / (r2 + eps2).sqrt();
    let inv_r_softened_cubed = inv_r_softened * inv_r_softened * inv_r_softened;
    let k = g * m_j * inv_r_softened_cubed;
    lin_alg::f64::Vec3::new(
        k * acc_dir.x * dist, // ax = k*∆x where ∆x = acc_dir.x * dist
        k * acc_dir.y * dist, // ay = k*∆y where ∆y = acc_dir.y * dist
        k * acc_dir.z * dist, // az = k*∆z where ∆z = acc_dir.z * dist
    )
}
