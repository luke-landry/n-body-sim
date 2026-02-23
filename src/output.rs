use csv;
use serde::Serialize;
use std::path::PathBuf;

pub struct SimulationData {
    times: Vec<f64>,
    ids: Vec<u64>,
    pos_x: Vec<f64>,
    pos_y: Vec<f64>,
    pos_z: Vec<f64>,
}

impl SimulationData {
    // The size of a single simulation data record is 5 64-bit (8-byte)
    // values: time, id, pos_x, pos_y, pos_z, so 5x8 = 40 bytes per record.
    pub const RECORD_SIZE_BYTES: usize = 40;

    pub fn with_capacity(capacity: usize) -> Self {
        SimulationData {
            times: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
            pos_x: Vec::with_capacity(capacity),
            pos_y: Vec::with_capacity(capacity),
            pos_z: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.times.len()
    }

    pub fn extend_from_step(&mut self, time: f64, pos_x: &[f64], pos_y: &[f64], pos_z: &[f64]) {
        self.times.extend(std::iter::repeat(time).take(pos_x.len()));
        self.ids.extend(0..pos_x.len() as u64);
        self.pos_x.extend_from_slice(pos_x);
        self.pos_y.extend_from_slice(pos_y);
        self.pos_z.extend_from_slice(pos_z);
    }
}

#[derive(Serialize)]
struct SimulationDataRecord {
    time: f64,
    id: u64,
    x: f64,
    y: f64,
    z: f64,
}

pub struct SimulationDataWriter {
    path: PathBuf,
    rx: std::sync::mpsc::Receiver<SimulationData>,
}

impl SimulationDataWriter {
    pub fn new(path: PathBuf, rx: std::sync::mpsc::Receiver<SimulationData>) -> Self {
        SimulationDataWriter { path, rx }
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        let extension = self
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_ascii_lowercase());

        let format = match extension {
            Some(ext) => ext,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "File must have a valid extension",
                ));
            }
        };

        match format.as_str() {
            "csv" => self.write_csv(),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported file format",
            )),
        }
    }

    fn write_csv(&self) -> Result<(), std::io::Error> {
        let mut wtr = csv::Writer::from_path(&self.path)?;
        for batch in self.rx.iter() {
            for i in 0..batch.len() {
                let record = SimulationDataRecord {
                    time: batch.times[i],
                    id: batch.ids[i],
                    x: batch.pos_x[i],
                    y: batch.pos_y[i],
                    z: batch.pos_z[i],
                };
                wtr.serialize(record)?;
            }
        }
        wtr.flush()?;
        Ok(())
    }
}
