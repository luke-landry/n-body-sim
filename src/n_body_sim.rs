use crate::cli::Args;
use crate::input;
use crate::output;
use crate::simulation::{Parameters, Simulator};
use clap::Parser;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    println!("n-body-sim");

    let args = Args::parse();

    let bodies = input::load_bodies(&args.initial_conditions_path)?;

    let parameters = Parameters::new(
        args.time_step,
        args.num_steps,
        args.g_constant,
        args.softening_factor,
        args.theta,
    );

    let gravity = args.gravity.create(&parameters);

    let integrator = args.integrator.create(gravity);

    let mut simulator = Simulator::new(bodies, parameters, integrator);

    let data = simulator.run();

    match args.output_data_path {
        Some(path) => {
            output::save_to_csv(&path, data)?;
        }
        None => {
            output::print_data(data);
        }
    }

    Ok(())
}
