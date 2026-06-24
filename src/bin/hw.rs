use std::time::Duration;

use reachy::actuator_io::{ActuatorIO, hardware::HardwareIO};
use reachy::control::targets_at;

fn main() {
    let mut io = HardwareIO::new().expect("connect to actuators");

    let dt = 0.01; // 100 Hz; see skills/control-loops.md
    let mut t = 0.0_f64;
    loop {
        io.set_angles(&targets_at(t)).expect("write angles");
        std::thread::sleep(Duration::from_secs_f64(dt));
        t += dt;
    }
}
