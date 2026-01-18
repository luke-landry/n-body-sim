use crate::{gravity::Gravity, simulation::Body};

pub struct BarnesHutGravity {
    g_constant: f64,
    softening_factor: f64,
    theta: f64,
}

impl BarnesHutGravity {
    pub fn new(g_constant: f64, softening_factor: f64, theta: f64) -> Self {
        BarnesHutGravity {
            g_constant: g_constant,
            softening_factor: softening_factor,
            theta: theta,
        }
    }
}

impl Gravity for BarnesHutGravity {
    fn calculate_accelerations(&self, bodies: &[Body]) -> Vec<[f64; 2]> {
        // to be implemented
        vec![]
    }
}
