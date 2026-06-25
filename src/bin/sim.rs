use std::time::Duration;

use mujoco_rs::prelude::*;
use mujoco_rs::viewer::MjViewer;

use reachy::actuator_io::sim::InteractiveSimulation;
use reachy::actuator_io::NUM_ACTUATORS;
use reachy::control::run_test;

const MODEL_PATH: &str = "vendor/reachy_mini/descriptions/reachy_mini/mjcf/scene.xml";

fn main() {
    let model = Box::new(MjModel::from_xml(MODEL_PATH).expect("could not load model"));
    let sim_timestep = Duration::from_secs_f64(model.opt().timestep);
    println!("nq={}  nv={}  nu={}", model.nq(), model.nv(), model.nu());
    assert_eq!(model.nu() as usize, NUM_ACTUATORS);

    let data = MjData::new(model);
    let mut viewer = MjViewer::builder()
        .max_user_geoms(0)
        .vsync(true)
        .build_passive(data.model())
        .expect("could not launch viewer");

    let mut sim = InteractiveSimulation {
        data: data,
        viewer_state: Some(viewer.state().clone()),
    };

    let control_loop = std::thread::spawn(move || {
        run_test(&mut sim, sim_timestep);
    });

    while viewer.running() {
        viewer.render().expect("Error rendering");
    }

    control_loop
        .join()
        .expect("Could not join thread. Closing.")
}
