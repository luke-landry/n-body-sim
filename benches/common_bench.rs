// Common utilities for benchmarks

/// Generates a non-trivial deterministic distribution of bodies to
/// for more realistic and consistent performance during benchmarks
pub fn generate_distributed_bodies_positions(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut masses = Vec::with_capacity(n);
    let mut rx = Vec::with_capacity(n);
    let mut ry = Vec::with_capacity(n);
    let mut rz = Vec::with_capacity(n);

    // 20 x 20 x 20 bounding box
    let radius = 10.0;
    let height = 20.0;

    let mass_base = 1.0;

    for i in 0..n {
        // Spiral distribution in x/y, layered in z
        let angle = i as f64 * 0.61803398875; // golden angle for spacing
        let r = radius * (i as f64) / (n as f64); // radius * i/n where i/n goes from 0 to 1
        rx.push(r * angle.cos());
        ry.push(r * angle.sin());

        rz.push(height * ((i as f64) / (n as f64) - 0.5)); // z from -height/2 to +height/2

        // Masses vary slightly but repeatably
        masses.push(mass_base + (i % 10) as f64 * 0.1);
    }

    (masses, rx, ry, rz)
}
