use glam::DVec3;

use crate::{gravity::Gravity, integrators::Integrator, simulation::Body};

pub struct VelocityVerletIntegrator {
    gravity: Box<dyn Gravity>,
}

impl VelocityVerletIntegrator {
    pub fn new(gravity: Box<dyn Gravity>) -> Self {
        VelocityVerletIntegrator { gravity }
    }
}

impl Integrator for VelocityVerletIntegrator {
    fn step(&self, bodies: &mut [Body], time_step: f64, accelerations: &mut [DVec3]) {
        // to be implemented
    }
}
