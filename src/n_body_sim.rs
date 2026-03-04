use crate::args::Args;
use crate::body::Body;
use crate::gpu::CudaManager;
use crate::gpu::device_bodies::DeviceBodies;
use crate::gpu::simulation::GpuSimulator;
use crate::input;
use crate::output::{SimulationData, SimulationDataWriter};
use crate::simulation::{Parameters, Simulation, Simulator};
use std::error::Error;
use std::sync::mpsc::Sender;

pub struct NBodySim {
    simulation: Box<dyn Simulation>,
    writer: SimulationDataWriter,
}

impl NBodySim {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        let bodies = input::load_bodies(&args.initial_conditions_path)?;
        let parameters = Parameters::new(
            args.time_step,
            args.num_steps,
            args.g_constant,
            args.softening_factor,
            args.theta,
            args.progress,
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let simulation = Self::create_simulation(&args, parameters, bodies, Some(tx));
        let writer = SimulationDataWriter::new(args.output_data_path.clone(), rx);

        Ok(Self { simulation, writer })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let writer = self.writer;
        let mut simulation = self.simulation;

        // run writer and simulator in separate threads
        let writer_handle = std::thread::spawn(move || writer.run());
        let simulation_handle = std::thread::spawn(move || simulation.run());

        simulation_handle
            .join()
            .map_err(|_| "Simulation thread panicked")?;

        // wait for writer to finish after simulation completes
        writer_handle
            .join()
            .map_err(|_| "Writer thread panicked")?
            .map_err(|e| e.into())
    }

    fn create_simulation(
        args: &Args,
        parameters: Parameters,
        bodies: Vec<Body>,
        tx: Option<Sender<SimulationData>>,
    ) -> Box<dyn Simulation> {
        if args.gpu {
            let gpu = CudaManager::new().expect("Failed to initialize GPU for simulation");
            gpu.gpu_init_check()
                .expect("GPU initialization check failed");
            let device_bodies = DeviceBodies::new(&gpu, &bodies.as_slice().into())
                .expect("Failed to create GPU device bodies");
            let gravity = args.gravity.gpu_create(&parameters);
            let integrator = args.integrator.gpu_create(parameters.time_step);
            Box::new(GpuSimulator::new(
                gpu,
                parameters,
                device_bodies,
                gravity,
                integrator,
                tx,
            ))
        } else {
            let gravity = args.gravity.create(&parameters, bodies.len());
            let integrator = args
                .integrator
                .create(gravity, parameters.time_step, bodies.len());
            Box::new(Simulator::new(bodies, parameters, integrator, tx))
        }
    }
}
