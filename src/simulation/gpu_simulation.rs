use std::error::Error;

use crate::{
    body::{Bodies, Body},
    gpu::{cuda_manager::CudaManager, device_bodies::DeviceBodies},
    gravity::GpuGravity,
    integrators::gpu::gpu_integrator::GpuIntegrator,
    output::SimulationSnapshot,
    simulation::{Simulation, SimulationParameters},
};

pub struct GpuSimulation {
    gpu: CudaManager,
    parameters: SimulationParameters,
    bodies: DeviceBodies,
    gravity: Box<dyn GpuGravity>,
    integrator: Box<dyn GpuIntegrator>,
    time: f64,
}

impl GpuSimulation {
    pub fn new(
        parameters: SimulationParameters,
        bodies: &[Body],
        gravity: Box<dyn GpuGravity>,
        integrator: Box<dyn GpuIntegrator>,
    ) -> Result<Self, Box<dyn Error>> {
        // initialize GPU and upload body data
        let gpu = CudaManager::new()?;
        gpu.gpu_init_check()?;
        let bodies = DeviceBodies::new(&gpu, &Bodies::from(bodies))?;

        Ok(Self {
            gpu,
            parameters,
            bodies,
            gravity,
            integrator,
            time: 0.0,
        })
    }
}

impl Simulation for GpuSimulation {
    fn step(&mut self) -> Result<(), Box<dyn Error>> {
        self.integrator
            .step(&self.gpu, &mut self.bodies, &*self.gravity)?;
        self.time += self.parameters.time_step;
        Ok(())
    }

    fn snapshot(&self) -> Result<SimulationSnapshot, Box<dyn Error>> {
        // ensure all GPU work is complete before downloading data
        self.gpu.stream.synchronize()?;

        // clones body data from GPU back to CPU for output
        let bodies = self.bodies.download(&self.gpu)?;

        Ok(SimulationSnapshot {
            time: self.time,
            pos_x: bodies.pos_x,
            pos_y: bodies.pos_y,
            pos_z: bodies.pos_z,
        })
    }
}
