use crate::gravity::Gravity;
use rayon::prelude::*;
// This module uses the similarly named barnes_hut crate by
// David-OConnor for the Barnes-Hut algorithm implementation
use barnes_hut::{self, BhConfig, Tree};

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
    fn calculate_accelerations(
        &self,
        masses: &[f64],
        rx: &[f64],
        ry: &[f64],
        rz: &[f64],
        ax: &mut [f64],
        ay: &mut [f64],
        az: &mut [f64],
    ) {
        // to be implemented
    }
}
