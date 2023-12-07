//! Responsible for chosing which thrusters go on,
//! and taking input from user.
//!
//! Is designed to be generic over where thrusters are placed
//! and their rotations, so that building your own ships is possible.

use crate::prelude::*;

pub use api::*;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
	fn build(&self, app: &mut App) {}
}

mod systems {
	use super::PlayerMovementPlugin;
	use crate::prelude::*;

	impl PlayerMovementPlugin {
		pub(super) fn get_player_data() {}
	}
}

mod components {
	use crate::prelude::*;

	/// Stores all of the data concerning thruster movements.
	/// Placed on players
	#[derive(Component, Debug, Reflect)]
	pub(super) struct ThrusterData {
		blocks: HashMap<BlockId, (ForceAxis, f32)>,
	}

	#[derive(Debug, Reflect)]
	struct ForceAxis {
		forward: f32,
		right: f32,
		upwards: f32,
		roll_right: f32,
		turn_right: f32,
		pitch_up: f32,
	}

	impl ThrusterData {
		pub(super) fn get_blocks_strength(&self) -> HashMap<&BlockId, &f32> {
			self
				.blocks
				.iter()
				.map(|(id, (_, strength))| (id, strength))
				.collect()
		}
	}
}

/// Public usage that is exported from this crate
mod api {
	use super::components::ThrusterData;
	use crate::prelude::*;

	#[derive(SystemParam)]
	pub struct GetThrusterData<'w, 's> {
		players: Query<'w, 's, &'static ThrusterData>,
	}

	impl GetThrusterData<'_, '_> {
		/// Returns all of the thruster data for **EVERY* player
		pub fn get_all(&self) -> HashMap<&BlockId, &f32> {
			self
				.players
				.iter()
				.flat_map(ThrusterData::get_blocks_strength)
				.collect()
		}
	}
}
