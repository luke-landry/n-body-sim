use glam::DVec3;

use crate::gravity::Gravity;
use crate::simulation::Body;

pub struct NewtonGravity {
    g_constant: f64,
    softening_factor: f64,
}

impl NewtonGravity {
    pub fn new(g_constant: f64, softening_factor: f64) -> Self {
        NewtonGravity {
            g_constant,
            softening_factor,
        }
    }
}

/*
    Newton's law of universal gravitation:
        F = (G*m_1*m_2)/r^2

    Isolating the x, y, and z components of the force:
        F_x = F*(∆x/r) = (G*m_1*m_2*∆x)/r^3
        F_y = F*(∆y/r) = (G*m_1*m_2*∆y)/r^3
        F_z = F*(∆z/r) = (G*m_1*m_2*∆z)/r^3


    To avoid numerical instability as r->0, we add the softening factor (ε),
    which modifies ("softens") the gravity forces at short distances by replacing
    1/r with 1/(r^2 + ε^2)^(1/2), so 1/r^3 becomes 1/(r^2 + ε^2)^(3/2):
        F_x = (G*m_1*m_2*∆x)/(r^2 + ε^2)^(3/2)  --  (1a)
        F_y = (G*m_1*m_2*∆y)/(r^2 + ε^2)^(3/2)  --  (1b)
        F_z = (G*m_1*m_2*∆z)/(r^2 + ε^2)^(3/2)  --  (1c)

    Newton's 2nd law (treating m_1 as m):
        F = m*a  --  (2)

    Combining (1a), (1b), and (1c) with (2), dividing by m_1,
    we get the x, y and z components of the acceleration:
        a_x = (G*m_2*∆x) / (r^2 + ε^2)^(3/2)
        a_y = (G*m_2*∆y) / (r^2 + ε^2)^(3/2)
        a_z = (G*m_2*∆z) / (r^2 + ε^2)^(3/2)

    In an N-body system, by the superposition principle, the total force exerted on
    a body (i) is the sum of the forces exerted by each other body (j), so applying
    this to the acceleration components:
        a_ix = ∑(j=1..N, j != i){ (G*m_j*∆x) / (r^2 + ε^2)^(3/2) }
        a_iy = ∑(j=1..N, j != i){ (G*m_j*∆y) / (r^2 + ε^2)^(3/2) }
        a_iz = ∑(j=1..N, j != i){ (G*m_j*∆z) / (r^2 + ε^2)^(3/2) }
    where forces on body i point towards body j:
        ∆x = x_j - x_i
        ∆y = y_j - y_i
        ∆z = z_j - z_i
        r^2 = ∆x^2 + ∆y^2 + ∆z^2

    Letting
        k = G / (r^2 + ε^2)^(3/2)
    we have
        a_ix = ∑(j=1..N, j != i){ k*m_j*∆x }
        a_iy = ∑(j=1..N, j != i){ k*m_j*∆y }
        a_iz = ∑(j=1..N, j != i){ k*m_j*∆z }
    where computing k once per (i,j) saves having to recalculate the full formula

    By Newton's 3rd law, the force between a pair of bodies is equal and opposite:
        F_i = -F_j
    Since a = F / m, the acceleration on body i due to body j is proportional to m_j,
    and the acceleration on body j due to body i is opposite in direction and proportional to m_i.
        a_ix += k*m_j*∆x
        a_iy += k*m_j*∆y
        a_iz += k*m_j*∆z
    implies
        a_jx -= k*m_i*∆x
        a_jy -= k*m_i*∆y
        a_jz -= k*m_i*∆z
    This means we can compute the pairwise interactions in the same iteration, so for example the
    iteration computing the effect of b2 on b1 can easily compute the effect of b1 on b2 as well.
    This means instead of iterating (i=1..N)*(j=1..N), we can iterate (i=1..N)*(j=i+1..N)
    This reduces the number of iterations we need to perform from (N^2)-N to (N*(N-1))/2.
    The time complexity is still O(N^2), but this optimization effectively halves the # of
    iterations compared to a naive implementation.

    With r and a as (x,y,z) vectors, with r=(∆x, ∆y, ∆x)
        a_ix += k*m_j*∆x, a_jx -= k*m_i*∆x
        a_iy += k*m_j*∆y, a_jy -= k*m_i*∆y
        a_iz += k*m_j*∆z, a_jz -= k*m_i*∆z
    can be written as
        a_i = k * m_j * r
        a_j = k * m_i * r
    and simplified to 
        a_i = (k * r) * m_j
        a_j = (k * r)* m_i
*/
impl Gravity for NewtonGravity {
    fn calculate_accelerations(&self, bodies: &[Body], accelerations: &mut [DVec3]) {
        let g = self.g_constant;
        let epsilon_squared = self.softening_factor.powi(2);
        let n = bodies.len();

        for i in 0..n {
            for j in i + 1..n {
                let m_i = bodies[i].mass;
                let m_j = bodies[j].mass;
                let r = bodies[j].position - bodies[i].position;
                let k = g / ((r.length_squared() + epsilon_squared).powf(1.5));
                let kr = k * r;
                
                accelerations[i] += kr * m_j;
                accelerations[j] -= kr * m_i;
            }
        }
    }
}
