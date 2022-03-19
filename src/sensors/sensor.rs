use anyhow::Result;

pub trait Sensor {
    fn measure(&mut self) -> Result<Vec<Measurement>>;
}

pub type Measurement = f32;
