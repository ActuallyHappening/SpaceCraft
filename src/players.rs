use crate::prelude::*;

mod player;
mod player_movement;
mod spawn_points;
mod thruster_block;

/// Plugin Group
pub struct PlayerPlugins;

/// Handled in [self::player_movement]
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum PlayerMovementSet {
	/// After this set, the strengths for each player are computed
	ComputeStrengths,

	/// Must be done after [PlayerMovementSet::ComputeStrengths], syncs
	/// the thrusters with the physics sim (and visuals)
	EnactThrusters,
}

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(player::PlayerPlugin)
			.add(thruster_block::ThrusterPlugin)
			.add(spawn_points::SpawnPointsPlugin)
			.add(player_movement::PlayerMovementPlugin)
			.build()
	}
}
