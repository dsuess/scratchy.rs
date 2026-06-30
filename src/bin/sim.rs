use std::time::Duration;

use mujoco_rs::prelude::*;
use mujoco_rs::viewer::MjViewer;

use scratchy::actuator_io::NUM_ACTUATORS;
use scratchy::actuator_io::sim::InteractiveSimulation;
use scratchy::control::run_test;

const MODEL_PATH: &str = "vendor/reachy_mini/descriptions/reachy_mini/mjcf/scene.xml";

const BODY_PARTS: &[&str] = &[
    "body_down_3dprint_material", // rotating torso panel
    "body_top_3dprint_material",  // top panel
    "head_back_3dprint_material",
    "head_front_3dprint_material",
    "head_mic_3dprint_material",
    "antenna_body_3dprint_material",
    "m12_fisheye_lens_1_8mm_material", // M12 fisheye lens (1.8mm)
    "big_lens_d40_material",           // large lens, Ø40
    "small_lens_d30_material",         // small lens, Ø30
    "lens_cap_d40_3dprint_material",   // lens cap, Ø40
    "lens_cap_d30_3dprint_material",   // lens cap, Ø30
    "arducam_material",                // Arducam camera body
    "pp01102_arducam_carter_material", // Arducam carrier/mount plate
    "glasses_dolder_3dprint_material", // black eye/glasses frame (lens holder)
];
const HIDE_BODY_PARTS: bool = true;

/// Make every body-part material transparent so the surrounding shell doesn't
/// occlude the Stewart platform, without touching the mechanical parts.
fn hide_body_parts(model: &mut MjModel) {
    for name in BODY_PARTS {
        if let Some(info) = &mut model.material(name) {
            info.view_mut(model).rgba[3] = 0.0;
        }
    }
}

fn main() {
    let mut model = Box::new(MjModel::from_xml(MODEL_PATH).expect("could not load model"));
    let sim_timestep = Duration::from_secs_f64(model.opt().timestep);
    println!("nq={}  nv={}  nu={}", model.nq(), model.nv(), model.nu());
    assert_eq!(model.nu() as usize, NUM_ACTUATORS);

    if HIDE_BODY_PARTS {
        hide_body_parts(model.as_mut());
    }

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
