use std::path::Path;

use crate::simulation::Data;

pub fn print_data(data: Data) {
    println!("{}", "=".repeat(80));
    println!("{:^80}", "N-Body Simulation Results");
    println!("{}", "=".repeat(80));

    for snapshot in &data {
        println!("{}", snapshot);
    }

    println!("{}", "=".repeat(80));
    println!("Total snapshots: {}", data.len());
}

pub fn save_to_csv(path: &Path, data: Data) -> Result<(), std::io::Error> {
    // to be implemented
    Ok(())
}
