use crate::prelude::*;

#[allow(unused_imports)]
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		// app.add_systems(Startup, spawn_default_cameras);
		app.init_resource::<Cameras>().register_type::<Cameras>().add_systems(
			Update,
			Self::sync_cameras,
		);
	}
}

impl CameraPlugin {
	/// Syncs the resource [Cameras] with the actual cameras in the world
	fn sync_cameras(config: Res<Cameras>,) {}
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
struct PrimaryCamera;

/// Holds state about the cameras of the game
#[derive(Resource, Debug, Default, Reflect)]
pub struct Cameras {
	primary_cam: CameraConfig,
}

/// State of a camera
#[derive(Debug, Reflect, Default)]
pub enum CameraConfig {
	#[default]
	FollowLocalPlayer,
	FollowingCameraBlock(BlockId),
}

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

pub use camera_block::*;
mod camera_block {
	use crate::prelude::*;

	/// Camera block that is spawned into the world
	#[derive(Bundle)]
	pub struct CameraBlockBundle {
		pbr: PbrBundle,
		name: Name,
		id: BlockId,
	}

	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub struct CameraBlock {
		id: BlockId,
	}

	impl BlockBlueprint<CameraBlock> {
		pub fn new_camera(
			position: impl Into<manual_builder::RelativePixel>,
			facing: impl Into<Quat>,
		) -> Self {
			Self {
				transform: Transform::from_rotation(facing.into())
					.translate(position.into().into_world_offset()),
				mesh: OptimizableMesh::None,
				material: OptimizableMaterial::None,
				specific_marker: CameraBlock {
					id: BlockId::random(),
				},
			}
		}
	}

	impl FromBlueprint for CameraBlockBundle {
		type Blueprint = BlockBlueprint<CameraBlock>;

		fn stamp_from_blueprint(
			BlockBlueprint {
				transform,
				mesh,
				material,
				specific_marker,
			}: &Self::Blueprint,
			mma: &mut MMA,
		) -> Self {
			Self {
				pbr: PbrBundle {
					transform: *transform,
					mesh: mesh.get_mesh(mma),
					material: material.get_material(&mut mma.mats),
					..default()
				},
				name: Name::new("CameraBlock"),
				id: specific_marker.id,
			}
		}
	}
}
