use crate::simulation::Body;

pub mod euler;

pub trait Integrator {
    /// Advances the simulation by one time step
    fn step(&self, bodies: &mut Vec<Body>, time_step: f64);
}
