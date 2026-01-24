use std::{fmt, path::Path};

use crate::simulation::Body;
use csv::Writer;
use serde::Serialize;

#[derive(Serialize)]
pub struct BodySnapshot {
    time: f64,
    id: usize,
    x: f64,
    y: f64,
}

impl BodySnapshot {
    pub fn create(body: &Body, time: f64) -> Self {
        BodySnapshot {
            time,
            id: body.id,
            x: body.position[0],
            y: body.position[1],
        }
    }
}

impl fmt::Display for BodySnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Time: {:>10.4}s | Body {:>3} | Position: ({:>12.6}, {:>12.6})",
            self.time, self.id, self.x, self.y
        )
    }
}

pub fn print_data(data: Vec<BodySnapshot>) {
    println!("{}", "=".repeat(80));
    println!("{:^80}", "N-Body Simulation Results");
    println!("{}", "=".repeat(80));

    for snapshot in &data {
        println!("{}", snapshot);
    }

    println!("{}", "=".repeat(80));
    println!("Total snapshots: {}", data.len());
}

pub fn save_to_csv(path: &Path, data: Vec<BodySnapshot>) -> Result<(), std::io::Error> {
    let mut wtr = Writer::from_path(path)?;

    for snapshot in &data {
        wtr.serialize(snapshot)?;
    }

    wtr.flush()?;

    Ok(())
}
