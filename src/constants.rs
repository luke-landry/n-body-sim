/// Value of the 2022 CODATA Newtonian constant of gravitation as defined in
/// https://physics.nist.gov/cgi-bin/cuu/Value?bg
// pub const G: f64 = 6.67430e-11; // m^3 kg^-1 s^-2

pub const DEFAULT_G: f64 = 1.0;

pub const DEFAULT_TIMESTEP: f64 = 1.0;

pub const DEFAULT_NUM_STEPS: usize = 50;

pub const DEFAULT_SOFTENING_FACTOR: f64 = 1e-5;

pub const DEFAULT_GRAVITY: &str = "newton";

pub const DEFAULT_INTEGRATOR: &str = "euler";

/// Default theta value for barnes hut gravity calculations
pub const DEFAULT_THETA: f64 = 0.5;
