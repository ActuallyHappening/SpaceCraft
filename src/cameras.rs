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

	/// Entity that is a camera block.
	#[derive(Debug, Clone, Copy, Reflect, Deref)]
	pub struct BlockEntity(pub Entity);

	/// Entity that is a [super::camera_bundle::CameraBundle].
	#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Deref)]
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
			mut config: ResMut<resources::CamerasConfig>,
		) {
			if config.requires_fallback() {
				// there is no cameras already spawned
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
				*config = CamerasConfig::Fallback {
					cam: CameraEntity(entity),
				};
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
						res.clean_any_fallback_cam(&mut commands);
						*res = CamerasConfig::PrimaryCamera {
							block: *follow_camera_block,
							cam: CameraEntity(camera_entity),
							config: Default::default(),
						};
					}
				}
			}
		}

		pub(super) fn update_cameras(
			mut cams: Query<&mut Rig, With<CameraMarker>>,
			blocks: Query<&GlobalTransform, With<CameraBlockMarker>>,
			config: Res<CamerasConfig>,
		) {
			match config.into_inner() {
				CamerasConfig::None => {
					// no cameras spawned
				}
				CamerasConfig::Fallback { .. } => {
					// only a fallback camera spawned, no need to update
				}
				CamerasConfig::PrimaryCamera { block, cam, config } => {
					// primary camera spawned already, lets update it
					if let Ok(mut rig) = cams.get_mut(**cam) {
						if let Ok(block_global_transform) = blocks.get(**block) {
							// translating block to global scope
							let block_transform = block_global_transform
								.reparented_to(&GlobalTransform::IDENTITY);

							// sets position to same as the block it is following
							rig.driver_mut::<bevy_dolly::prelude::Position>().position =
								block_transform.translation;
							// sets rotation to same as the parent block it is following
							rig.driver_mut::<bevy_dolly::prelude::Rotation>().rotation =
								block_transform.rotation;
							
							if config.can_orbit() {

							}
						} else {
							error!("Cannot find block {:?} in query", block);
						}
					} else {
						error!("Cannot find camera {:?} in query", cam);
					}
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
