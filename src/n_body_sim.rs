use crate::cli::Args;
use crate::input;
use crate::output;
use crate::simulation::{Parameters, Simulator};
use clap::Parser;
use std::error::Error;

pub struct NBodySim {
    args: Args,
    simulator: Simulator,
}

impl NBodySim {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let args = Args::parse();
        let bodies = input::load_bodies(&args.initial_conditions_path)?;
        let parameters = Parameters::new(
            args.time_step,
            args.num_steps,
            args.g_constant,
            args.softening_factor,
            args.theta,
            args.progress,
        );

        let gravity = args.gravity.create(&parameters);
        let integrator = args
            .integrator
            .create(gravity, parameters.time_step, bodies.len());

        let simulator = Simulator::new(bodies, parameters, integrator);

        Ok(Self { args, simulator })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let mut simulator = self.simulator;
        let data = simulator.run();
        match self.args.output_data_path {
            Some(path) => {
                output::save_to_csv(&path, data)?;
            }
            None => {
                output::print_data(data);
            }
        }

        Ok(())
    }
}
