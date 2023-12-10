use crate::prelude::*;

use bevy_dolly::system::Dolly;

pub use api::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		// app.add_systems(Startup, spawn_default_cameras);
		app
			.register_type::<resources::CamerasConfig>()
			.register_type::<api::CameraEntity>()
			.init_resource::<resources::CamerasConfig>()
			.add_event::<events::ChangeCameraConfig>()
			.add_systems(
				Update,
				(
					Self::handle_fallback_cam,
					Self::handle_change_camera_events,
					Self::update_cameras,
					Dolly::<camera_bundle::CameraMarker>::update_active,
				)
					.chain()
					.in_set(Client),
			);
	}
}

mod api {
	use crate::prelude::*;

	pub use super::camera_block::{CameraBlockBlueprint, CameraBlockBundle, CameraBlockMarker};
	pub use super::events::ChangeCameraConfig;

	#[derive(Debug, Clone, Copy, Reflect)]
	pub struct BlockEntity(pub Entity);

	/// Marker for a [camera_bundle::CameraBundle].
	#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash)]
	pub struct CameraEntity(pub Entity);

	#[derive(Debug, Reflect)]
	pub struct SecondaryCameraConfig {
		anchor: global::UiCameras,
		width: f32,
		height: f32,
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
	pub(super) struct CameraBundle {
		cam: Camera3dBundle,
		rig: Rig,
		// bloom: BloomSettings,
		render_layer: RenderLayers,
		name: Name,
		vis: VisibilityBundle,
		id: CameraMarker,
	}

	/// Marker for actual cameras, [CameraBundle]
	#[derive(Component)]
	pub(super) struct CameraMarker;

	impl CameraBundle {
		/// When bloom is enabled, this is the [BloomSettings] that
		/// should be used.
		pub const _DEFAULT_BLOOM: BloomSettings = BloomSettings {
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

		// pub const DEFAULT_BLOOM: BloomSettings = BloomSettings::NATURAL;
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
				// bloom: Self::DEFAULT_BLOOM,
				id: CameraMarker,
			}
		}
	}
}

mod systems {
	use bevy_dolly::prelude::*;

	use super::{
		camera_bundle::{CameraBundle, CameraMarker},
		events::ChangeCameraConfig,
		resources::{self, CamerasConfig},
		CameraBlockMarker, CameraEntity, CameraPlugin,
	};
	use crate::prelude::*;

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
					let entity = commands.spawn(Camera3dBundle::default()).id();
					debug!("Spawned a fallback camera");
					config.set_fallback_cam(entity, &mut commands);
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
		) {
			for e in events.read() {
				match e {
					ChangeCameraConfig::SetPrimaryCamera {
						follow_camera_block,
					} => {
						// spawns new camera
						let camera_entity = commands.spawn(CameraBundle::default()).id();
						res.set_primary_cam(
							*follow_camera_block,
							CameraEntity(camera_entity),
							&mut commands,
						);
					}
				}
			}
		}

		pub(super) fn update_cameras(
			mut cams: Query<(Entity, &mut Rig), With<CameraMarker>>,
			blocks: Query<&GlobalTransform, With<CameraBlockMarker>>,
			config: Res<CamerasConfig>,
		) {
			for (camera_entity, mut rig) in cams.iter_mut() {
				let camera_entity = CameraEntity(camera_entity);
				if let Some(block_entity) = config.get_cam_from_id(camera_entity) {
					if let Ok(block_global_transform) = blocks.get(block_entity.0) {
						let block_transform = block_global_transform.reparented_to(&GlobalTransform::IDENTITY);
						// sets position to same as the block it is following
						rig.driver_mut::<bevy_dolly::prelude::Position>().position =
							block_transform.translation;
						// sets rotation to same as the parent block it is following is
						rig.driver_mut::<bevy_dolly::prelude::Rotation>().rotation = block_transform.rotation;
					} else {
						error!("Camera {:?} has no global transform", camera_entity);
					}
				} else {
					warn!(
						"Camera {:?} has no block entity assigned in CamerasConfig resource",
						camera_entity
					);
				}
			}
		}
	}
}

mod resources {
	use crate::prelude::*;

	use super::{BlockEntity, CameraEntity};

	/// Holds state about the cameras of the game.
	///
	/// Public so that UI can change where camera is pointing
	/// e.g. in load screen point towards highest ranked player
	#[derive(Resource, Debug, Reflect)]
	#[reflect(Resource)]
	pub(super) struct CamerasConfig {
		/// Pointer to the block entity the primary camera should
		/// be following, and the actual ID of the Camera.
		///
		/// When [Err], is actually a default camera which should be removed
		/// when a better primary camera can be spawned.
		primary_cam: PrimaryCam,

