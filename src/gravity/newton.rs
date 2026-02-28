use crate::gravity::Gravity;

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
        let n = masses.len();
        let g = self.g_constant;
        let eps2 = self.softening_factor * self.softening_factor;
        for i in 0..n {
            (ax[i], ay[i], az[i]) = compute_acceleration_for_body(i, g, eps2, masses, rx, ry, rz);
        }
    }
}

/// Computes the acceleration components for body i from all other bodies j != i.
/// The implementation uses output parameters to avoid overhead of returning by value
pub fn compute_acceleration_for_body(
    i: usize,
    g: f64,
    eps2: f64,
    masses: &[f64],
    rx: &[f64],
    ry: &[f64],
    rz: &[f64],
) -> (f64, f64, f64) {
    let n = masses.len();
    let rx_i = rx[i];
    let ry_i = ry[i];
    let rz_i = rz[i];
    let mut ax_i = 0.0;
    let mut ay_i = 0.0;
    let mut az_i = 0.0;

    // Compute the acceleration components for body i by summing the contributions from each other body j != i
    for j in 0..n {
        // Benchmarking shows negligible performance difference having this check
        // in the loop instead of separating into two loops to avoid this branch
        if j == i {
            continue;
        }

        let m_j = masses[j];
        let dx = rx[j] - rx_i;
        let dy = ry[j] - ry_i;
        let dz = rz[j] - rz_i;
        let (ax_ij, ay_ij, az_ij) = compute_acceleration(g, eps2, m_j, dx, dy, dz);
        ax_i += ax_ij;
        ay_i += ay_ij;
        az_i += az_ij;
    }
    (ax_i, ay_i, az_i)
}

pub fn compute_acceleration(
    g: f64,
    eps2: f64,
    m_j: f64,
    dx: f64,
    dy: f64,
    dz: f64,
) -> (f64, f64, f64) {
    let r2 = (dx * dx) + (dy * dy) + (dz * dz);
    let inv_r_softened = 1.0 / (r2 + eps2).sqrt();
    let inv_r_softened_cubed = inv_r_softened * inv_r_softened * inv_r_softened;
    let k = g * m_j * inv_r_softened_cubed;
    (k * dx, k * dy, k * dz)
}
