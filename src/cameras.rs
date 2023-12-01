use crate::prelude::*;

#[allow(unused_imports)]
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		// app.add_systems(Startup, spawn_default_cameras);
		app
			.init_resource::<Cameras>()
			.register_type::<Cameras>()
			.add_systems(Update, Self::sync_cameras);
	}
}

impl CameraPlugin {
	/// Syncs the resource [Cameras] with the actual cameras in the world
	fn sync_cameras(config: Res<Cameras>) {}
}

/// Cameras spawned into the world
#[derive(Bundle)]
struct CameraBundle {
	cam: Camera3dBundle,
	bloom: BloomSettings,
	render_layer: RenderLayers,
	name: Name,
	vis: VisibilityBundle,
}

impl CameraBundle {
	/// When bloom is enabled, this is the [BloomSettings] that
	/// should be used.
	pub const DEFAULT_BLOOM: BloomSettings = BloomSettings {
		intensity: 1.0,
		low_frequency_boost: 0.5,
		low_frequency_boost_curvature: 0.5,
		high_pass_frequency: 0.5,
		prefilter_settings: BloomPrefilterSettings {
			threshold: 3.0,
			threshold_softness: 0.6,
		},
		composite_mode: BloomCompositeMode::Additive,
	};
}

impl Default for CameraBundle {
	fn default() -> Self {
		CameraBundle {
			cam: Camera3dBundle {
				transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
				camera: bevy::prelude::Camera {
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
			name: Name::new("Camera"),
			render_layer: GlobalRenderLayers::InGame.into(),
			vis: VisibilityBundle::default(),
			bloom: Self::DEFAULT_BLOOM,
		}
	}
}

/// Marks the camera that takes center screen.
/// There is not always a camera with this component,
/// but there is at most one.
#[derive(Component, Debug, Reflect)]
enum Camera {
	Primary,
	Secondary {
		width: f32,
		height: f32,
		anchor: global::UiCameras,
	},
}

/// Holds state about the cameras of the game.
/// 
/// Public so that UI can change where camera is pointing
/// e.g. in load screen point towards highest ranked player
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
