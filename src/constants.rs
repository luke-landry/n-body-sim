/// Value of the 2022 CODATA Newtonian constant of gravitation as defined in
/// https://physics.nist.gov/cgi-bin/cuu/Value?bg
// pub const G: f64 = 6.67430e-11; // m^3 kg^-1 s^-2

pub const DEFAULT_INITIAL_CONDITIONS_PATH: &str = "initial_conditions.csv";

pub const DEFAULT_OUTPUT_PATH: &str = "output.txt";

pub const DEFAULT_G: f64 = 1.0;

pub const DEFAULT_TIMESTEP: f64 = 0.01;

pub const DEFAULT_NUM_STEPS: usize = 10000;

pub const DEFAULT_SOFTENING_FACTOR: f64 = 0.005;

pub const DEFAULT_GRAVITY: &str = "newton";

pub const DEFAULT_INTEGRATOR: &str = "euler";

/// Default theta value for barnes hut gravity calculations
pub const DEFAULT_THETA: f64 = 0.5;

pub const DEFAULT_BENCHMARK_GRAVITY_METHODS: &str = "newton,newton-parallel";

pub const DEFAULT_BENCHMARK_INTEGRATOR_METHODS: &str = "euler,velocity-verlet";

/// Default n values for benchmarking when the benchmark flag is set
pub const DEFAULT_BENCHMARK_N_VALUES: &str = "2,3,5,10,15,25,50,75,100,150,200,250,300";

pub const DEFAULT_BENCHMARK_NUM_RUNS: usize = 5;

pub const DEFAULT_BENCHMARK_OUTPUT_PATH: &str = "benchmark_results.csv";
