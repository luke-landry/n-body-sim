use crate::{Integrator, simulation::Body};

pub struct EulerIntegrator;

impl Integrator for EulerIntegrator {
    fn step(&self, bodies: &mut Vec<Body>, time_step: f64){

    }
}
