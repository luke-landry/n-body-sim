/// Value of the 2022 CODATA Newtonian constant of gravitation as defined in
/// https://physics.nist.gov/cgi-bin/cuu/Value?bg
// pub const G: f64 = 6.67430e-11; // m^3 kg^-1 s^-2

/// Default G value (1 for simplicity)
pub const DEFAULT_G: f64 = 1.0;

/// Default simulation timestep (s): 3600 seconds = 1 hour
pub const DEFAULT_TIMESTEP: f64 = 3600.0;

/// Default simulation number of steps: 8760 hours = 1 year
/// assuming default timestep of 1 hour is used
pub const DEFAULT_NUM_STEPS: usize = 8760;

/// Default softening factor (m): 100 km
pub const DEFAULT_SOFTENING_FACTOR: f64 = 1e5;

/// Default gravity force calculation method
pub const DEFAULT_GRAVITY: &str = "newton";

/// Default integrator for computing next-step state
pub const DEFAULT_INTEGRATOR: &str = "euler";

/// Default theta value for barnes hut gravity calculations
pub const DEFAULT_THETA: f64 = 0.5;
