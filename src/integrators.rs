use crate::simulation::Body;

pub mod euler;
pub mod runge_kutta;
pub mod velocity_verlet;

pub trait Integrator {
    /// Advances the simulation by one time step
    fn step(&mut self, bodies: &mut [Body]);
}
