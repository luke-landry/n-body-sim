use std::{error::Error, sync::mpsc::Sender};

use crate::{output::SimulationData, simulation::Simulation};

pub struct SimulationRunner {
    num_steps: usize,
    progress: bool,
    tx: Sender<SimulationData>,
}

impl SimulationRunner {
    /// Max size of a batch of simulation data to send in bytes
    const BATCH_SIZE_BYTES: usize = 16000000; // 16 MB

    /// Number of simulation data records that fit in a batch of BATCH_SIZE_BYTES bytes
    const BATCH_SIZE: usize = Self::BATCH_SIZE_BYTES / SimulationData::RECORD_SIZE_BYTES;

    pub fn new(num_steps: usize, progress: bool, tx: Sender<SimulationData>) -> Self {
        Self {
            num_steps,
            progress,
            tx,
        }
    }

    pub fn run(&mut self, mut simulation: Box<dyn Simulation>) -> Result<(), Box<dyn Error>> {
        let mut buffer = SimulationData::with_capacity(Self::BATCH_SIZE);
        let one_percent_steps = (self.num_steps / 100).max(1);

        // record initial state
        buffer.extend_from_snapshot(&simulation.snapshot()?);

        for step in 0..self.num_steps {
            simulation.step()?;

            buffer.extend_from_snapshot(&simulation.snapshot()?);

            if buffer.len() >= Self::BATCH_SIZE {
                // avoid cloning the buffer by replacing it with a new empty
                // one and sending a full batch to the output channel
                let batch =
                    std::mem::replace(&mut buffer, SimulationData::with_capacity(Self::BATCH_SIZE));
                self.tx.send(batch)?;
            }

            if self.progress && step % one_percent_steps == 0 {
                println!("{}", ((step + 1) * 100) / self.num_steps);
            }
        }

        // send any remaining data in the buffer
        if buffer.len() > 0 {
            self.tx.send(buffer)?;
        }

        Ok(())
    }
}
