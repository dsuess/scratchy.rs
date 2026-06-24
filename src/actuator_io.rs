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
}

#[cfg(test)]
mod testing {
    use super::*;

    /// In-memory backend that just echoes back the angles last written.
    #[derive(Default)]
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
mod sim {
    use super::*;
    use mujoco_rs::wrappers::{MjData, MjModel};
    use std::ops::Deref;

    impl<M: Deref<Target = MjModel>> ActuatorIO for MjData<M> {
        fn set_angles(&mut self, angles: &ActuatorAngles) -> Result<()> {
            let ctrl = self.ctrl_mut();
            ctrl.copy_from_slice(angles);
            Ok(())
        }

        fn angles(&mut self) -> Result<ActuatorAngles> {
            // TODO Correct error handling!
            let result = self.qpos().try_into().unwrap();
            Ok(result)
        }
    }
}

#[cfg(feature = "hardware")]
pub mod hardware {
    use super::*;

    /// Drives the real Reachy Mini servos.
    ///
    /// Stub: the actual servo-bus driver (serial protocol, etc.) is not wired up
    /// yet — see the follow-ups in the plan.
    pub struct HardwareIO {
        _private: (),
    }

    impl HardwareIO {
        /// Open the connection to the actuator bus.
        pub fn new() -> Result<Self> {
            todo!("connect to the Reachy Mini servo bus")
        }
    }

    impl ActuatorIO for HardwareIO {
        fn set_angles(&mut self, _angles: &ActuatorAngles) -> Result<()> {
            todo!("write target angles to the servos")
        }

        fn angles(&mut self) -> Result<ActuatorAngles> {
            todo!("read current angles from the servos")
        }
    }
}
