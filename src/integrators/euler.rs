use crate::{Integrator, gravity::Gravity, simulation::Body};

pub struct EulerIntegrator {
    gravity: Box<dyn Gravity>,
}

impl EulerIntegrator {
    pub fn new(gravity: Box<dyn Gravity>) -> Self {
        EulerIntegrator { gravity }
    }
}

impl Integrator for EulerIntegrator {
    fn step(&self, bodies: &mut Vec<Body>, time_step: f64) {}
}
