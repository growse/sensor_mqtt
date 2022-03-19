use std::collections::VecDeque;
use std::time::Duration;

use rand_distr::num_traits::abs;

use crate::samplers::sampler::Sampler;
use crate::sensors::sensor::Measurement;

pub struct FancySampler {
    possible_intervals: Vec<Duration>,
    dampening: u16, // Don't jump to a larger interval if we've changed interval in the last $dampening measurements
    previous_measurements: Vec<Measurement>,
    current_interval_index: usize,
}

impl FancySampler {
    fn new() -> Self {
        FancySampler {
            possible_intervals: vec![60, 30, 20, 15, 10, 5, 3, 1]
                .iter()
                .map(|i| Duration::from_secs(*i))
                .collect(),
            current_interval_index: 0,
            dampening: 0,
            previous_measurements: vec![],
        }
    }
}

impl Sampler for FancySampler {
    fn next(&mut self, measurement: Measurement) -> Duration {
        let previous_average = measurements_mean_delta(self.previous_measurements.as_ref());
        self.possible_intervals[self.current_interval_index]
    }

    fn max(&self) -> Duration {
        match self.possible_intervals.iter().max() {
            None => Duration::from_secs(1),
            Some(value) => value.clone(),
        }
    }
}

fn measurements_mean_delta(m: &Vec<Measurement>) -> Option<f32> {
    let mut offset = m.clone();
    offset.pop();
    let deltas: Vec<Measurement> = m
        .iter()
        .zip(offset.iter())
        .map(|p| abs(p.0 - p.1))
        .collect();
    println!("TOOOT {:?}", offset);
    match deltas.len() {
        0 => None,
        l => Some(deltas.iter().sum::<f32>() / l as f32),
    }
}

#[cfg(test)]
mod tests {
    use crate::samplers::fancy::{measurements_mean_delta, FancySampler};
    use crate::samplers::sampler::Sampler;
    use crate::sensors::pretend::PretendSensor;
    use crate::sensors::sensor::{Measurement, Sensor};

    #[test]
    fn can_take_mean_of_list_of_measurements() {
        let input = vec![1 as f32, 3 as f32, 5 as f32];
        let actual = measurements_mean_delta(input.as_ref());
        assert_eq!(Some(2 as f32), actual);
    }

    #[test]
    fn fancy_sampler_returns_first_interval_after_one_measurement() {
        let mut sampler = FancySampler::new();
        let wait_for = sampler.next(5 as f32);
        assert_eq!(60, wait_for.as_secs())
    }

    #[test]
    fn fancy_sampler_returns_first_interval_after_lots_of_measurements() {
        let mut sensor = PretendSensor::new();
        let mut sampler = FancySampler::new();
        for _ in 0..100 {
            let m = sensor.measure().unwrap();
            info!("Measurements {m:?}");
            let wait_for = sampler.next(m[0]);
            info!("Will wait for {wait_for:?}");
        }
        let measurement = sensor.measure().unwrap();
        let wait_for = sampler.next(measurement[0]);
        assert_eq!(60, wait_for.as_secs())
    }

    #[test]
    fn fancy_sampler_increases_frequency_when_new_measurement_more_than_average_delta_arrives() {
        let mut sampler = FancySampler::new();
        for _ in 0..5 {
            sampler.next(5 as f32);
        }
        assert!(false);
    }
}
