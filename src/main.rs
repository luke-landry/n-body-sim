mod physics;

use physics::InitialCondition;
use clap::Parser;
use csv::Reader;
use std::{error::Error, path::PathBuf};

#[derive(Parser)]
struct Args {

    #[arg(short, long)]
    initial_conditions: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>>{
    println!("n-body-sim");
    
    let args = Args::parse();
    let mut rdr = Reader::from_path(args.initial_conditions)?;
    let initial_conditions: Vec<InitialCondition> =
        rdr.deserialize().collect::<Result<_,_>>()?;

    Ok(())
}
