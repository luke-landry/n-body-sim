use clap::Parser;
use n_body_sim::args::Args;
use n_body_sim::n_body_sim::NBodySim;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let sim = NBodySim::new(args)?;
    sim.run()
}
