use bevy::ecs::schedule::{ScheduleBuildSettings, ScheduleLabel};

use crate::prelude::*;

pub use traits::*;

pub struct BlueprintsPlugin;

impl Plugin for BlueprintsPlugin {
	fn build(&self, app: &mut App) {
		type BE = BlueprintExpansion;
		app
			.configure_sets(
				Blueprints,
				(
					BE::ClearJustExpandedMarker,
					(BE::Player, BE::SpawnPoints, BE::Terrain),
					BE::Expand1, // expands player children like structure blueprints
					(BE::ThrusterBlocks,),
					BE::Expand2,
				)
					.chain(),
			)
			.register_type::<JustExpanded>()
			.add_systems(
				Blueprints,
				((
					apply_deferred,
					Self::clear_blueprint_updated_markers,
					apply_deferred,
				)
					.chain()
					.in_set(BE::ClearJustExpandedMarker),),
			)
			.add_systems(
				FixedUpdate,
				Self::run_blueprints_schedule.in_set(GlobalSystemSet::BlueprintExpansion),
			)
			.edit_schedule(Blueprints, |schedule| {
				schedule.set_build_settings(ScheduleBuildSettings {
					ambiguity_detection: bevy::ecs::schedule::LogLevel::Error,
					..default()
				});
			});
	}
}

/// Schedule that runs to fully expand blueprints
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Blueprints;

/// If an entity has this component, it means that it was just spawned
/// / its blueprint was just expanded.
#[derive(Component, Debug, Default, Reflect)]
#[component(storage = "SparseSet")]
pub struct JustExpanded;

/// Makes sure that the blueprints that the player creates are
/// expanded before, so that thruster visuals can be spawned on time.
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlueprintExpansion {
	/// Removes the [JustExpanded] marker components and calls [apply_deferred] (twice)
	ClearJustExpandedMarker,

	/// Expands player's blueprint,
	/// which also spawns a few children.
	Player,
	SpawnPoints,
	/// Asteroids
	Terrain,

	/// Runs [apply_deferred]
	Expand1,

	ThrusterBlocks,

	/// Runs [apply_deferred]
	Expand2,
}

// (BlueprintExpansionClass::Player, BlueprintExpansionClass::Thruster)
// 	.chain()
// 	.in_set(GlobalSystemSet::BlueprintExpansion),

mod systems {
	use crate::prelude::*;

	use super::{JustExpanded, BlueprintsPlugin};

	impl BlueprintsPlugin {
		pub(super) fn clear_blueprint_updated_markers(
			markers: Query<Entity, With<JustExpanded>>,
			mut commands: Commands,
		) {
			for entity in markers.iter() {
				commands.entity(entity).remove::<JustExpanded>();
			}
		}

		pub(super) fn run_blueprints_schedule(world: &mut World) {
			// trace!("Running Blueprints schedule normally");
			world.try_run_schedule(Blueprints).ok();
		}
	}
}

mod traits {
	use crate::prelude::*;

	/// Represents a type [Blueprint] that can be [Blueprint::stamp]ed into
	/// a bundle that can be spawned, i.e., a [Bundle] that is specifically
	/// [Blueprint::For]
	pub trait Blueprint: std::fmt::Debug {
		/// The bundle type that this blueprint can be stamped into.
		type Bundle: Bundle;
		/// A way to access the world when stamping, typically [MMA],
		/// for things like [AssetServer] or [ResMut<Assets<Mesh>>].
		type StampSystemParam<'w, 's>: SystemParam;

		/// Stamps this blueprint into a bundle that can be spawned.
		fn stamp(&self, system_param: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle;
	}

	/// A blueprint that is synced over the network.
	/// Hence, it must be serializable and deserializable,
	/// and contain at least a [bevy_replicon] serializable component.
	pub trait NetworkedBlueprintBundle:
		Bundle + std::ops::Deref<Target = Self::NetworkedBlueprintComponent>
	{
		type NetworkedBlueprintComponent: Component
			+ Serialize
			+ DeserializeOwned
			+ Blueprint
			+ ReplicationMarker;

		/// What access is needed when expanding this blueprint.
		type SpawnSystemParam: SystemParam;

		/// The system that expands this blueprint on both server and client side.
		/// Runs whenever a new instance of this blueprint is spawned.
		/// By default, immediately stamps the blueprint bundle on top of the entity.
		fn expand_system(
			instances: Query<
				(Entity, &Self::NetworkedBlueprintComponent),
				Changed<Self::NetworkedBlueprintComponent>,
			>,
			mut commands: Commands,
			mut expand_system_param: <Self::NetworkedBlueprintComponent as Blueprint>::StampSystemParam<
				'_,
				'_,
			>,
			_spawn_system_param: Self::SpawnSystemParam,
		) {
			for (e, blueprint) in instances.iter() {
				trace!(
					"Expanding blueprint: {:?}",
					// std::any::type_name::<Self::NetworkedBlueprintComponent>(),
					blueprint
				);
				commands
					.entity(e)
					.insert(blueprint.stamp(&mut expand_system_param))
					.insert(JustExpanded);
			}
		}
	}
}
