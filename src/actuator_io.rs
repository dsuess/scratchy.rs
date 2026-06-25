pub const NUM_ACTUATORS: usize = 9;

#[derive(Debug)]
pub enum ActuatorIoError {
    /// Underlying transport (serial bus, USB) failed.
    Io(std::io::Error),
    /// A requested angle was outside the actuator's safe range.
    OutOfRange,
}

impl From<std::io::Error> for ActuatorIoError {
    fn from(err: std::io::Error) -> Self {
        ActuatorIoError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, ActuatorIoError>;
pub type ActuatorAngles = [f64; NUM_ACTUATORS];

pub trait ActuatorIO {
    fn set_angles(&mut self, angles: &ActuatorAngles) -> Result<()>;
    fn angles(&mut self) -> Result<ActuatorAngles>;
    fn step(&mut self) -> () {}
    fn alive(&mut self) -> bool {
        true
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    /// In-memory backend that just echoes back the angles last written.
    #[derive(Debug, Default)]
    pub struct TestIO {
        angles: ActuatorAngles,
    }

    impl ActuatorIO for TestIO {
        fn set_angles(&mut self, angles: &ActuatorAngles) -> Result<()> {
            self.angles = *angles;
            Ok(())
        }

        fn angles(&mut self) -> Result<ActuatorAngles> {
            Ok(self.angles)
        }
    }

    #[test]
    fn round_trips_angles() {
        let mut io = TestIO::default();
        let target = [1.0; NUM_ACTUATORS];
        io.set_angles(&target).unwrap();
        assert_eq!(io.angles().unwrap(), target);
    }
}

#[cfg(feature = "sim")]
pub mod sim {
    use super::*;
    use mujoco_rs::viewer::ViewerSharedState;
    use mujoco_rs::wrappers::{MjData, MjModel};

    use std::ops::Deref;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    pub struct InteractiveSimulation<M: Deref<Target = MjModel>> {
        pub data: MjData<M>,
        pub viewer_state: Option<Arc<Mutex<ViewerSharedState>>>,
    }

    impl<M: Deref<Target = MjModel>> ActuatorIO for InteractiveSimulation<M> {
        fn set_angles(&mut self, angles: &ActuatorAngles) -> Result<()> {
            let ctrl = self.data.ctrl_mut();
            ctrl.copy_from_slice(angles);
            Ok(())
        }

        fn angles(&mut self) -> Result<ActuatorAngles> {
            // TODO Correct error handling!
            let result = self.data.ctrl().try_into().expect("Could not read");
            Ok(result)
        }

        fn step(&mut self) -> () {
            self.data.step();
            if let Some(viewer_state) = &mut self.viewer_state {
                let mut state = viewer_state.lock().unwrap();
                state.sync_data(&mut self.data);
            }
        }

        fn alive(&mut self) -> bool {
            if let Some(viewer_state) = &mut self.viewer_state {
                let state = viewer_state.lock().unwrap();
                return state.running();
            }
            true
        }
    }
}

#[cfg(feature = "hardware")]
pub mod hardware {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use ctrlc;
    use rustypot::servo::dynamixel::xl330::Xl330Controller;
    use serialport;

    use super::*;

    const SERIAL_PORT_TTY_DEVICE: &str = "/dev/ttyAMA3";
    const SERVO_IDS: [u8; NUM_ACTUATORS] = [10, 11, 12, 13, 14, 15, 16, 17, 18];

    pub struct HardwareIO {
        controller: Xl330Controller,
        shutdown: Arc<AtomicBool>,
    }

    impl HardwareIO {
        pub fn new() -> Result<Self> {
            let comms_port = serialport::new(SERIAL_PORT_TTY_DEVICE, 1_000_000)
                .timeout(Duration::from_millis(100))
                .open()
                .expect(&format!("Failed opening device {}", SERIAL_PORT_TTY_DEVICE));
            let controller = Xl330Controller::new()
                .with_protocol_v2()
                .with_serial_port(comms_port);
            let mut result = HardwareIO {
                controller,
                shutdown: Arc::new(AtomicBool::new(false)),
            };

            {
                let handler_flag = result.shutdown.clone();
                ctrlc::set_handler(move || {
                    handler_flag.store(true, Ordering::Relaxed);
                })
                .expect("Failed to install Ctrl-C handler");
            }

            // Turn off all torques by default
            result
                .controller
                .sync_write_torque_enable(&SERVO_IDS, &[false; NUM_ACTUATORS])
                .unwrap();

            Ok(result)
        }

        pub fn enable_servos(mut self, indices: &[usize]) -> Result<Self> {
            let ids: Vec<u8> = indices
                .iter()
                .filter_map(|&i| SERVO_IDS.get(i).copied())
                .collect();
            let vals = vec![true; ids.len()];
            self.controller
                .sync_write_torque_enable(&ids, &vals)
                .unwrap();
            Ok(self)
        }
    }

    impl ActuatorIO for HardwareIO {
        fn set_angles(&mut self, angles: &ActuatorAngles) -> Result<()> {
            self.controller
                .sync_write_goal_position(&SERVO_IDS, angles)
                .expect("Failed writing");
            Ok(())
        }

        fn angles(&mut self) -> Result<ActuatorAngles> {
            let result: ActuatorAngles = self
                .controller
                .sync_read_present_position(&SERVO_IDS)
                .expect("Cannot read positions")
                .try_into()
                .expect("Cannot convert");
            Ok(result)
        }

        fn alive(&mut self) -> bool {
            !self.shutdown.load(Ordering::Relaxed)
        }
    }
}
