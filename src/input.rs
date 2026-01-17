use crate::simulation::Body;
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct InitialCondition {
    pub name: String,
    pub mass: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub vel_x: f64,
    pub vel_y: f64,
}

/// Loads the bodies given a path to a CSV file of initial conditions
pub fn load_bodies(path: &Path) -> Result<Vec<Body>, Box<dyn Error>> {
    let mut csv_reader = Reader::from_path(path)?;
    csv_reader
        .deserialize::<InitialCondition>()
        .map(|ic| ic.map(Body::from))
        .collect::<Result<Vec<Body>, csv::Error>>()
        .map_err(|e| e.into())
}
