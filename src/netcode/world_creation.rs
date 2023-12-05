use crate::prelude::*;
	use bevy::ecs::schedule::ScheduleLabel;

	pub(super) struct WorldCreationPlugin;

	impl Plugin for WorldCreationPlugin {
		fn build(&self, app: &mut App) {
			#[allow(clippy::upper_case_acronyms)]
			type WCS = WorldCreationSet;

			app
				.configure_sets(
					WorldCreation,
					(WCS::SpawnPoints, WCS::FlushSpawnPoints, WCS::InitialPlayer).chain(),
				)
				.add_systems(WorldCreation, apply_deferred.in_set(WCS::FlushSpawnPoints))
				.add_event::<CreateWorldEvent>()
				.add_systems(FixedUpdate, Self::handle_world_creation_events);

			let system_id = app.world.register_system(WorldCreation::run_schedule);
			app.world.insert_resource(WorldCreationRunSystem(system_id));
		}
	}

	#[derive(ScheduleLabel, Hash, Debug, Clone, Eq, PartialEq)]
	pub struct WorldCreation;

	#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
	pub enum WorldCreationSet {
		SpawnPoints,
		Asteroids,

		FlushSpawnPoints,
		/// Must be done after spawn points
		InitialPlayer,
	}

	#[derive(Resource)]
	struct WorldCreationRunSystem(SystemId);

	#[derive(Event)]
	pub struct CreateWorldEvent;

	impl WorldCreationPlugin {
		fn handle_world_creation_events(
			mut events: EventReader<CreateWorldEvent>,
			mut commands: Commands,
			system: Res<WorldCreationRunSystem>,
		) {
			for _ in events.read() {
				commands.run_system(system.0);
			}
		}
	}

	impl WorldCreation {
		fn run_schedule(world: &mut World) {
			info!("Running WorldCreation schedule");
			world.try_run_schedule(WorldCreation).ok();
		}
	}
