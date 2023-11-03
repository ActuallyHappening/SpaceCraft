use bevy::input::mouse::{MouseMotion, MouseWheel};

use crate::utils::*;

use super::MainCamera;

/// Tags an entity as capable of panning and orbiting.
#[derive(Component, Debug)]
pub struct PanOrbitCamera {
	pub radius: f32,
}

impl Default for PanOrbitCamera {
	fn default() -> Self {
		PanOrbitCamera { radius: 15.0 }
	}
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn orbit_camera(
	windows: Query<&Window>,
	mut ev_motion: EventReader<MouseMotion>,
	input_mouse: Res<Input<MouseButton>>,

	mut query: Query<(&PanOrbitCamera, &mut Rig, &Transform), With<MainCamera>>,
) {
	if let Ok(window) = windows.get_single() {
		// change input mapping for orbit and panning here
		let orbit_button = MouseButton::Right;

		let mut rotation_x = 0.;
		let mut rotation_y = 0.;

		// consume any remaining events, so they don't pile up if we don't need them
		// (and also to avoid Bevy warning us about not checking events every frame update)
		ev_motion.clear();
	} else {
		tracing::warn!("Cannot orbit camera if no windows exist");
	}
}

