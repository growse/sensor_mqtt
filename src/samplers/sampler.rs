use crate::sensors::sensor::Measurement;
use std::time::Duration;

pub trait Sampler {
    fn next(&mut self, _: Measurement) -> Duration;
    fn max(&self) -> Duration;
}
