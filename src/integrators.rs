use crate::{body::Bodies, gravity::Gravity};

pub mod euler;
pub mod runge_kutta;
pub mod velocity_verlet;

/// SoA representation of accelerations
pub struct Accelerations {
    pub ax: Vec<f64>,
    pub ay: Vec<f64>,
    pub az: Vec<f64>,
}

impl Accelerations {
    pub fn zero(&mut self) {
        self.ax.fill(0.0);
        self.ay.fill(0.0);
        self.az.fill(0.0);
    }

    /// Returns a tuple of references to the acceleration components slices
    pub fn as_slices(&self) -> (&[f64], &[f64], &[f64]) {
        (&self.ax, &self.ay, &self.az)
    }

    /// Returns a tuple of mutable references to the 3D acceleration component slices
    pub fn as_mut_slices(&mut self) -> (&mut [f64], &mut [f64], &mut [f64]) {
        (&mut self.ax, &mut self.ay, &mut self.az)
    }
}

/// Computes the accelerations for all bodies based on their positions and masses using the provided gravity model
pub fn compute_acceleration(
    gravity: &mut dyn Gravity,
    bodies: &Bodies,
    accelerations: &mut Accelerations,
) {
    accelerations.zero();
    gravity.calculate_accelerations(
        &bodies.masses,
        &bodies.pos_x,
        &bodies.pos_y,
        &bodies.pos_z,
        &mut accelerations.ax,
        &mut accelerations.ay,
        &mut accelerations.az,
    );
}

pub trait Integrator: Send {
    /// Advances the simulation by one time step
    fn step(&mut self, bodies: &mut Bodies);
}
