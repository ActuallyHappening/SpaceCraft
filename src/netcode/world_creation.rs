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
				(
					(WCS::SpawnPoints, WCS::Asteroids),
					WCS::ApplyBlueprints,
					WCS::InitialPlayer,
				)
					.chain(),
			)
			.add_event::<CreateWorldEvent>()
			.add_systems(
				FixedUpdate,
				Self::handle_world_creation_events.in_set(GlobalSystemSet::WorldCreation),
			)
			.add_systems(WorldCreation, apply_deferred.in_set(WCS::ApplyBlueprints));

		let system_id = app.world.register_system(WorldCreation::run_schedule);
		app.world.insert_resource(WorldCreationRunSystem(system_id));
	}
}

#[derive(ScheduleLabel, Hash, Debug, Clone, Eq, PartialEq)]
pub struct WorldCreation;

#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum WorldCreationSet {
	Asteroids,
	SpawnPoints,

	/// runs [apply_deferred] so that the bundles are actually spawned in
	ApplyBlueprints,

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
		// in the future might add more fields to create world event,
		// like creating chunks e.t.c.
		for _ in events.read() {
			commands.run_system(system.0);
		}
	}
}

impl WorldCreation {
	fn run_schedule(world: &mut World) {
		info!("Running WorldCreation schedule");
		world.try_run_schedule(WorldCreation).ok();
		info!("Running Blueprints schedule after WorldCreation");
		world.try_run_schedule(Blueprints).ok();
	}
}
