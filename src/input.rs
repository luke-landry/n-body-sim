use crate::simulation::Body;

use csv::Reader;
use glam::DVec3;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct InitialCondition {
    pub mass: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub vel_z: f64,
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
                    "read body #{:0>4} | Mass: {:>12e} | Pos: ({:>12e}, {:>12e}, {:>12e}) | Vel: ({:>12e}, {:>12e}, {:>12e}) |",
                    id, ic.mass, ic.pos_x, ic.pos_y, ic.pos_z, ic.vel_x, ic.vel_y, ic.vel_z
                );
                Body::new(id, ic.mass, DVec3::new(ic.pos_x, ic.pos_y, ic.pos_z), DVec3::new(ic.vel_x, ic.vel_y, ic.vel_z))
            })
        })
        .collect::<Result<Vec<Body>, _>>()
        .map_err(|e| e.into())
}
