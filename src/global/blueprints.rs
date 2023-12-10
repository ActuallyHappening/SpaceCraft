use bevy::ecs::schedule::ScheduleLabel;

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
					(BE::Player, BE::SpawnPoints),
					BE::Expand1, // expands player children like structure blueprints
					(BE::ThrusterBlocks, BE::CameraBlocks, BE::StructureBlocks),
					BE::Expand2,
				)
					.chain(),
			)
			.register_type::<BlueprintUpdated>()
			.add_systems(
				Blueprints,
				((Self::clear_blueprint_updated_markers, apply_deferred)
					.chain()
					.in_set(BE::ClearJustExpandedMarker),),
			)
			.add_systems(
				FixedUpdate,
				Self::run_blueprints_schedule.in_set(GlobalSystemSet::BlueprintExpansion),
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

	ThrusterBlocks,
	CameraBlocks,
	StructureBlocks,

	/// Runs [apply_deferred]
	Expand2,
}

// (BlueprintExpansionClass::Player, BlueprintExpansionClass::Thruster)
// 	.chain()
// 	.in_set(GlobalSystemSet::BlueprintExpansion),

mod systems {
	use crate::prelude::*;

	use super::{BlueprintUpdated, BlueprintsPlugin};

	impl BlueprintsPlugin {
		pub(super) fn clear_blueprint_updated_markers(
			markers: Query<Entity, With<BlueprintUpdated>>,
			mut commands: Commands,
		) {
			for entity in markers.iter() {
				commands.entity(entity).remove::<BlueprintUpdated>();
			}
		}

		pub(super) fn run_blueprints_schedule(world: &mut World) {
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
					.insert(BlueprintUpdated);
			}
		}
	}
}
