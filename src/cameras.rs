//! How cameras work:
//!
//! [self::Cameras] is a resource that can be modified by other systems externally.
//! It tells use the specification of the UI of the cameras, for example:
//! - Follow the local player around normally
//! - Also, show a secondary cameras in the bottom left
//!
//! Where cameras are is entirely dependent on the [self::CameraBlock] component,
//! which is added only through the [CameraBlockBundle] block.
//! If following a player, it knows through [ControllablePlayer] and looks
//! for *direct children* with the [self::CameraBlock] component.

use crate::{netcode::ClientID, players::ControllablePlayer, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		// app.add_systems(Startup, spawn_default_cameras);
		app
			.init_resource::<resources::CamerasConfig>()
			.register_type::<resources::CamerasConfig>();
			// .add_systems(Update, Self::sync_cameras.in_set(Client));
	}
}

mod camera_bundle {
	use crate::prelude::*;
	use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};
	use bevy_dolly::dolly_type::Rig;

	/// Cameras spawned into the world.
	/// Is not replicated, is not serialized, and
	/// is managed by [super::CameraPlugin]
	#[derive(Bundle)]
	struct CameraBundle {
		cam: Camera3dBundle,
		rig: Rig,
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
				rig: Rig::builder()
					.with(bevy_dolly::prelude::Position::new(Vec3::ZERO))
					.with(bevy_dolly::prelude::Rotation::new(Quat::IDENTITY))
					.build(),
				name: Name::new("Camera"),
				render_layer: GlobalRenderLayers::InGame.into(),
				vis: VisibilityBundle::default(),
				bloom: Self::DEFAULT_BLOOM,
			}
		}
	}
}

mod systems {
	use crate::prelude::*;


}

mod resources {
	use crate::prelude::*;

	/// Holds state about the cameras of the game.
	///
	/// Public so that UI can change where camera is pointing
	/// e.g. in load screen point towards highest ranked player
	#[derive(Resource, Debug, Default, Reflect)]
	pub struct CamerasConfig {
		/// Pointer to the block
		primary_cam: Option<Entity>,
		/// Pointer to blocks, with extra configuration for the cameras
		secondary_cams: Vec<(Entity, SecondaryCameraConfig)>,
	}

	#[derive(Debug, Reflect)]
	pub struct SecondaryCameraConfig {
		anchor: global::UiCameras,
		width: f32,
		height: f32,
	}

	impl CamerasConfig {
		/// BlockId should be valid
		pub fn set_primary_cam(&mut self, block_id: Entity) {
			self.primary_cam = Some(block_id);
		}
		pub fn get_primary_cam(&self) -> Option<Entity> {
			self.primary_cam
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
		marker: CameraBlockMarker,
	}

	/// Blueprint for [CameraBlockBundle]
	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub struct CameraBlock {
		id: BlockId,
	}

	#[derive(Component)]
	pub struct CameraBlockMarker;

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
				marker: CameraBlockMarker,
			}
		}
	}
}