		/// Pointer to blocks, with extra configuration for the cameras.
		secondary_cams: HashMap<CameraEntity, (BlockEntity, super::SecondaryCameraConfig)>,
	}

	#[derive(Debug, Reflect)]
	pub(super) enum PrimaryCam {
		Primary(BlockEntity, CameraEntity),
		Fallback(Option<Entity>),
	}

	use PrimaryCam::{Fallback, Primary};

	impl PrimaryCam {
		pub fn get_fallback(&self) -> Option<Option<Entity>> {
			match self {
				Self::Fallback(e) => Some(*e),
				Self::Primary(_, _) => None,
			}
		}

		pub fn get_primary(&self) -> Option<(BlockEntity, CameraEntity)> {
			match self {
				Self::Fallback(_) => None,
				Self::Primary(b, c) => Some((*b, *c)),
			}
		}
	}

	impl Default for CamerasConfig {
		fn default() -> Self {
			CamerasConfig {
				primary_cam: Fallback(None),
				secondary_cams: HashMap::new(),
			}
		}
	}

	impl CamerasConfig {
		pub fn get_fallback_cam(&self) -> Option<Option<Entity>> {
			self.primary_cam.get_fallback()
		}

		/// If a fallback camera is in place, despawn it
		fn clean_fallback_cam(&mut self, commands: &mut Commands) {
			if let Fallback(Some(e)) = &mut self.primary_cam {
				debug!("Despawning old fallback camera");
				commands.entity(*e).despawn_recursive();
				self.primary_cam = Fallback(None);
			}
		}

		fn clean_primary_cam(&mut self, commands: &mut Commands) {
			if let Primary(_, e) = &self.primary_cam {
				debug!("Despawning old primary camera");
				commands.entity(e.0).despawn_recursive();
				self.primary_cam = Fallback(None);
			}
		}

		/// Assumes you have spawned a camera, and want to set it
		/// as the primary camera. `entity` is the entity of the [CameraBundle]
		pub fn set_fallback_cam(&mut self, entity: Entity, commands: &mut Commands) {
			self.clean_fallback_cam(commands);
			match &mut self.primary_cam {
				Fallback(_) => {
					self.primary_cam = Fallback(Some(entity));
				}
				Primary(_, _) => error!("Trying to set the fallback cam when a fallback is not needed"),
			}
		}

		// pub fn get_primary_cam(&self) -> Option<(BlockEntity, CameraEntity)> {
		// 	self.primary_cam.get_primary()
		// }

		/// Sets the primary camera to follow the given camera block entity.
		/// Must have already spawned the camera.
		///
		/// Automatically clears up old cameras in the process, like
		/// fallback and primary.
		pub fn set_primary_cam(
			&mut self,
			block_entity: BlockEntity,
			camera_entity: CameraEntity,
			commands: &mut Commands,
		) {
			self.clean_fallback_cam(commands);
			self.clean_primary_cam(commands);
			self.primary_cam = Primary(block_entity, camera_entity);
		}

		/// Returns the camera [BlockEntity] that the camera
		/// with the given [CameraId] is following.
		pub fn get_cam_from_id(&self, id: CameraEntity) -> Option<BlockEntity> {
			if let Some((block_entity, _)) = self.secondary_cams.get(&id) {
				Some(*block_entity)
			} else if let Some((block_entity, primary_id)) = self.primary_cam.get_primary() {
				if primary_id == id {
					Some(block_entity)
				} else {
					None
				}
			} else {
				None
			}
		}
	}
}

mod events {
	use crate::prelude::*;

	use super::BlockEntity;

	#[derive(Debug, Event)]
	pub enum ChangeCameraConfig {
		SetPrimaryCamera { follow_camera_block: BlockEntity },
	}
}

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
	#[derive(Debug, Reflect, Serialize, Deserialize, Clone)]
	pub struct CameraBlockBlueprint {
		pub id: BlockId,
	}

	/// Marker for [BlockBlueprint]s that are [CameraBlockBlueprint]s,
	/// which spawn [CameraBlockBundle]s.
	#[derive(Component)]
	pub struct CameraBlockMarker;

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

	impl Blueprint for BlockBlueprint<CameraBlockBlueprint> {
		type Bundle = CameraBlockBundle;
		type StampSystemParam<'w, 's> = MMA<'w>;

		fn stamp(&self, mma: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle {
			let BlockBlueprint {
				transform,
				mesh,
				material,
				specific_marker,
			} = self;
			Self::Bundle {
				pbr: PbrBundle {
					transform: *transform,
					mesh: mesh.clone().into_mesh(mma),
					material: material.clone().into_material(&mut mma.mats),
					..default()
				},
				name: Name::new("CameraBlock"),
				id: specific_marker.id,
				marker: CameraBlockMarker,
			}
		}
	}
}
