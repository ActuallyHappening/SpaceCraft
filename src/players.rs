use crate::prelude::*;

pub use player::{ControllablePlayer, PlayerBlueprint, PlayerMovement};
mod player;

pub use spawn_points::AvailableSpawnPoints;
mod spawn_points;

mod player_movement;
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
