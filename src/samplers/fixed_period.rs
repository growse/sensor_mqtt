use std::time::Duration;

use crate::samplers::sampler::Sampler;
use crate::sensors::sensor::Measurement;

pub struct FixedPeriodSampler {
    interval: Duration,
}

impl FixedPeriodSampler {
    pub fn new(interval_seconds: Duration) -> Self {
        FixedPeriodSampler {
            interval: interval_seconds,
        }
    }
}

impl Sampler for FixedPeriodSampler {
    fn next(&mut self, _measurements: Measurement) -> Duration {
        debug!("Next measurement is in {}s", self.interval.as_secs());
        self.interval
    }

    fn max(&self) -> Duration {
        self.interval
    }
}
//
// #[cfg(test)]
// mod tests {
//     use crate::samplers::fixed_period::FixedPeriodSampler;
//     use crate::samplers::sampler::Sampler;
//     use crate::sensors::sensor::get_sensor;
//
//     #[test]
//     fn fixed_period_sampler_outputs_same_tick_delay() {
//         let interval = std::time::Duration::from_secs(14);
//         let mut sensor = get_sensor();
//         let measurement = sensor.measure().unwrap();
//         let mut sampler = FixedPeriodSampler::new(interval);
//         assert_eq!(14, sampler.next(measurement).as_secs());
//     }
// }
