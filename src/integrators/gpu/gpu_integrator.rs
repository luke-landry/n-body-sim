use std::error::Error;

use crate::{
    gpu::{cuda_manager::CudaManager, device_bodies::DeviceBodies},
    gravity::GpuGravity,
};

pub trait GpuIntegrator: Send {
    fn step(
        &self,
        gpu: &CudaManager,
        bodies: &mut DeviceBodies,
        gravity: &dyn GpuGravity, // Passed in, not owned
    ) -> Result<(), Box<dyn Error>>;
}
