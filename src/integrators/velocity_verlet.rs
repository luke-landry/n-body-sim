use crate::{Integrator, gravity::Gravity, simulation::Body};

pub struct VelocityVerletIntegrator {
    gravity: Box<dyn Gravity>,
}

impl VelocityVerletIntegrator {
    pub fn new(gravity: Box<dyn Gravity>) -> Self {
        VelocityVerletIntegrator { gravity }
    }
}

impl Integrator for VelocityVerletIntegrator {
    fn step(&self, bodies: &mut Vec<Body>, time_step: f64) {
        // to be implemented
    }
}
