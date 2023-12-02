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

pub use events::ChangeCameraConfig;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		// app.add_systems(Startup, spawn_default_cameras);
		app
			.register_type::<resources::CamerasConfig>()
			.register_type::<CameraId>()
			.init_resource::<resources::CamerasConfig>()
			.add_event::<ChangeCameraConfig>()
			.add_systems(
				Update,
				(
					Self::handle_fallback_cam,
					Self::handle_change_camera_events,
				)
					.in_set(Client),
			);
	}
}

/// Marker for a [camera_bundle::CameraBundle].
#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
struct CameraId(BlockId);

impl CameraId {
	fn random() -> Self {
		Self(BlockId::random())
	}
}

#[derive(Debug, Reflect)]
pub struct SecondaryCameraConfig {
	anchor: global::UiCameras,
	width: f32,
	height: f32,
}

mod camera_bundle {
	use crate::prelude::*;
	use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};
	use bevy_dolly::dolly_type::Rig;

	use super::CameraId;

	/// Cameras spawned into the world.
	/// Is not replicated, is not serialized, and
	/// is managed by [super::CameraPlugin]
	#[derive(Bundle)]
	pub(super) struct CameraBundle {
		cam: Camera3dBundle,
		rig: Rig,
		bloom: BloomSettings,
		render_layer: RenderLayers,
		name: Name,
		vis: VisibilityBundle,
		id: super::CameraId,
	}

	impl CameraBundle {
		pub fn default_new(id: CameraId) -> Self {
			Self { id, ..default() }
		}
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
				id: super::CameraId::random(),
			}
		}
	}
}

mod systems {
	use super::{
		camera_bundle::CameraBundle, events::ChangeCameraConfig, resources, CameraBlockMarker,
		CameraId, CameraPlugin,
	};
	use crate::{netcode::ClientID, prelude::*, players::ControllablePlayer};

	impl CameraPlugin {
		/// Spawns the fallback camera if needed
		pub(super) fn handle_fallback_cam(
			mut commands: Commands,
			config: ResMut<resources::CamerasConfig>,
		) {
			let config = config.into_inner();
			match config.get_fallback_cam() {
				Some(None) => {
					// spawn a fallback camera
					let id = commands.spawn(Camera3dBundle::default()).id();
					debug!("Spawned a fallback camera");
					config.set_fallback_cam(id);
				}
				Some(Some(_)) => {
					// a fallback camera has already been spawned
					// and should stay there
				}
				None => {
					// there is already a primary camera
				}
			}
		}

		pub(super) fn handle_change_camera_events(
			mut commands: Commands,
			mut events: EventReader<ChangeCameraConfig>,
			mut res: ResMut<resources::CamerasConfig>,

			blocks: Query<(Entity, &BlockId), With<CameraBlockMarker>>,
			cameras: Query<(Entity, &CameraId)>,
		) {
			for e in events.read() {
				match e {
					ChangeCameraConfig::SetPrimaryCamera { follow_block } => {
						if let Some((camera_block, _)) = blocks.iter().find(|(_, b)| *b == follow_block) {
							// removes old cameras
							if let Some((_, camera_id)) = res.get_primary_cam() {
								if let Some((e, id)) = cameras.iter().find(|(_, id)| **id == camera_id) {
									debug!("Despawning old primary camera {:?} {:?}", e, id);
									commands.entity(e).despawn_recursive();
								} else {
									warn!("Can't find old primary camera to despawn");
								}
							}

							// spawns new camera
							let camera_id = CameraId::random();
							commands.spawn(CameraBundle::default_new(camera_id));
							res.set_primary_cam(camera_block, camera_id, &mut commands);
						}
					}
				}
			}
		}
	}
}

mod resources {
	use crate::prelude::*;

	use super::CameraId;

	/// Holds state about the cameras of the game.
	///
	/// Public so that UI can change where camera is pointing
	/// e.g. in load screen point towards highest ranked player
	#[derive(Resource, Debug, Reflect)]
	pub(super) struct CamerasConfig {
		/// Pointer to the block entity the primary camera should
		/// be following, and the actual ID of the Camera.
		///
		/// When [Err], is actually a default camera which should be removed
		/// when a better primary camera can be spawned.
		primary_cam: Result<(Entity, CameraId), Option<Entity>>,

		/// Pointer to blocks, with extra configuration for the cameras.
		secondary_cams: Vec<(Entity, CameraId, super::SecondaryCameraConfig)>,
	}

	impl Default for CamerasConfig {
		fn default() -> Self {
			CamerasConfig {
				primary_cam: Err(None),
				secondary_cams: Vec::new(),
			}
		}
	}

	impl CamerasConfig {
		pub fn get_fallback_cam(&self) -> Option<Option<Entity>> {
			self.primary_cam.err()
		}

		pub fn set_fallback_cam(&mut self, entity: Entity) {
			match &mut self.primary_cam {
				Err(e) => {
					*e = Some(entity);
				}
				Ok(_) => error!("Trying to set the fallback cam when a fallback is not needed"),
			}
		}

		pub fn get_primary_cam(&self) -> Option<(Entity, CameraId)> {
			self.primary_cam.ok()
		}

		pub fn set_primary_cam(
			&mut self,
			block_entity: Entity,
			camera_id: CameraId,
			commands: &mut Commands,
		) {
			if let Err(Some(e)) = self.primary_cam {
				commands.entity(e).despawn_recursive()
			}
			self.primary_cam = Ok((block_entity, camera_id));
		}
	}
}

mod events {
	use bevy::ecs::event::Event;

	use crate::blocks::BlockId;

	#[derive(Debug, Event)]
	pub enum ChangeCameraConfig {
		SetPrimaryCamera {
			/// Eventually converted into an [Entity]
			follow_block: BlockId,
		},
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
	pub struct CameraBlockBlueprint {
		id: BlockId,
	}

	#[derive(Component)]
	pub(super) struct CameraBlockMarker;

	impl BlockBlueprint<CameraBlockBlueprint> {
		pub fn new_camera(
			position: impl Into<manual_builder::RelativePixel>,
			facing: impl Into<Quat>,
		) -> Self {
			Self {
				transform: Transform::from_rotation(facing.into())
					.translate(position.into().into_world_offset()),
				mesh: OptimizableMesh::Sphere {
					radius: PIXEL_SIZE / 2.,
				},
				material: OptimizableMaterial::OpaqueColour(Color::BLACK),
				specific_marker: CameraBlockBlueprint {
					id: BlockId::random(),
				},
			}
		}
	}

	impl FromBlueprint for CameraBlockBundle {
		type Blueprint = BlockBlueprint<CameraBlockBlueprint>;

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
