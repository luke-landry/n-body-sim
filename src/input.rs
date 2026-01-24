use crate::simulation::Body;
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct InitialCondition {
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
        .enumerate()
        .map(|(id, result)| {
            result.map(|ic| {
                println!(
                    "read body #{:<4} | Mass: {:>7} | Pos: ({:>7}, {:>7}) | Vel: ({:>7}, {:>7})",
                    id, ic.mass, ic.pos_x, ic.pos_y, ic.vel_x, ic.vel_y
                );
                Body::new(id, ic.mass, [ic.pos_x, ic.pos_y], [ic.vel_x, ic.vel_y])
            })
        })
        .collect::<Result<Vec<Body>, _>>()
        .map_err(|e| e.into())
}
