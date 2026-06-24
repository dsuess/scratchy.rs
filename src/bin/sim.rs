use std::time::Duration;

use mujoco_rs::prelude::*;
use mujoco_rs::viewer::MjViewer;

use reachy::actuator_io::ActuatorIO;
use reachy::control::targets_at;

const MODEL_PATH: &str = "vendor/reachy_mini/descriptions/reachy_mini/mjcf/scene.xml";

fn main() {
    let model = MjModel::from_xml(MODEL_PATH).expect("could not load model");
    let mut data = MjData::new(&model);

    println!("nq={}  nv={}  nu={}", model.nq(), model.nv(), model.nu());

    let mut viewer = MjViewer::builder()
        .max_user_geoms(0)
        .build_passive(&model)
        .expect("could not launch viewer");

    let timestep = model.opt().timestep;
    let mut t = 0.0_f64;
    while viewer.running() {
        data.set_angles(&targets_at(t)).unwrap();
        data.step();
        viewer.sync_data(&mut data);
        viewer.render().expect("render failed");
        std::thread::sleep(Duration::from_secs_f64(timestep));
        t += timestep;
    }
}
