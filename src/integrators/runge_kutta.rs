use glam::DVec3;

use crate::{gravity::Gravity, integrators::Integrator, simulation::Body};

pub struct RungeKuttaIntegrator {
    gravity: Box<dyn Gravity>,
    time_step: f64,
    k1_r: Vec<DVec3>,
    k1_v: Vec<DVec3>,
    k2_r: Vec<DVec3>,
    k2_v: Vec<DVec3>,
    k3_r: Vec<DVec3>,
    k3_v: Vec<DVec3>,
    k4_r: Vec<DVec3>,
    k4_v: Vec<DVec3>,
    temp_bodies: Vec<Body>,
}

impl RungeKuttaIntegrator {
    pub fn new(gravity: Box<dyn Gravity>, time_step: f64, num_bodies: usize) -> Self {
        RungeKuttaIntegrator {
            gravity,
            time_step,
            k1_r: vec![DVec3::ZERO; num_bodies],
            k1_v: vec![DVec3::ZERO; num_bodies],
            k2_r: vec![DVec3::ZERO; num_bodies],
            k2_v: vec![DVec3::ZERO; num_bodies],
            k3_r: vec![DVec3::ZERO; num_bodies],
            k3_v: vec![DVec3::ZERO; num_bodies],
            k4_r: vec![DVec3::ZERO; num_bodies],
            k4_v: vec![DVec3::ZERO; num_bodies],
            temp_bodies: vec![
                Body {
                    id: 0,
                    mass: 0.0,
                    position: DVec3::ZERO,
                    velocity: DVec3::ZERO,
                };
                num_bodies
            ],
        }
    }
}

/*
    The fourth order Runge-Kutta (RK4) method is given by the following formula:
        y_(n+1) = y_n + (1/6) * (k1 + 2*k2 + 2*k3 + k4)
    where:
        k1 = f(t_n, y_n)
        k2 = f(t_n + h/2, y_n + (h/2)*k1)
        k3 = f(t_n + h/2, y_n + (h/2)*k2)
        k4 = f(t_n + h, y_n + h*k3)

    In this formula,
        y represents the state of the system
        t is time
        h is the time step
        k is the slope (derivative) of the state at different points in time
        f is the function that computes the derivative based on the current state and time.

    Let r_n, v_n, a_n be the current position, velocity, and acceleration, respectively
    Let r_(n+1), v_(n+1), a_(n+1) be the next position, velocity, and acceleration, respectively
    Let dt be the time step

    Our system has two state variables: position (r) and velocity (v), so the state equation
        y_(n+1) = y_n + (dt/6) * (k1 + 2*k2 + 2*k3 + k4)
    can be split into two equations:
        r_(n+1) = r_n + (dt/6) * (k1_r + 2*k2_r + 2*k3_r + k4_r)
        v_(n+1) = v_n + (dt/6) * (k1_v + 2*k2_v + 2*k3_v + k4_v)

    Noting that the derivative of position is velocity and the derivative of velocity is acceleration, we can define:
        k1_r = v_n
        k1_v = compute_acceleration(r_n, mass)
        k2_r = v_n + (dt/2)*k1_v
        k2_v = compute_acceleration(r_n + (dt/2)*k1_r, mass)
        k3_r = v_n + (dt/2)*k2_v
        k3_v = compute_acceleration(r_n + (dt/2)*k2_r, mass)
        k4_r = v_n + dt*k3_v
        k4_v = compute_acceleration(r_n + dt*k3_r, mass)

*/

impl Integrator for RungeKuttaIntegrator {
    fn step(&mut self, bodies: &mut [Body]) {
        // TODO iterations like below should be optimized in the future
        // by using struct of arrays (SoA) instead of array of structs (AoS)
        // since it would allow very simple element-wise operations on vectors
        // which improves cache locality and allows for better SIMD optimization.
        // In this case, gravity interface would change to take vectors of positions
        // and masses instead of a vector of bodies.
        //
        // Trying to implement RK4 without SoA would be very inefficient since it
        // requires multiple iterations over the bodies to compute the intermediate k values,
        // which involves a lot of redundant calculations and memory accesses. Will implement
        // RK4 after refactoring the code to use SoA,
        //
        //
        // self.k1_r.fill(DVec3::ZERO);
        // self.k1_v.fill(DVec3::ZERO);
        // self.k2_r.fill(DVec3::ZERO);
        // self.k2_v.fill(DVec3::ZERO);
        // self.k3_r.fill(DVec3::ZERO);
        // self.k3_v.fill(DVec3::ZERO);
        // self.k4_r.fill(DVec3::ZERO);
        // self.k4_v.fill(DVec3::ZERO);

        // let v_n: Vec<DVec3> = bodies.iter().map(|body| body.velocity).collect();

        // self.k1_r = v_n.clone();
        // self.gravity.calculate_accelerations(bodies, &mut self.k1_v);

        // for i in 0..bodies.len() {
        //     self.k2_r[i] = v_n[i] + (self.time_step / 2.0) * self.k1_v[i];
        // }
        // self.temp_bodies
        //     .iter_mut()
        //     .zip(bodies.iter())
        //     .for_each(|(temp_body, body)| {
        //         temp_body.id = body.id;
        //         temp_body.mass = body.mass;
        //         temp_body.position = body.position
        //             + (self.time_step / 2.0)
        //                 * self.k1_r[bodies.iter().position(|b| b.id == body.id).unwrap()];
        //         temp_body.velocity = body.velocity
        //     });
        // self.gravity
        //     .calculate_accelerations(&self.temp_bodies, &mut self.k2_v);
    }
}
