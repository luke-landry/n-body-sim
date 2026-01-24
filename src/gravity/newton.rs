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

    Isolating the x and y components of the force:
        F_x = F*(∆x/r) = (G*m_1*m_2*∆x)/r^3
        F_y = F*(∆y/r) = (G*m_1*m_2*∆y)/r^3

    To avoid numerical instability as r->0, we add the softening factor (ε),
    which modifies ("softens") the gravity forces at short distances by replacing
    1/r with 1/(r^2 + ε^2)^(1/2), so 1/r^3 becomes 1/(r^2 + ε^2)^(3/2):
        F_x = (G*m_1*m_2*∆x)/(r^2 + ε^2)^(3/2)  --  (1a)
        F_y = (G*m_1*m_2*∆y)/(r^2 + ε^2)^(3/2)  --  (1b)

    Newton's 2nd law (treating m_1 as m):
        F = m*a  --  (2)

    Combining (1a) and (1b) with (2), dividing by m_1,
    we get the x and y components of the acceleration:
        a_x = (G*m_2*∆x) / (r^2 + ε^2)^(3/2)
        a_y = (G*m_2*∆y) / (r^2 + ε^2)^(3/2)

    In an N-body system, by the superposition principle, the total force exerted on
    a body (i) is the sum of the forces exerted by each other body (j), so applying
    this to the acceleration components:
        a_ix = ∑(j=1..N, j != i){ (G*m_j*∆x) / (r^2 + ε^2)^(3/2) }
        a_iy = ∑(j=1..N, j != i){ (G*m_j*∆y) / (r^2 + ε^2)^(3/2) }
    where forces on body i point towards body j:
        ∆x = x_j - x_i
        ∆y = y_j - y_i
        r^2 = ∆x^2 + ∆y^2

    Letting
        k = G / (r^2 + ε^2)^(3/2)
    we have
        a_ix = ∑(j=1..N, j != i){ k*m_j*∆x }
        a_iy = ∑(j=1..N, j != i){ k*m_j*∆y }
    where computing k once per (i,j) saves having to recalculate the full formula twice (for x and y)

    By Newton's 3rd law, the force between a pair of bodies is equal and opposite:
        F_i = -F_j
    Since a = F / m, the acceleration on body i due to body j is proportional to m_j,
    and the acceleration on body j due to body i is opposite in direction and proportional to m_i.
        a_ix += k*m_j*∆x
        a_iy += k*m_j*∆y
    implies
        a_jx -= k*m_i*∆x
        a_jy -= k*m_i*∆y
    This means we can compute the pairwise interactions in the same iteration, so for example the
    iteration computing the effect of b2 on b1 can easily compute the effect of b1 on b2 as well.
    This means instead of iterating (i=1..N)*(j=1..N), we can iterate (i=1..N)*(j=i+1..N)
    This reduces the number of iterations we need to perform from (N^2)-N to (N*(N-1))/2.
    The time complexity is still O(N^2), but this optimization effectively halves the # of
    iterations compared to a naive implementation.
*/
impl Gravity for NewtonGravity {
    fn calculate_accelerations(&self, bodies: &[Body]) -> Vec<[f64; 2]> {
        let g = self.g_constant;
        let epsilon_squared = self.softening_factor.powi(2);
        let n = bodies.len();
        let mut a = vec![[0.0, 0.0]; n];

        for i in 0..n {
            for j in i + 1..n {
                let m_i = bodies[i].mass;
                let m_j = bodies[j].mass;
                let dx = bodies[j].position[0] - bodies[i].position[0];
                let dy = bodies[j].position[1] - bodies[i].position[1];
                let r_squared = dx * dx + dy * dy;
                let k = g / ((r_squared + epsilon_squared).powf(1.5));

                // a_ix
                a[i][0] += k * m_j * dx;
                // a_iy
                a[i][1] += k * m_j * dy;
                // a_jx
                a[j][0] -= k * m_i * dx;
                // a_jy
                a[j][1] -= k * m_i * dy;
            }
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Body;

    const MINIMUM_VALUE: f64 = 1e-10;
    const RELATIVE_PRECISION: f64 = 1e-5;

    fn within_tolerance(calculated: f64, expected: f64) -> bool {
        println!("calculated: {:e}, expected: {:e}", calculated, expected);

        // Negligible differences pass
        if expected.abs() < MINIMUM_VALUE && calculated.abs() < MINIMUM_VALUE {
            return true;
        }

        ((calculated - expected).abs()) / expected.abs() < RELATIVE_PRECISION
    }

    #[test]
    fn test_single_body_no_acceleration() {
        let gravity = NewtonGravity::new(1.0, 1.0);
        let bodies = vec![Body {
            id: 0,
            mass: 100.0,
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
        }];

        let accelerations = gravity.calculate_accelerations(&bodies);

        // A single body should have no acceleration
        assert_eq!(accelerations.len(), 1);
        assert_eq!(within_tolerance(accelerations[0][0], 0.0), true);
        assert_eq!(within_tolerance(accelerations[0][1], 0.0), true);
    }

    #[test]
    fn test_two_equal_masses_symmetric_acceleration() {
        let gravity = NewtonGravity::new(1.0, 0.0);
        let bodies = vec![
            Body {
                id: 0,
                mass: 12.5,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 1,
                mass: 12.5,
                position: [5.0, 0.0],
                velocity: [0.0, 0.0],
            },
        ];

        let accelerations = gravity.calculate_accelerations(&bodies);

        // Two equal masses should accelerate equally towards each other
        assert_eq!(accelerations.len(), 2);
        assert_eq!(within_tolerance(accelerations[0][0], 0.5), true);
        assert_eq!(within_tolerance(accelerations[0][1], 0.0), true);
        assert_eq!(within_tolerance(accelerations[1][0], -0.5), true);
        assert_eq!(within_tolerance(accelerations[1][1], 0.0), true);
    }

    #[test]
    fn test_two_different_masses_unequal_acceleration() {
        let gravity = NewtonGravity::new(1.0, 1.0);
        let bodies = vec![
            Body {
                id: 0,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 1,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
        ];

        let accelerations = gravity.calculate_accelerations(&bodies);

        // Small mass should accelerate faster than large mass
        assert_eq!(accelerations.len(), 2);
        // TODO acceleration assertions
    }

    #[test]
    fn test_three_bodies_varying_masses() {
        let gravity = NewtonGravity::new(1.0, 1.0);
        let bodies = vec![
            Body {
                id: 0,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 1,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 2,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
        ];

        let accelerations = gravity.calculate_accelerations(&bodies);

        // Three bodies with varying masses should each have accelerations from both other bodies
        assert_eq!(accelerations.len(), 3);
        // TODO acceleration assertions
    }

    #[test]
    fn test_five_bodies_varying_masses() {
        let gravity = NewtonGravity::new(6.674e-11, 1.0);
        let bodies = vec![
            Body {
                id: 0,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 1,
                mass: 1.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 2,
                mass: 0.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 3,
                mass: 0.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
            Body {
                id: 4,
                mass: 0.0,
                position: [0.0, 0.0],
                velocity: [0.0, 0.0],
            },
        ];

        let accelerations = gravity.calculate_accelerations(&bodies);

        // Five bodies should each have accelerations from all four other bodies
        assert_eq!(accelerations.len(), 5);
        // TODO acceleration assertions
    }
}
