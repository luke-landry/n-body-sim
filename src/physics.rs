use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct InitialCondition {
    pos_x: u32,
    pos_y: u32,
    vel_x: u32,
    vel_y: u32,
}
