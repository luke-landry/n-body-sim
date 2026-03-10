use crate::args::Args;
use crate::body::Body;
use crate::input;
use crate::output::SimulationDataWriter;
use crate::simulation::cpu_simulation::CpuSimulation;
use crate::simulation::gpu_simulation::GpuSimulation;
use crate::simulation::{Simulation, SimulationParameters};
use crate::simulation_runner::SimulationRunner;
use std::error::Error;

pub struct NBodySim {
    simulation: Box<dyn Simulation>,
    runner: SimulationRunner,
    writer: SimulationDataWriter,
}

impl NBodySim {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        let bodies = input::load_bodies(&args.initial_conditions_path)?;
        let parameters = SimulationParameters::new(
            args.time_step,
            args.num_steps,
            args.g_constant,
            args.softening_factor,
            args.theta,
            args.progress,
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let simulation = Self::create_simulation(&args, parameters, bodies);
        let runner = SimulationRunner::new(args.num_steps, args.progress, tx);
        let writer = SimulationDataWriter::new(args.output_data_path.clone(), rx);
        Ok(Self {
            simulation,
            runner,
            writer,
        })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let writer = self.writer;
        let mut runner = self.runner;
        let simulation = self.simulation;

        // run writer and simulator in separate threads
        let writer_handle = std::thread::spawn(move || writer.run());
        let simulation_handle =
            std::thread::spawn(move || runner.run(simulation).map_err(|e| e.to_string()));

        simulation_handle
            .join()
            .map_err(|_| "Simulation thread panicked")??;

        // wait for writer to finish after simulation completes
        writer_handle
            .join()
            .map_err(|_| "Writer thread panicked")?
            .map_err(|e| e.into())
    }

    fn create_simulation(
        args: &Args,
        parameters: SimulationParameters,
        bodies: Vec<Body>,
    ) -> Box<dyn Simulation> {
        if args.gpu {
            let gravity = args.gravity.gpu_create(&parameters);
            let integrator = args.integrator.gpu_create(parameters.time_step);
            Box::new(
                GpuSimulation::new(parameters, bodies.as_slice(), gravity, integrator)
                    .expect("Failed to initialize GPU simulation"),
            )
        } else {
            let gravity = args.gravity.create(&parameters, bodies.len());
            let integrator = args
                .integrator
                .create(gravity, parameters.time_step, bodies.len());
            Box::new(CpuSimulation::new(
                parameters,
                bodies.as_slice(),
                integrator,
            ))
        }
    }
}
