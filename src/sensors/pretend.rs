use std::borrow::BorrowMut;

use anyhow::Result;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

use crate::sensors::sensor::{Measurement, Sensor};

pub struct PretendSensor {
    distribution: Normal<f32>,
    rng: StdRng,
}

impl PretendSensor {
    pub fn new() -> Self {
        PretendSensor {
            distribution: Normal::new(2.0, 3.0).unwrap(),
            rng: rand::rngs::StdRng::seed_from_u64(1),
        }
    }
}

impl Sensor for PretendSensor {
    fn measure(&mut self) -> Result<Vec<Measurement>> {
        let normal_val = self.distribution.sample(self.rng.borrow_mut());
        Ok(vec![normal_val])
    }
}
