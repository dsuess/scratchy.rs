//! Backend-agnostic robot behaviour shared by every binary.
//!
//! This module knows nothing about MuJoCo or real hardware — it only maps time
//! to desired actuator angles, so the sim and hardware binaries move identically.

use crate::actuator_io::{ActuatorAngles, NUM_ACTUATORS};

pub const YAW_BODY: usize = 0;
pub const RIGHT_ANTENNA: usize = 7;
pub const LEFT_ANTENNA: usize = 8;

/// Desired actuator angles at time `t` (seconds).
pub fn targets_at(t: f64) -> ActuatorAngles {
    let mut a = [0.0; NUM_ACTUATORS];
    a[YAW_BODY] = 0.8 * (2.0 * t).sin();
    a[RIGHT_ANTENNA] = 0.5 * (5.0 * t).sin();
    a[LEFT_ANTENNA] = 0.5 * (5.0 * t + std::f64::consts::PI).sin();
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_zero_only_antenna_phase_offset_moves() {
        let a = targets_at(0.0);
        assert_eq!(a[YAW_BODY], 0.0);
        assert_eq!(a[RIGHT_ANTENNA], 0.0);
        // left antenna is phase-shifted by PI, so sin(PI) ~ 0 too
        assert!(a[LEFT_ANTENNA].abs() < 1e-9);
        // all other actuators are idle
        assert!(a.iter().all(|v| v.abs() < 1e-9));
    }

    #[test]
    fn left_and_right_antennas_are_antiphase() {
        // half a period of the 5 rad/s antenna wave
        let t = std::f64::consts::PI / 5.0;
        let a = targets_at(t);
        assert!((a[LEFT_ANTENNA] + a[RIGHT_ANTENNA]).abs() < 1e-9);
    }
}
