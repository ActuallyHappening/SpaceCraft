use crate::prelude::*;

use super::{BlockEntity, CameraEntity};

/// Holds state about the cameras of the game.
///
/// Public so that UI can change where camera is pointing
/// e.g. in load screen point towards highest ranked player
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub(super) enum CamerasConfig {
	/// No cameras currently spawned (default)
	#[default]
	None,

	/// A fallback cameras has been spawned, but no camera block has been selected
	Fallback { cam: CameraEntity },

	/// A primary camera that takes up the whole screen has been set
	PrimaryCamera {
		block: BlockEntity,
		cam: CameraEntity,
		config: CameraConfiguration,
	},
}

#[derive(Debug, Reflect)]
pub struct CameraConfiguration {
	allow_orbit: bool,
}

impl Default for CameraConfiguration {
	fn default() -> Self {
		CameraConfiguration { allow_orbit: true }
	}
}

impl CamerasConfig {
	pub(super) fn clean_any_fallback_cam(&self, commands: &mut Commands) {
		if let CamerasConfig::Fallback { cam } = self {
			commands.entity(**cam).despawn_recursive();
		}
	}

	pub(super) fn requires_fallback(&self) -> bool {
		matches!(self, CamerasConfig::None)
	}
}

impl CameraConfiguration {
	pub(super) fn can_orbit(&self) -> bool {
		self.allow_orbit
	}
}