use crate::args::Args;
use crate::input;
use crate::output::SimulationDataWriter;
use crate::simulation::{Parameters, Simulator};
use std::error::Error;

pub struct NBodySim {
    simulator: Simulator,
    writer: SimulationDataWriter,
}

impl NBodySim {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        let bodies = input::load_bodies(&args.initial_conditions_path)?;
        let parameters = Parameters::new(
            args.time_step,
            args.num_steps,
            args.g_constant,
            args.softening_factor,
            args.theta,
            args.progress,
        );

        let gravity = args.gravity.create(&parameters, bodies.len());
        let integrator = args
            .integrator
            .create(gravity, parameters.time_step, bodies.len());

        let (tx, rx) = std::sync::mpsc::channel();
        let simulator = Simulator::new(bodies, parameters, integrator, Some(tx));
        let writer = SimulationDataWriter::new(args.output_data_path.clone(), rx);

        Ok(Self { simulator, writer })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let writer = self.writer;
        let mut simulator = self.simulator;

        // run writer and simulator in separate threads
        let writer_handle = std::thread::spawn(move || writer.run());
        let simulator_handle = std::thread::spawn(move || simulator.run());

        simulator_handle
            .join()
            .map_err(|_| "Simulator thread panicked")?;

        // wait for writer to finish after simulation completes
        writer_handle
            .join()
            .map_err(|_| "Writer thread panicked")?
            .map_err(|e| e.into())
    }
}
