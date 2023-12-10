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
#[reflect(Default)]
pub(super) enum PrimaryCam {
	Primary(BlockEntity, CameraEntity),
	Fallback(Option<Entity>),
}

impl Default for PrimaryCam {
	fn default() -> Self {
		Self::Fallback(None)
	}
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
