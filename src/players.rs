use crate::prelude::*;

mod player;
mod player_movement;
mod spawn_points;
mod thruster_block;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(player::PlayerPlugin)
			.add(thruster_block::ThrusterPlugin)
			.add(self::spawn_points::SpawnPointsPlugin)
			.add(self::player_movement::PlayerMovementPlugin)
			.build()
	}
}
