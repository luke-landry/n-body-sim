use std::{fmt, path::Path};

use crate::simulation::Bodies;
use csv::Writer;
use serde::Serialize;

#[derive(Serialize)]
pub struct BodySnapshot {
    time: f64,
    id: usize,
    x: f64,
    y: f64,
    z: f64,
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

/// SoA representation of BodySnapshot
pub struct BodiesSnapshot {
    pub time: f64,
    pub ids: Vec<usize>,
    pub pos_x: Vec<f64>,
    pub pos_y: Vec<f64>,
    pub pos_z: Vec<f64>,
}

impl BodiesSnapshot {
    pub fn from_state(bodies: &Bodies, time: f64) -> Self {
        BodiesSnapshot {
            time,
            ids: (0..bodies.len()).collect(),
            pos_x: bodies.pos_x.clone(),
            pos_y: bodies.pos_y.clone(),
            pos_z: bodies.pos_z.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }
}

pub fn flatten_bodies_snapshots(snapshots: &[BodiesSnapshot]) -> Vec<BodySnapshot> {
    let mut flat = Vec::new();
    for snapshot in snapshots {
        let n = snapshot.len();
        for i in 0..n {
            flat.push(BodySnapshot {
                time: snapshot.time,
                id: snapshot.ids[i],
                x: snapshot.pos_x[i],
                y: snapshot.pos_y[i],
                z: snapshot.pos_z[i],
            });
        }
    }
    flat
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
