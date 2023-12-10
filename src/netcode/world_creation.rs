use crate::prelude::*;
use bevy::ecs::schedule::{ScheduleBuildSettings, ScheduleLabel};

pub(super) struct WorldCreationPlugin;

impl Plugin for WorldCreationPlugin {
	fn build(&self, app: &mut App) {
		#[allow(clippy::upper_case_acronyms)]
		type WCS = WorldCreationSet;

		app
			.add_event::<CreateWorldEvent>()
			.add_systems(
				FixedUpdate,
				Self::handle_world_creation_events.in_set(GlobalSystemSet::WorldCreation),
			)
			.edit_schedule(WorldCreation, |schedule| {
				schedule.set_build_settings(ScheduleBuildSettings {
					ambiguity_detection: bevy::ecs::schedule::LogLevel::Error,
					..default()
				});
			});
	}
}

#[derive(ScheduleLabel, Hash, Debug, Clone, Eq, PartialEq)]
pub struct WorldCreation;

/// System ordering for the [WorldCreation] [Schedule]
#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum WorldCreationSet {
	Asteroids,
	SpawnPoints,
}

#[derive(Event, Debug)]
pub struct CreateWorldEvent;

impl WorldCreationPlugin {
	fn handle_world_creation_events(
		world: &mut World,
		// mut events: EventReader<CreateWorldEvent>,
	) {
		// in the future might add more fields to create world event,
		// like creating chunks e.t.c.
		world.resource_scope(|world: &mut World, events: Mut<Events<CreateWorldEvent>>| {
			for e in events.get_reader().read(&events) {
				info!("Running WorldCreation schedule in response to {:?}", e);
				world.run_schedule(WorldCreation);
				world.run_schedule(Blueprints);
			}
		});
	}
}
