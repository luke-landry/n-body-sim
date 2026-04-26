use std::error::Error;

use crate::gpu::{cuda_manager::CudaManager, device_bodies::DeviceBodies};

pub mod cpu {
    pub mod barnes_hut;
    pub mod newton;
    pub mod newton_parallel;
}

pub mod gpu {
    pub mod newton_parallel;
}

pub trait Gravity: Send {
    fn calculate_accelerations(
        &mut self,
        masses: &[f64],
        rx: &[f64],
        ry: &[f64],
        rz: &[f64],
        ax: &mut [f64],
        ay: &mut [f64],
        az: &mut [f64],
    );
}

pub trait GpuGravity: Send {
    fn calculate_accelerations(
        &self,
        gpu: &CudaManager,
        bodies: &DeviceBodies,
    ) -> Result<(), Box<dyn Error>>;
}
