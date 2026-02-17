use crate::{
    gravity::Gravity,
    integrators::{Accelerations, Integrator, compute_acceleration},
    simulation::Bodies,
};

pub struct RungeKuttaIntegrator {
    gravity: Box<dyn Gravity>,
    time_step: f64,
    accelerations: Accelerations,
    k1: RK4K,
    k2: RK4K,
    k3: RK4K,
    k4: RK4K,
    temp_bodies: Bodies,
}

impl RungeKuttaIntegrator {
    pub fn new(gravity: Box<dyn Gravity>, time_step: f64, num_bodies: usize) -> Self {
        RungeKuttaIntegrator {
            gravity,
            time_step,
            accelerations: Accelerations {
                ax: vec![0.0; num_bodies],
                ay: vec![0.0; num_bodies],
                az: vec![0.0; num_bodies],
            },
            k1: RK4K::new(num_bodies),
            k2: RK4K::new(num_bodies),
            k3: RK4K::new(num_bodies),
            k4: RK4K::new(num_bodies),
            temp_bodies: Bodies {
                masses: vec![0.0; num_bodies],
                pos_x: vec![0.0; num_bodies],
                pos_y: vec![0.0; num_bodies],
                pos_z: vec![0.0; num_bodies],
                vel_x: vec![0.0; num_bodies],
                vel_y: vec![0.0; num_bodies],
                vel_z: vec![0.0; num_bodies],
            },
        }
    }
}

/// Helper to store a k value for RK4, which consists of
/// intermediate position and velocity values for each body
pub struct RK4K {
    pub rx: Vec<f64>,
    pub ry: Vec<f64>,
    pub rz: Vec<f64>,

    // The k values for velocity are actually the intermediate accelerations
    // so they can be saved in a Accelerations struct internally
    pub v: Accelerations,
}

impl RK4K {
    pub fn new(n: usize) -> Self {
        Self {
            rx: vec![0.0; n],
            ry: vec![0.0; n],
            rz: vec![0.0; n],
            v: Accelerations {
                ax: vec![0.0; n],
                ay: vec![0.0; n],
                az: vec![0.0; n],
            },
        }
    }

    /// Return tuple of references to the 3D position and velocity component slices of all bodies
    pub fn as_slices(&mut self) -> (&[f64], &[f64], &[f64], &[f64], &[f64], &[f64]) {
        (
            &self.rx, &self.ry, &self.rz, &self.v.ax, &self.v.ay, &self.v.az,
        )
    }

