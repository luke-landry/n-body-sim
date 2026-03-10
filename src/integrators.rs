// CPU-based integrator implementations
pub mod cpu {
    // top level integrator module for CPU-based integrators
    pub mod integrator;

    pub mod euler;
    pub mod runge_kutta;
    pub mod velocity_verlet;
}

// GPU-based integrator implementations
pub mod gpu {
    // top level integrator module for GPU-based integrators
    pub mod gpu_integrator;

    pub mod euler;
    // pub mod velocity_verlet; -- not implemented yet
    // pub mod runge_kutta; -- not implemented yet
}
