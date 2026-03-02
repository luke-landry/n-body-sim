use cudarc::driver::CudaSlice;
use std::error::Error;

use crate::{body::Bodies, gpu::CudaManager};

/// SoA representation of bodies on the GPU
pub struct DeviceBodies {
    pub masses: CudaSlice<f64>,
    pub pos_x: CudaSlice<f64>,
    pub pos_y: CudaSlice<f64>,
    pub pos_z: CudaSlice<f64>,
    pub vel_x: CudaSlice<f64>,
    pub vel_y: CudaSlice<f64>,
    pub vel_z: CudaSlice<f64>,

    // stores intermediate accelerations during gravity calculations
    pub acc_x: CudaSlice<f64>,
    pub acc_y: CudaSlice<f64>,
    pub acc_z: CudaSlice<f64>,
}

impl DeviceBodies {
    pub fn new(gpu: &CudaManager, bodies: &Bodies) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            masses: gpu.stream.clone_htod(&bodies.masses)?,
            pos_x: gpu.stream.clone_htod(&bodies.pos_x)?,
            pos_y: gpu.stream.clone_htod(&bodies.pos_y)?,
            pos_z: gpu.stream.clone_htod(&bodies.pos_z)?,
            vel_x: gpu.stream.clone_htod(&bodies.vel_x)?,
            vel_y: gpu.stream.clone_htod(&bodies.vel_y)?,
            vel_z: gpu.stream.clone_htod(&bodies.vel_z)?,
            acc_x: gpu.stream.alloc_zeros(bodies.len())?,
            acc_y: gpu.stream.alloc_zeros(bodies.len())?,
            acc_z: gpu.stream.alloc_zeros(bodies.len())?,
        })
    }

    /// Download body data from the GPU back to the CPU.
    /// Synchronization is the responsibility of the caller.
    pub fn download(&self, gpu: &CudaManager) -> Result<Bodies, Box<dyn Error>> {
        Ok(Bodies {
            masses: gpu.stream.clone_dtoh(&self.masses)?,
            pos_x: gpu.stream.clone_dtoh(&self.pos_x)?,
            pos_y: gpu.stream.clone_dtoh(&self.pos_y)?,
            pos_z: gpu.stream.clone_dtoh(&self.pos_z)?,
            vel_x: gpu.stream.clone_dtoh(&self.vel_x)?,
            vel_y: gpu.stream.clone_dtoh(&self.vel_y)?,
            vel_z: gpu.stream.clone_dtoh(&self.vel_z)?,
        })
    }

    pub fn len(&self) -> usize {
        self.masses.len()
    }
}
