use glam::DVec3;

use crate::{gravity::Gravity, integrators::Integrator, simulation::Body};

pub struct VelocityVerletIntegrator {
    gravity: Box<dyn Gravity>,
    time_step: f64,
    accelerations_current: Vec<DVec3>,
    accelerations_next: Vec<DVec3>,
}

impl VelocityVerletIntegrator {
    pub fn new(gravity: Box<dyn Gravity>, time_step: f64, num_bodies: usize) -> Self {
        VelocityVerletIntegrator {
            gravity,
            time_step,
            accelerations_current: vec![DVec3::ZERO; num_bodies],
            accelerations_next: vec![DVec3::ZERO; num_bodies],
        }
    }
}

/*
    Let r, v, a, be current position, velocity, and acceleration, respectively
    Let r_n, v_n, a_n, be the next position, velocity, and acceleration, respectively
    Let dt be the time step

    The standard velocity verlet algorithm is as follows:
        1. a = compute_acceleration(r)
        2. v_half = v + (0.5 * a * dt)
        3. r_n = r + (v_half * dt)
        4. a_n = compute_acceleration(r_n)
        5. v_n = v_half + (0.5 * a_n * dt)

    It achieves second-order accuracy by using the velocity at the half time step (v_half)
    to update the position, and then using the new acceleration (a_n) to update the velocity.
    This allows it to capture the motion of the system more accurately than first-order methods.

    However, by combining the velocity half steps (steps 2 and 5), we can get a simplified
    version of the algorithm that only requires 3 steps. This is only possible when
    acceleration is based solely on position, which is the case for gravitational forces.

    The simplified Velocity Verlet algorithm consists of the following steps:
        1. a = compute_acceleration(r)
        2. r_n = r + (v * dt) + (0.5 * a * dt^2)
        3. a_n = compute_acceleration(r_n)
        4. v_n = v + (0.5 * (a + a_n) * dt)

    The Velocity Verlet algorithm is a symplectic integrator, so it will conserve energy better
    than non-symplectic integrators. It is also a second-order method, which means it is more
    accurate than first-order methods like Euler's method, especially for larger time steps.

*/
impl Integrator for VelocityVerletIntegrator {
    fn step(&mut self, bodies: &mut [Body]) {
        // Calculate the current intermediate accelerations based on the current positions of the bodies
        self.accelerations_current.fill(DVec3::ZERO);
        self.gravity
            .calculate_accelerations(bodies, &mut self.accelerations_current);

        for (i, body) in bodies.iter_mut().enumerate() {
            body.position += (body.velocity * self.time_step)
                + (0.5 * self.accelerations_current[i] * self.time_step * self.time_step);
        }

        // The next accelerations vector will store the accelerations after the position update
        self.accelerations_next.fill(DVec3::ZERO);
        self.gravity
            .calculate_accelerations(bodies, &mut self.accelerations_next);

        for (i, body) in bodies.iter_mut().enumerate() {
            body.velocity +=
                0.5 * (self.accelerations_current[i] + self.accelerations_next[i]) * self.time_step;
        }
    }
}
