use glam::DVec3;

use crate::{gravity::Gravity, integrators::Integrator, simulation::Body};

pub struct EulerIntegrator {
    gravity: Box<dyn Gravity>,
}

impl EulerIntegrator {
    pub fn new(gravity: Box<dyn Gravity>) -> Self {
        EulerIntegrator { gravity }
    }
}

/*
    Let r, v, a, be current position, velocity, and acceleration, respectively
    Let r_n, v_n, a_n, be the next position, velocity, and acceleration, respectively
    Let dt be the time step

    The standard Euler method would be as follows:
        1. r_n = r + v*dt
        2. v_n = v + a*dt

    Next position is calculated first, then next velocity. This means current
    acceleration is not taken into account for position updates, which makes simulations
    unstable because this "adds energy" to the system.

    For example, in a two body system, with one orbiting another, the orbiting body
    each time step will move in a straight line tangent to the curve of its orbit.
    By the time it updates velocity, it has already "overshot" the orbit. This adds
    artificial energy, causing the orbiting body to spiral outward forever.

    To avoid this, we use the semi-implicit Euler method:
        1. v_n = v + a*dt
        2. r_n = r + v_n*dt

    New velocity is calculated first, then the next position is calculated using the
    next velocity. This sequence takes into account the current acceleration when
    updating position. So, in the orbiting two-body system example, this would result
    in the body being pulled more towards the center of gravity, which helps correct
    for the overshoot. The semi-implicit Euler method is a "symplectic integrator"
    which means it will maintain the long-term stability of the system by preventing
    energy from drifting away over time.
*/
impl Integrator for EulerIntegrator {
    fn step(&self, bodies: &mut [Body], time_step: f64, accelerations: &mut [DVec3]) {
        accelerations.fill(DVec3::ZERO);
        self.gravity.calculate_accelerations(bodies, accelerations);
        for i in 0..bodies.len() {
            bodies[i].velocity += accelerations[i] * time_step;
            bodies[i].position += bodies[i].velocity * time_step;
        }
    }
}
