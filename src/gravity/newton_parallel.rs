use crate::gravity::newton::compute_acceleration_for_body;
use crate::gravity::{Accelerations, Gravity};
use crate::simulation::Bodies;
use rayon::prelude::*;

pub struct NewtonParallelGravity {
    g_constant: f64,
    softening_factor: f64,
}

impl NewtonParallelGravity {
    pub fn new(g_constant: f64, softening_factor: f64) -> Self {
        NewtonParallelGravity {
            g_constant,
            softening_factor,
        }
    }
}

/*
    Using the following equations from the sequential Newton gravity implementation,
    the acceleration per body can be calculated independently in parallel:
        a_ix = ∑(j=1..N, j != i){ k*∆x }
        a_iy = ∑(j=1..N, j != i){ k*∆y }
        a_iz = ∑(j=1..N, j != i){ k*∆z }
    where
        k = (G * m_j)/ (r^2 + ε^2)^(3/2)

    To parallelize the calculations, we can use par_iter from the rayon library, which
    will divide the work of calculating accelerations for each body using a thread pool.
*/
impl Gravity for NewtonParallelGravity {
    fn calculate_accelerations(&self, bodies: &Bodies, accelerations: &mut Accelerations) {
        let n = bodies.len();
        let g = self.g_constant;
        let eps2 = self.softening_factor.powi(2);

        let m = &bodies.masses;

        let rx = &bodies.pos_x;
        let ry = &bodies.pos_y;
        let rz = &bodies.pos_z;

        accelerations
            .ax
            .par_iter_mut()
            .zip(accelerations.ay.par_iter_mut())
            .zip(accelerations.az.par_iter_mut())
            .enumerate()
            .for_each(|(i, ((ax_i, ay_i), az_i))| {
                (*ax_i, *ay_i, *az_i) = compute_acceleration_for_body(i, n, g, eps2, m, rx, ry, rz);
            });
    }
}