    /// Return tuple of mutable references to the 3D position and velocity component slices of all bodies
    pub fn as_slices_mut(
        &mut self,
    ) -> (
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
    ) {
        (
            &mut self.rx,
            &mut self.ry,
            &mut self.rz,
            &mut self.v.ax,
            &mut self.v.ay,
            &mut self.v.az,
        )
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
        k1_v = compute_acceleration(r_n)
        k2_r = v_n + (dt/2)*k1_v
        k2_v = compute_acceleration(r_n + (dt/2)*k1_r)
        k3_r = v_n + (dt/2)*k2_v
        k3_v = compute_acceleration(r_n + (dt/2)*k2_r)
        k4_r = v_n + dt*k3_v
        k4_v = compute_acceleration(r_n + dt*k3_r)

    So, the full steps for the RK4 method are:
        1. k1_r = v_n
        2. k1_v = compute_acceleration(r_n)
        3. k2_r = v_n + (dt/2)*k1_v
        4. k2_v = compute_acceleration(r_n + (dt/2)*k1_r)
        5. k3_r = v_n + (dt/2)*k2_v
        6. k3_v = compute_acceleration(r_n + (dt/2)*k2_r)
        7. k4_r = v_n + dt*k3_v
        8. k4_v = compute_acceleration(r_n + dt*k3_r)
        9. r_(n+1) = r_n + (dt/6) * (k1_r + 2*k2_r + 2*k3_r + k4_r)
        10. v_(n+1) = v_n + (dt/6) * (k1_v + 2*k2_v + 2*k3_v + k4_v)
*/

impl Integrator for RungeKuttaIntegrator {
    fn step(&mut self, bodies: &mut Bodies) {
        self.accelerations.zero();

        let n = bodies.len();
        let dt = self.time_step;

        // 1. k1_r = v_n
        self.k1.rx.copy_from_slice(&bodies.vel_x);
        self.k1.ry.copy_from_slice(&bodies.vel_y);
        self.k1.rz.copy_from_slice(&bodies.vel_z);

        // 2. k1_v = compute_acceleration(r_n)
        compute_acceleration(&mut *self.gravity, bodies, &mut self.k1.v);

        {
            let (_, _, _, _, vx, vy, vz) = bodies.as_slices();
            let (_, _, _, k1_vx, k1_vy, k1_vz) = self.k1.as_slices();
            let (k2_rx, k2_ry, k2_rz, _, _, _) = self.k2.as_slices_mut();

            // 3. k2_r = v_n + (dt/2)*k1_v
            for i in 0..n {
                k2_rx[i] = vx[i] + (dt / 2.0) * k1_vx[i];
                k2_ry[i] = vy[i] + (dt / 2.0) * k1_vy[i];
                k2_rz[i] = vz[i] + (dt / 2.0) * k1_vz[i];
            }
        }
        {
            let (masses, rx, ry, rz, vx, vy, vz) = bodies.as_slices();
            let (k1_rx, k1_ry, k1_rz, k1_vx, k1_vy, k1_vz) = self.k1.as_slices();

            // Need to create temporary bodies with the intermediate positions and velocities to compute the k2_v values
            self.temp_bodies.masses.copy_from_slice(masses);
            let (_, temp_rx, temp_ry, temp_rz, temp_vx, temp_vy, temp_vz) =
                self.temp_bodies.as_slices_mut();
            for i in 0..n {
                temp_rx[i] = rx[i] + (dt / 2.0) * k1_rx[i];
                temp_ry[i] = ry[i] + (dt / 2.0) * k1_ry[i];
                temp_rz[i] = rz[i] + (dt / 2.0) * k1_rz[i];
                temp_vx[i] = vx[i] + (dt / 2.0) * k1_vx[i];
                temp_vy[i] = vy[i] + (dt / 2.0) * k1_vy[i];
                temp_vz[i] = vz[i] + (dt / 2.0) * k1_vz[i];
            }

            // 4. k2_v = compute_acceleration(r_n + (dt/2)*k1_r)
            compute_acceleration(&mut *self.gravity, &self.temp_bodies, &mut self.k2.v);
        }
        {
            let (_, _, _, _, vx, vy, vz) = bodies.as_slices();
            let (k2_vx, k2_vy, k2_vz) = self.k2.v.as_slices();
            let (k3_rx, k3_ry, k3_rz, _, _, _) = self.k3.as_slices_mut();

            // 5. k3_r = v_n + (dt/2)*k2_v
            for i in 0..n {
                k3_rx[i] = vx[i] + (dt / 2.0) * k2_vx[i];
                k3_ry[i] = vy[i] + (dt / 2.0) * k2_vy[i];
                k3_rz[i] = vz[i] + (dt / 2.0) * k2_vz[i];
            }
        }
        {
            let (masses, rx, ry, rz, vx, vy, vz) = bodies.as_slices();
            let (k2_rx, k2_ry, k2_rz, k2_vx, k2_vy, k2_vz) = self.k2.as_slices();

            // Need to create temporary bodies with the intermediate positions and velocities to compute the k3_v values
            self.temp_bodies.masses.copy_from_slice(masses);
            let (_, temp_rx, temp_ry, temp_rz, temp_vx, temp_vy, temp_vz) =
                self.temp_bodies.as_slices_mut();
            for i in 0..n {
                temp_rx[i] = rx[i] + (dt / 2.0) * k2_rx[i];
                temp_ry[i] = ry[i] + (dt / 2.0) * k2_ry[i];
                temp_rz[i] = rz[i] + (dt / 2.0) * k2_rz[i];
                temp_vx[i] = vx[i] + (dt / 2.0) * k2_vx[i];
                temp_vy[i] = vy[i] + (dt / 2.0) * k2_vy[i];
                temp_vz[i] = vz[i] + (dt / 2.0) * k2_vz[i];
            }

            // 6. k3_v = compute_acceleration(r_n + (dt/2)*k2_r)
            compute_acceleration(&mut *self.gravity, &self.temp_bodies, &mut self.k3.v);
        }
        {
            let (_, _, _, _, vx, vy, vz) = bodies.as_slices();
            let (k3_vx, k3_vy, k3_vz) = self.k3.v.as_slices();
            let (k4_rx, k4_ry, k4_rz, _, _, _) = self.k4.as_slices_mut();

            // 7. k4_r = v_n + dt*k3_v
            for i in 0..n {
                k4_rx[i] = vx[i] + dt * k3_vx[i];
                k4_ry[i] = vy[i] + dt * k3_vy[i];
                k4_rz[i] = vz[i] + dt * k3_vz[i];
            }
        }
        {
            let (masses, rx, ry, rz, vx, vy, vz) = bodies.as_slices();
            let (k3_rx, k3_ry, k3_rz, k3_vx, k3_vy, k3_vz) = self.k3.as_slices();

            // Need to create temporary bodies with the intermediate positions and velocities to compute the k4_v values
            self.temp_bodies.masses.copy_from_slice(masses);
            let (_, temp_rx, temp_ry, temp_rz, temp_vx, temp_vy, temp_vz) =
                self.temp_bodies.as_slices_mut();
            for i in 0..n {
                temp_rx[i] = rx[i] + dt * k3_rx[i];
                temp_ry[i] = ry[i] + dt * k3_ry[i];
                temp_rz[i] = rz[i] + dt * k3_rz[i];
                temp_vx[i] = vx[i] + dt * k3_vx[i];
                temp_vy[i] = vy[i] + dt * k3_vy[i];
                temp_vz[i] = vz[i] + dt * k3_vz[i];
            }

            // 8. k4_v = compute_acceleration(r_n + dt*k3_r)
            compute_acceleration(&mut *self.gravity, &self.temp_bodies, &mut self.k4.v);
        }

        let (_, rx, ry, rz, vx, vy, vz) = bodies.as_slices_mut();
        let (k1_rx, k1_ry, k1_rz, k1_vx, k1_vy, k1_vz) = self.k1.as_slices();
        let (k2_rx, k2_ry, k2_rz, k2_vx, k2_vy, k2_vz) = self.k2.as_slices();
        let (k3_rx, k3_ry, k3_rz, k3_vx, k3_vy, k3_vz) = self.k3.as_slices();
        let (k4_rx, k4_ry, k4_rz, k4_vx, k4_vy, k4_vz) = self.k4.as_slices();

        // 9. r_(n+1) = r_n + (dt/6) * (k1_r + 2*k2_r + 2*k3_r + k4_r)
        for i in 0..n {
            rx[i] += (dt / 6.0) * (k1_rx[i] + (2.0 * k2_rx[i]) + (2.0 * k3_rx[i]) + k4_rx[i]);
            ry[i] += (dt / 6.0) * (k1_ry[i] + (2.0 * k2_ry[i]) + (2.0 * k3_ry[i]) + k4_ry[i]);
            rz[i] += (dt / 6.0) * (k1_rz[i] + (2.0 * k2_rz[i]) + (2.0 * k3_rz[i]) + k4_rz[i]);
        }

        // 10. v_(n+1) = v_n + (dt/6) * (k1_v + 2*k2_v + 2*k3_v + k4_v)
        for i in 0..n {
            vx[i] += (dt / 6.0) * (k1_vx[i] + (2.0 * k2_vx[i]) + (2.0 * k3_vx[i]) + k4_vx[i]);
            vy[i] += (dt / 6.0) * (k1_vy[i] + (2.0 * k2_vy[i]) + (2.0 * k3_vy[i]) + k4_vy[i]);
            vz[i] += (dt / 6.0) * (k1_vz[i] + (2.0 * k2_vz[i]) + (2.0 * k3_vz[i]) + k4_vz[i]);
        }
    }
}
