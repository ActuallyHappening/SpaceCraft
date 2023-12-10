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
		marker: CameraMarker,
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
				marker: CameraMarker,
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
					let entity = commands
						.spawn(Camera3dBundle {
							camera: Camera {
								hdr: true,
								..default()
							},
							camera_3d: Camera3d {
								clear_color: ClearColorConfig::Custom(Color::BLUE),
								..default()
							},
							..default()
						})
						.id();
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

mod resources;

mod events {
	use crate::prelude::*;

	use super::BlockEntity;

	#[derive(Debug, Event)]
	pub enum ChangeCameraConfig {
		SetPrimaryCamera { follow_camera_block: BlockEntity },
	}
}

mod camera_block;
