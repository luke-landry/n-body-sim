mod cli;
mod constants;
mod gravity;
mod input;
mod integrators;
mod n_body_sim;
mod output;
mod simulation;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    n_body_sim::run()
}
