use std::error::Error;

use crate::{
    body::{Bodies, Body},
    integrators::cpu::integrator::Integrator,
    output::SimulationSnapshot,
    simulation::{Simulation, SimulationParameters},
};

pub struct CpuSimulation {
    parameters: SimulationParameters,
    bodies: Bodies,
    integrator: Box<dyn Integrator>,
    time: f64,
}

impl CpuSimulation {
    pub fn new(
        parameters: SimulationParameters,
        bodies: &[Body],
        integrator: Box<dyn Integrator>,
    ) -> Self {
        // convert AoS body data to SoA format
        let bodies = Bodies::from(bodies);
        Self {
            parameters,
            bodies,
            integrator,
            time: 0.0,
        }
    }
}

impl Simulation for CpuSimulation {
    fn step(&mut self) -> Result<(), Box<dyn Error>> {
        self.integrator.step(&mut self.bodies);
        self.time += self.parameters.time_step;
        Ok(())
    }

    fn snapshot(&self) -> Result<SimulationSnapshot, Box<dyn Error>> {
        Ok(SimulationSnapshot {
            time: self.time,
            pos_x: self.bodies.pos_x.clone(),
            pos_y: self.bodies.pos_y.clone(),
            pos_z: self.bodies.pos_z.clone(),
        })
    }
}
