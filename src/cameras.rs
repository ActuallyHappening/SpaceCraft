use crate::prelude::*;

#[allow(unused_imports)]
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, spawn_default_cameras);
	}
}

fn spawn_default_cameras(mut commands: Commands) {
	commands
		.spawn((
			Camera3dBundle {
				transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
				camera: Camera {
					hdr: true,
					..default()
				},
				camera_3d: Camera3d {
					clear_color: ClearColorConfig::Custom(Color::BLACK),
					..default()
				},
				tonemapping: Tonemapping::None,
				..default()
			},
			PrimaryCamera,
			// BloomSettings {
			// 	intensity: 1.0,
			// 	low_frequency_boost: 0.5,
			// 	low_frequency_boost_curvature: 0.5,
			// 	high_pass_frequency: 0.5,
			// 	prefilter_settings: BloomPrefilterSettings {
			// 		threshold: 3.0,
			// 		threshold_softness: 0.6,
			// 	},
			// 	composite_mode: BloomCompositeMode::Additive,
			// },
		))
		.insert(VisibilityBundle::default())
		.named("Main Camera")
		.render_layer(GlobalRenderLayers::InGame);
}

/// Marks the camera that takes center screen.
/// There is not always a camera with this component,
/// but there is at most one.
#[derive(Component, Debug)]
pub struct PrimaryCamera;

impl PrimaryCamera {
	fn exists(cams: Query<(), With<PrimaryCamera>>) -> bool {
		let iter = cams.iter();
		let len = iter.len();
		match len {
			0 => false,
			1 => true,
			_ => {
				warn!(
					"There are {} primary cameras, but there should be at most 1",
					len
				);
				false
			}
		}
	}
}

/// Camera block that is spawned into the world
#[derive(Bundle)]
pub struct CameraBlockBundle {
	pbr: PbrBundle,
	name: Name,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraBlock;

impl BlockBlueprint<CameraBlockBundle> {
	pub fn new_camera(
		position: impl Into<manual_builder::RelativePixel>,
		facing: impl Into<Quat>,
	) -> Self {
		Self {
			transform: Transform::from_rotation(facing.into())
				.translate(position.into().into_world_offset()),
			..todo!()
		}
	}
}
