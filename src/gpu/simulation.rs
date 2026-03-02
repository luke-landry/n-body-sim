use std::sync::mpsc::Sender;

use crate::{
    gpu::{
        CudaManager, device_bodies::DeviceBodies, gravity::GpuGravity, integrators::GpuIntegrator,
    },
    output::SimulationData,
    simulation::{Parameters, Simulation},
};

pub struct GpuSimulator {
    gpu: CudaManager,
    parameters: Parameters,
    bodies: DeviceBodies,
    gravity: Box<dyn GpuGravity>,
    integrator: Box<dyn GpuIntegrator>,
    tx: Option<Sender<SimulationData>>,
}

impl GpuSimulator {
    // Max size of a batch of simulation data to send in bytes
    const BATCH_SIZE_BYTES: usize = 16000000;

    // Number of simulation data records that fit in a batch of BATCH_SIZE_BYTES bytes
    const BATCH_SIZE: usize = Self::BATCH_SIZE_BYTES / SimulationData::RECORD_SIZE_BYTES;

    pub fn new(
        gpu: CudaManager,
        parameters: Parameters,
        bodies: DeviceBodies,
        gravity: Box<dyn GpuGravity>,
        integrator: Box<dyn GpuIntegrator>,
        tx: Option<Sender<SimulationData>>,
    ) -> Self {
        Self {
            gpu,
            parameters,
            bodies,
            gravity,
            integrator,
            tx,
        }
    }
}

impl Simulation for GpuSimulator {
    fn run(&mut self) {
        let mut buffer = SimulationData::with_capacity(Self::BATCH_SIZE);
        let one_percent_steps = (self.parameters.num_steps / 100).max(1);
        let mut time = 0.0;

        // main simulation loop
        for step in 0..self.parameters.num_steps {
            if let Some(tx) = &self.tx {
                self.gpu
                    .stream
                    .synchronize()
                    .expect("Failed to synchronize GPU");

                // download body data from GPU to CPU for output
                let bodies = self
                    .bodies
                    .download(&self.gpu)
                    .expect("Failed to download bodies from GPU");

                buffer.extend_from_step(time, &bodies.pos_x, &bodies.pos_y, &bodies.pos_z);

                if buffer.len() >= Self::BATCH_SIZE {
                    // avoid cloning the buffer by replacing it with an
                    // empty one and sending the full batch to the output channel
                    let batch = std::mem::replace(
                        &mut buffer,
                        SimulationData::with_capacity(Self::BATCH_SIZE),
                    );
                    if let Err(e) = tx.send(batch) {
                        eprintln!(
                            "Failed to send simulation data batch: {}\nHalting simulation.",
                            e
                        );
                        return;
                    }
                }
            }

            self.integrator
                .step(&self.gpu, &mut self.bodies, &*self.gravity)
                .expect("Failed to perform simulation step on GPU");

            time += self.parameters.time_step;

            if self.parameters.progress && step % one_percent_steps == 0 {
                println!("{}", (step * 100) / self.parameters.num_steps);
            }
        }

        self.gpu
            .stream
            .synchronize()
            .expect("Failed to synchronize GPU");

        // download body data from GPU to CPU for output
        let bodies = self
            .bodies
            .download(&self.gpu)
            .expect("Failed to download bodies from GPU");

        // record final state and send remaining data
        buffer.extend_from_step(time, &bodies.pos_x, &bodies.pos_y, &bodies.pos_z);
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(buffer) {
                eprintln!("Failed to send final simulation data batch: {}", e);
                return;
            }
        }
    }
}
