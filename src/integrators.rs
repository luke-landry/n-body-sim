use glam::DVec3;

use crate::simulation::Body;

pub mod euler;
pub mod velocity_verlet;

pub trait Integrator {
    /// Advances the simulation by one time step
    fn step(&self, bodies: &mut [Body], time_step: f64, accelerations: &mut [DVec3]);
}
