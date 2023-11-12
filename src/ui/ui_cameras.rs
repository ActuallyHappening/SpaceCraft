//! Manages spawning and responding to screen re-size events for
//! all ui cameras

use bevy::core_pipeline::tonemapping::Tonemapping;

use crate::prelude::*;

/// Plugin
pub struct UiCamerasPlugin;

impl Plugin for UiCamerasPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Startup, Self::spawn_ui_cameras)
			.add_systems(Update, update_cameras);
	}
}

#[derive(Bundle)]
struct UiCameraBundle {
	inner_camera: Camera2dBundle,
	render_layer: RenderLayers,
	variant: UiCamera,
	name: Name,

	vis: VisibilityBundle,
}

impl UiCamerasPlugin {
	fn spawn_ui_camera(cam: UiCamera, commands: &mut Commands) {
		commands.spawn(UiCameraBundle {
			inner_camera: Camera2dBundle {
				camera: Camera {
					order: GlobalCameraOrders::Ui(cam.variant).into(),
					hdr: true,
					..default()
				},
				camera_2d: Camera2d {
					clear_color: ClearColorConfig::None,
				},
				tonemapping: Tonemapping::None,
				..default()
			},
			render_layer: GlobalRenderLayers::Ui(cam.variant).into(),
			variant: cam,
			name: Name::new(format!("UI Camera: {:?}", cam.variant)),
			vis: Default::default(),
		});
	}

	fn spawn_ui_cameras(mut commands: Commands) {
		for variant in UiCameras::iter() {
			Self::spawn_ui_camera(UiCamera { variant }, &mut commands)
		}
	}
}

/// Component of UiCameras that are 2D
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct UiCamera {
	pub variant: UiCameras,
}

impl UiCamera {
	/// Returns the translation each camera would like to have, given the
	/// current screen size.
	/// Useful for positioning the camera relative to the screen, e.g. center or
	/// top left
	fn get_camera_transform(&self, half_width: f32, half_height: f32) -> Vec2 {
		let (wf, hf) = match self.variant {
			UiCameras::TopLeft => (1, -1), // works
			UiCameras::TopMiddle => (0, -1), // works
			UiCameras::TopRight => (-1, -1), // works
			UiCameras::MiddleLeft => (1, 0), // works
			UiCameras::Center => (0, 0),   // works
			UiCameras::MiddleRight => (-1, 0), // works
			UiCameras::BottomLeft => (1, 1), // works
			UiCameras::BottomMiddle => (0, 1), //
			UiCameras::BottomRight => (-1, 1), // works
		};
		Vec2::new(wf as f32 * half_width, hf as f32 * half_height)
	}
}

/// Handles screen resizing events
fn update_cameras(
	windows: Query<&Window>,
	mut resize_events: EventReader<bevy::window::WindowResized>,
	mut cam: Query<(&mut Transform, &UiCamera)>,
) {
	for ev in resize_events.read() {
		let window = windows.get(ev.window).unwrap();
		for (mut cam, variant) in cam.iter_mut() {
			let width = window.resolution.width();
			let height = window.resolution.height();

			let cam_translation = variant.get_camera_transform(width / 2., height / 2.);
			cam.translation = Vec3::new(cam_translation.x, cam_translation.y, 0.);
		}
	}
}

#[derive(SystemParam)]
pub struct CorrectCamera<'w, 's> {
	cam: Query<'w, 's, &'static UiCamera>,
}

impl CorrectCamera<'_, '_> {
	/// Logs error to console and checks if camera is valid
	pub fn confirm(&self, cam_entity: &Entity, expected_type: UiCameras) -> bool {
		match self.cam.get(*cam_entity) {
			Ok(cam) => {
				if cam.variant == expected_type {
					true
				} else {
					// todo: remove when seeing
					warn!("Wrong camera found for bevy_mod_picking hit event");
					false
				}
			}
			Err(_) => {
				error!("Not a camera entity!");
				false
			}
		}
	}
}
