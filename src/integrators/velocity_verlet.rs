use crate::{
    gravity::Gravity,
    integrators::{Accelerations, Integrator, compute_acceleration},
    simulation::Bodies,
};

pub struct VelocityVerletIntegrator {
    gravity: Box<dyn Gravity>,
    time_step: f64,
    accelerations_current: Accelerations,
    accelerations_next: Accelerations,
}

impl VelocityVerletIntegrator {
    pub fn new(gravity: Box<dyn Gravity>, time_step: f64, num_bodies: usize) -> Self {
        VelocityVerletIntegrator {
            gravity,
            time_step,
            accelerations_current: Accelerations {
                ax: vec![0.0; num_bodies],
                ay: vec![0.0; num_bodies],
                az: vec![0.0; num_bodies],
            },
            accelerations_next: Accelerations {
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

    The standard velocity verlet algorithm is as follows:
        1. a_n = compute_acceleration(r_n)
        2. v_half = v_n + (0.5 * a_n * dt)
        3. r_(n+1) = r_n + (v_half * dt)
        4. a_(n+1) = compute_acceleration(r_(n+1))
        5. v_(n+1) = v_half + (0.5 * a_(n+1) * dt)

    It achieves second-order accuracy by using the velocity at the half time step (v_half)
    to update the position, and then using the new acceleration (a_(n+1)) to update the velocity.
    This allows it to capture the motion of the system more accurately than first-order methods.

    By combining the velocity half steps (steps 2 and 5), we can get a simplified
    version of the algorithm that only requires 3 steps. This is only possible when
    acceleration is based solely on position, which is the case for gravitational forces.

    The simplified Velocity Verlet algorithm consists of the following steps:
        1. a_n = compute_acceleration(r_n)
        2. r_(n+1) = r_n + (v_n * dt) + (0.5 * a_n * dt^2)
        3. a_(n+1) = compute_acceleration(r_(n+1))
        4. v_(n+1) = v_n + (0.5 * (a_n + a_(n+1)) * dt)

    The Velocity Verlet algorithm is a symplectic integrator, so it will conserve energy better
    than non-symplectic integrators. It is also a second-order method, which means it is more
    accurate than first-order methods like Euler's method, especially for larger time steps.
*/
impl Integrator for VelocityVerletIntegrator {
    fn step(&mut self, bodies: &mut Bodies) {
        self.accelerations_current.zero();
        self.accelerations_next.zero();

        let n = bodies.len();
        let dt = self.time_step;
        let dt2 = dt * dt;

        // Scopes to please the borrow checker when taking mutable references
        // to bodies in calculate_accelerations and the r and v variables
        {
            // 1. a_n = compute_acceleration(r_n)
            compute_acceleration(&*self.gravity, bodies, &mut self.accelerations_current);

            let (ax, ay, az) = self.accelerations_current.as_slices();
            let (_, rx, ry, rz, vx, vy, vz) = bodies.as_slices_mut();

            // 2. r_(n+1) = r_n + (v_n * dt) + (0.5 * a_n * dt^2)
            for i in 0..n {
                rx[i] += (vx[i] * dt) + (0.5 * ax[i] * dt2);
                ry[i] += (vy[i] * dt) + (0.5 * ay[i] * dt2);
                rz[i] += (vz[i] * dt) + (0.5 * az[i] * dt2);
            }
        }
        {
            // 3. a_(n+1) = compute_acceleration(r_(n+1))
            compute_acceleration(&*self.gravity, bodies, &mut self.accelerations_next);

            let (ax, ay, az) = self.accelerations_current.as_slices();
            let (ax_next, ay_next, az_next) = self.accelerations_next.as_slices();
            let (_, _, _, _, vx, vy, vz) = bodies.as_slices_mut();

            // 4. v_(n+1) = v_n + (0.5 * (a_n + a_(n+1)) * dt)
            for i in 0..n {
                vx[i] += 0.5 * (ax[i] + ax_next[i]) * dt;
                vy[i] += 0.5 * (ay[i] + ay_next[i]) * dt;
                vz[i] += 0.5 * (az[i] + az_next[i]) * dt;
            }
        }
    }
}
