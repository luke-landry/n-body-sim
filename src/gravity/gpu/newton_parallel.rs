use cudarc::driver::{LaunchConfig, PushKernelArg};

use crate::{
    gpu::{self, cuda_manager::CudaManager},
    gravity::GpuGravity,
};

pub struct GpuNewtonParallelGravity {
    g_constant: f64,
    eps2: f64, // softening factor epsilon squared
}

impl GpuNewtonParallelGravity {
    pub fn new(g_constant: f64, eps2: f64) -> Self {
        Self { g_constant, eps2 }
    }
}

impl GpuGravity for GpuNewtonParallelGravity {
    fn calculate_accelerations(
        &self,
        gpu: &CudaManager,
        bodies: &gpu::device_bodies::DeviceBodies,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let num_bodies = bodies.len() as u32;

        let mut builder = gpu
            .stream
            .launch_builder(&gpu.cuda_fn_newton_compute_accelerations);

        builder
            .arg(&bodies.masses)
            .arg(&bodies.pos_x)
            .arg(&bodies.pos_y)
            .arg(&bodies.pos_z)
            .arg(&bodies.acc_x)
            .arg(&bodies.acc_y)
            .arg(&bodies.acc_z)
            .arg(&num_bodies)
            .arg(&self.g_constant)
            .arg(&self.eps2);

        let config = LaunchConfig::for_num_elems(num_bodies);
        unsafe { builder.launch(config) }?;
        Ok(())
    }
}
