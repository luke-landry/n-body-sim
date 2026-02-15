use crate::gravity::Accelerations;
use crate::simulation::Bodies;
use crate::{gravity::Gravity, integrators::Integrator};

pub struct EulerIntegrator {
    gravity: Box<dyn Gravity>,
    time_step: f64,
    accelerations: Accelerations,
}

impl EulerIntegrator {
    pub fn new(gravity: Box<dyn Gravity>, time_step: f64, num_bodies: usize) -> Self {
        EulerIntegrator {
            gravity,
            time_step,
            accelerations: Accelerations {
                ax: vec![0.0; num_bodies],
                ay: vec![0.0; num_bodies],
                az: vec![0.0; num_bodies],
            },
        }
    }
}

/*
    Let r_n, v_n, a_n be the current position, velocity, and acceleration, respectively
    Let r_(n+1), v_(n+1), a_(n+1) be the next position, velocity, and acceleration, respectively
    Let dt be the time step

    The standard Euler method would be as follows:
        1. a_n = compute_acceleration(r_n)
        2. r_(n+1) = r_n + v_n * dt
        3. v_(n+1) = v_n + a_n * dt

    Next position is calculated first, then next velocity. This means current
    acceleration is not taken into account for position updates, which makes simulations
    unstable because this "adds energy" to the system.

    For example, in a two body system, with one orbiting another, the orbiting body
    each time step will move in a straight line tangent to the curve of its orbit.
    By the time it updates velocity, it has already "overshot" the orbit. This adds
    artificial energy, causing the orbiting body to spiral outward forever.

    To avoid this, we use the semi-implicit Euler method:
        1. a_n = compute_acceleration(r_n)
        2. v_(n+1) = v_n + a_n * dt
        3. r_(n+1) = r_n + v_(n+1) * dt

    New velocity is calculated first, then the next position is calculated using the
    next velocity. This sequence takes into account the current acceleration when
    updating position. So, in the orbiting two-body system example, this would result
    in the body being pulled more towards the center of gravity, which helps correct
    for the overshoot. The semi-implicit Euler method is a "symplectic integrator"
    which means it will maintain the long-term stability of the system by preventing
    energy from drifting away over time.
*/
impl Integrator for EulerIntegrator {
    fn step(&mut self, bodies: &mut Bodies) {
        self.accelerations.zero();

        let n = bodies.len();
        let dt = self.time_step;

        self.gravity
            .calculate_accelerations(bodies, &mut self.accelerations);
        let ax = &self.accelerations.ax;
        let ay = &self.accelerations.ay;
        let az = &self.accelerations.az;

        let rx = &mut bodies.pos_x;
        let ry = &mut bodies.pos_y;
        let rz = &mut bodies.pos_z;

        let vx = &mut bodies.vel_x;
        let vy = &mut bodies.vel_y;
        let vz = &mut bodies.vel_z;

        // v_(n+1) = v_n + a_n * dt
        for i in 0..n {
            vx[i] += ax[i] * dt;
            vy[i] += ay[i] * dt;
            vz[i] += az[i] * dt;
        }

        // r_(n+1) = r_n + v_(n+1) * dt
        for i in 0..n {
            rx[i] += vx[i] * dt;
            ry[i] += vy[i] * dt;
            rz[i] += vz[i] * dt;
        }
    }
}
