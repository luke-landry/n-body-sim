use crate::gravity::{Accelerations, Gravity};
use crate::simulation::Bodies;

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
        k = (G * m_j) / (r^2 + ε^2)^(3/2)
    we have
        a_ix = ∑(j=1..N, j != i){ k*∆x }
        a_iy = ∑(j=1..N, j != i){ k*∆y }
        a_iz = ∑(j=1..N, j != i){ k*∆z }
    where computing k once per (i,j) saves having to recalculate the full formula for each component.

    Writing this in vector terms, we have
        a_i = ∑(j=1..N, j != i){ k*∆r }
    this is how the following code maps to the formula:
        compute_acceleration_for_body() performs the outer loop ∑(j=1..N, j != i)
        accumulate_pair() computes the { k*∆r } for a pair of bodies (i,j)
*/
impl Gravity for NewtonGravity {
    fn calculate_accelerations(&self, bodies: &Bodies, accelerations: &mut Accelerations) {
        let n = bodies.len();
        let g = self.g_constant;
        let eps2 = self.softening_factor.powi(2);

        let m = &bodies.masses;

        let rx = &bodies.pos_x;
        let ry = &bodies.pos_y;
        let rz = &bodies.pos_z;

        let ax = &mut accelerations.ax;
        let ay = &mut accelerations.ay;
        let az = &mut accelerations.az;

        for i in 0..n {
            (ax[i], ay[i], az[i]) = compute_acceleration_for_body(i, n, g, eps2, m, rx, ry, rz);
        }
    }
}

// Computes the acceleration components for body i from all other bodies j != i.
pub fn compute_acceleration_for_body(
    i: usize,
    n: usize,
    g: f64,
    eps2: f64,
    m: &[f64],
    rx: &[f64],
    ry: &[f64],
    rz: &[f64],
) -> (f64, f64, f64) {
    let rx_i = rx[i];
    let ry_i = ry[i];
    let rz_i = rz[i];
    let mut ax = 0.0;
    let mut ay = 0.0;
    let mut az = 0.0;

    // split the loop into two to avoid the if statement for
    //      if j != i { continue; }
    // to avoid branching in a single loop

    for j in 0..i {
        accumulate_pair(
            g, eps2, m[j], rx[j], ry[j], rz[j], rx_i, ry_i, rz_i, &mut ax, &mut ay, &mut az,
        );
    }
    for j in (i + 1)..n {
        accumulate_pair(
            g, eps2, m[j], rx[j], ry[j], rz[j], rx_i, ry_i, rz_i, &mut ax, &mut ay, &mut az,
        );
    }
    (ax, ay, az)
}

// Computes the contribution to the acceleration of body i from body j using the formulas:
//      a_ix += k*∆x
//      a_iy += k*∆y
//      a_iz += k*∆z
// where
//      k = (G * m_j) / (r^2 + ε^2)^(3/2)
pub fn accumulate_pair(
    g: f64,
    eps2: f64,
    m_j: f64,
    rx_j: f64,
    ry_j: f64,
    rz_j: f64,
    rx_i: f64,
    ry_i: f64,
    rz_i: f64,
    ax_i: &mut f64,
    ay_i: &mut f64,
    az_i: &mut f64,
) {
    let dx = rx_j - rx_i;
    let dy = ry_j - ry_i;
    let dz = rz_j - rz_i;
    let r2 = dx * dx + dy * dy + dz * dz;
    let r_softened = r2 + eps2;

    // this is equivalent to using r_softened.powf(-1.5) but avoids an expensive
    // floating-point exponentiation and instead uses a single square root
    // and a few multiplications, which is much faster
    let inv_r_softened_sqrt = 1.0 / r_softened.sqrt();
    let inv_r_softened_sqrt_cubed = inv_r_softened_sqrt * inv_r_softened_sqrt * inv_r_softened_sqrt;

    let k = g * m_j * inv_r_softened_sqrt_cubed;
    *ax_i += k * dx;
    *ay_i += k * dy;
    *az_i += k * dz;
}
