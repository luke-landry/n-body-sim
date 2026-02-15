mod benchmark;
mod cli;
mod constants;
mod gravity;
mod input;
mod integrators;
mod n_body_sim;
mod output;
mod simulation;

use crate::cli::Args;
use crate::n_body_sim::NBodySim;
use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.benchmark {
        return benchmark::run_benchmark(args);
    }

    let sim = NBodySim::new(args)?;
    sim.run()
}
