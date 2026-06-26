use std::time::Duration;

use scratchy::actuator_io::{ActuatorIO, hardware::HardwareIO};
use scratchy::control::{LEFT_ANTENNA, RIGHT_ANTENNA, YAW_BODY, run_test};

fn main() {
    let mut io = HardwareIO::new()
        .expect("connect to actuators")
        .enable_servos(&[YAW_BODY, LEFT_ANTENNA, RIGHT_ANTENNA])
        .expect("Failed setting servos");
    run_test(&mut io, Duration::from_millis(10));
}
