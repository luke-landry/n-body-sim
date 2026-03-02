#[derive(Clone)]
pub struct Body {
    pub mass: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub vel_z: f64,
}

impl Body {
    pub fn new(
        mass: f64,
        pos_x: f64,
        pos_y: f64,
        pos_z: f64,
        vel_x: f64,
        vel_y: f64,
        vel_z: f64,
    ) -> Self {
        Body {
            mass,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
        }
    }
}

impl From<Bodies> for Vec<Body> {
    fn from(bodies: Bodies) -> Self {
        let mut result = Vec::with_capacity(bodies.len());
        for i in 0..bodies.len() {
            result.push(Body {
                mass: bodies.masses[i],
                pos_x: bodies.pos_x[i],
                pos_y: bodies.pos_y[i],
                pos_z: bodies.pos_z[i],
                vel_x: bodies.vel_x[i],
                vel_y: bodies.vel_y[i],
                vel_z: bodies.vel_z[i],
            });
        }
        result
    }
}

/// SoA representation of a Vec<Body>
pub struct Bodies {
    pub masses: Vec<f64>,
    pub pos_x: Vec<f64>,
    pub pos_y: Vec<f64>,
    pub pos_z: Vec<f64>,
    pub vel_x: Vec<f64>,
    pub vel_y: Vec<f64>,
    pub vel_z: Vec<f64>,
}

impl Bodies {
    pub fn new(
        masses: Vec<f64>,
        pos_x: Vec<f64>,
        pos_y: Vec<f64>,
        pos_z: Vec<f64>,
        vel_x: Vec<f64>,
        vel_y: Vec<f64>,
        vel_z: Vec<f64>,
    ) -> Self {
        let len = masses.len();
        assert!(
            pos_x.len() == len
                && pos_y.len() == len
                && pos_z.len() == len
                && vel_x.len() == len
                && vel_y.len() == len
                && vel_z.len() == len,
            "All input vectors must have the same length"
        );

        Bodies {
            masses,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
        }
    }

    pub fn len(&self) -> usize {
        self.masses.len()
    }

    /// Return tuple of references to the mass, 3D position, and 3D velocity components
    pub fn as_slices(&mut self) -> (&[f64], &[f64], &[f64], &[f64], &[f64], &[f64], &[f64]) {
        (
            &self.masses,
            &self.pos_x,
            &self.pos_y,
            &self.pos_z,
            &self.vel_x,
            &self.vel_y,
            &self.vel_z,
        )
    }
    /// Return tuple of mutable references to the mass, 3D position, and 3D velocity components
    pub fn as_slices_mut(
        &mut self,
    ) -> (
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
    ) {
        (
            &mut self.masses,
            &mut self.pos_x,
            &mut self.pos_y,
            &mut self.pos_z,
            &mut self.vel_x,
            &mut self.vel_y,
            &mut self.vel_z,
        )
    }
}

impl From<&[Body]> for Bodies {
    fn from(bodies: &[Body]) -> Self {
        let mut masses = Vec::with_capacity(bodies.len());
        let mut pos_x = Vec::with_capacity(bodies.len());
        let mut pos_y = Vec::with_capacity(bodies.len());
        let mut pos_z = Vec::with_capacity(bodies.len());
        let mut vel_x = Vec::with_capacity(bodies.len());
        let mut vel_y = Vec::with_capacity(bodies.len());
        let mut vel_z = Vec::with_capacity(bodies.len());

        for body in bodies {
            masses.push(body.mass);
            pos_x.push(body.pos_x);
            pos_y.push(body.pos_y);
            pos_z.push(body.pos_z);
            vel_x.push(body.vel_x);
            vel_y.push(body.vel_y);
            vel_z.push(body.vel_z);
        }

        Bodies {
            masses,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
        }
    }
}
