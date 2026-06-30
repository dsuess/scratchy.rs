use crate::actuator_io::{self, ActuatorAngles, ActuatorIO, NUM_ACTUATORS};

use spin_sleep::sleep_until;
use std::time::{Duration, Instant};

pub const YAW_BODY: usize = 0;
pub const RIGHT_ANTENNA: usize = 7;
pub const LEFT_ANTENNA: usize = 8;

pub fn run_test<T>(model: &mut T, timestep: Duration)
where
    T: ActuatorIO,
{
    let mut angles: ActuatorAngles = model.angles().expect("Could not fetch init angles");
    let mut t: f64 = 0.0;

    model.step();
    while model.alive() {
        let loop_start = Instant::now();
        angles[YAW_BODY] = 0.8 * (2.0 * t).sin();
        angles[RIGHT_ANTENNA] = 0.5 * (5.0 * t).sin();
        angles[LEFT_ANTENNA] = 0.5 * (5.0 * t + std::f64::consts::PI).sin();
        model.set_angles(&angles).expect("Failed to set angles");
        model.step();
        t += timestep.as_secs_f64();
        let next_step = loop_start + timestep;

        let elapsed = loop_start.elapsed();
        if elapsed <= timestep {
            sleep_until(next_step);
        } else {
            println!(
                "Control loop cannot keep real-time at t={t:.2}s: \
                 iteration took {:.3}ms but the timestep budget is {:.3}ms (over by {:.3}ms)",
                elapsed.as_secs_f64() * 1e3,
                timestep.as_secs_f64() * 1e3,
                (elapsed - timestep).as_secs_f64() * 1e3,
            )
        }
    }
}
