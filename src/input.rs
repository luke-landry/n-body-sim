use crate::simulation::{Body, InitialCondition};
use csv::Reader;
use std::error::Error;
use std::path::PathBuf;

/// Loads the bodies given a path to a CSV file of initial conditions
pub fn load_bodies(path: &PathBuf) -> Result<Vec<Body>, Box<dyn Error>> {
    let mut csv_reader = Reader::from_path(path)?;

    csv_reader
        .deserialize::<InitialCondition>()
        .map(|ic| ic.map(Body::from))
        .collect::<Result<Vec<Body>, csv::Error>>()
        .map_err(|e| e.into())
}
