use bevy::ecs::schedule::ScheduleLabel;

use crate::prelude::*;

pub struct BlueprintsPlugin;

impl Plugin for BlueprintsPlugin {
	fn build(&self, app: &mut App) {
		type BE = BlueprintExpansion;
		app
			.configure_sets(
				Blueprints,
				(
					BE::ClearJustExpandedMarker,
					(BE::Player, BE::SpawnPoints),
					BE::Expand1,
					(BE::Thruster, BE::CameraBlocks),
					BE::Expand2,
				)
					.chain(),
			)
			.register_type::<BlueprintUpdated>()
			.add_systems(
				Blueprints,
				((Self::clear_blueprint_updated_markers, apply_deferred)
					.chain()
					.in_set(BE::ClearJustExpandedMarker)),
			);
	}
}

/// Schedule that runs to fully expand blueprints
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Blueprints;

/// If an entity has just 'refreshed' its blueprint,
/// as in there is data that needs/has been updated from it,
/// then this component should be added.
/// 
/// This allows optimization like only running systems over entities that have
/// just had their blueprints updated / just been spawned in and need hydrating.
#[derive(Component, Debug, Default, Reflect)]
pub struct BlueprintUpdated;

/// Makes sure that the blueprints that the player creates are
/// expanded before, so that thruster visuals can be spawned on time.
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlueprintExpansion {
	/// Removes the [JustExpanded] marker components and calls [apply_deferred]
	ClearJustExpandedMarker,

	/// Expands player's blueprint,
	/// which also spawns a few children.
	Player,
	SpawnPoints,

	/// Runs [apply_deferred]
	Expand1,

	Thruster,
	CameraBlocks,

	/// Runs [apply_deferred]
	Expand2,
}

// (BlueprintExpansionClass::Player, BlueprintExpansionClass::Thruster)
// 	.chain()
// 	.in_set(GlobalSystemSet::BlueprintExpansion),

mod systems {
	use crate::prelude::*;

	use super::{BlueprintsPlugin, BlueprintUpdated};

	impl BlueprintsPlugin {
		pub(super) fn clear_blueprint_updated_markers(markers: Query<Entity, With<BlueprintUpdated>>, mut commands: Commands) {
			for entity in markers.iter() {
				commands.entity(entity).remove::<BlueprintUpdated>();
			}
		}
	}
}
