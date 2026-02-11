use crate::gravity::Gravity;
use crate::simulation::Body;
use glam::DVec3;
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
        a_ix = ∑(j=1..N, j != i){ k*m_j*∆x }
        a_iy = ∑(j=1..N, j != i){ k*m_j*∆y }
        a_iz = ∑(j=1..N, j != i){ k*m_j*∆z }
    where
        k = G / (r^2 + ε^2)^(3/2)

    To parallelize the calculations, we can use par_iter from the rayon library, which
    will divide the work of calculating accelerations for each body across multiple threads.
    To keep each calculation independent so that it can be computed in parallel, the "trick"
    using Newton's third law that allows partially updating the acceleration of body j while
    calculating the acceleration of body i cannot be used here, and the loop will have to
    iterate N^2 times. However, the overall performance should still improve due to parallelization.
*/
impl Gravity for NewtonParallelGravity {
    fn calculate_accelerations(&self, bodies: &[Body], accelerations: &mut [DVec3]) {
        let g = self.g_constant;
        let epsilon_squared = self.softening_factor.powi(2);
        accelerations
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, a_i)| {
                let pos_i = bodies[i].position;
                for (j, body_j) in bodies.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let r = body_j.position - pos_i;
                    let k = g / ((r.length_squared() + epsilon_squared).powf(1.5));
                    *a_i += k * body_j.mass * r;
                }
            });
    }
}
