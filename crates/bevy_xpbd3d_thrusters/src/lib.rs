#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

pub mod prelude {
	pub use crate::plugins::{ThrusterPlugin, ThrusterSystemSet};
	pub use crate::shared_types::{components::Thruster, ForceAxis};

	pub use bevy_xpbd3d_parenting::prelude::*;

	pub(crate) use bevy::{prelude::*, utils::HashMap};
	pub(crate) use bevy_inspector_egui::prelude::*;
}

mod plugins {
	use std::marker::PhantomData;

	use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};

	use crate::prelude::*;

	#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
	pub enum ThrusterSystemSet {
		PrepareThrusters,

		/// See [ThrusterPlugin::sync_thrusters_with_internal_forces]
		SyncInternalForces,
	}

	/// Minimal plugin for syncing the [Thruster] component with
	/// [InternalForce]s.
	pub struct ThrusterPlugin {
		schedule: InternedScheduleLabel,
		_ph: PhantomData<()>,
	}

	impl ThrusterPlugin {
		pub fn new(schedule: impl ScheduleLabel) -> Self {
			Self {
				schedule: schedule.intern(),
				_ph: PhantomData,
			}
		}
	}

	impl Default for ThrusterPlugin {
		fn default() -> Self {
			ThrusterPlugin::new(Update)
		}
	}

	impl Plugin for ThrusterPlugin {
		fn build(&self, app: &mut App) {
			#[allow(clippy::upper_case_acronyms)]
			type TSS = ThrusterSystemSet;

			app
				.register_type::<ForceAxis>()
				.register_type::<Thruster>()
				.configure_sets(
					self.schedule,
					(TSS::PrepareThrusters, TSS::SyncInternalForces).chain(),
				)
				.add_systems(
					self.schedule,
					(
						Self::prepare_thrusters.in_set(TSS::PrepareThrusters),
						Self::sync_thrusters_with_internal_forces.in_set(TSS::SyncInternalForces),
					),
				);
		}
	}
}

mod systems {
	use crate::{plugins::ThrusterPlugin, prelude::*};

	impl ThrusterPlugin {
		/// Reads data from [Thruster]s, and applies it to the physics simulation
		pub(super) fn sync_thrusters_with_internal_forces(
			mut thrusters: Query<(&Thruster, &mut InternalForce)>,
		) {
			for (thruster, mut internal_force) in thrusters.iter_mut() {
				internal_force
					.set(Vec3::Z * thruster.get_current_status() * thruster.get_strength_factor());
			}
		}

		pub(super) fn prepare_thrusters(
			unprepared_thrusters: Query<
				(Entity, Option<&Name>),
				(With<Thruster>, Without<InternalForce>),
			>,
			mut commands: Commands,
		) {
			for (thruster, name) in unprepared_thrusters.iter() {
				let _e = commands.entity(thruster).insert(InternalForce::default());

				#[cfg(feature = "debug")]
				debug!(
					"Added `InternalForce` to an entity {:?} with the `Thruster` component {:?}",
					thruster,
					match name {
						Some(name) => format!("named {}", name.as_str()),
						None => "and no name".into(),
					}
				);
			}
		}
	}
}

mod shared_types {
	use bevy::prelude::*;

	/// Forces are between [-1, 1],
	/// Torques are 'normalized' to [-1, 1] // TODO
	///
	/// The Greek philosopher, Archimedes, said,
	/// “Give me a lever long enough and a fulcrum on which to place it, and I shall move the world.”
	#[derive(Reflect, Debug, Default, Clone, Copy)]
	pub struct ForceAxis {
		forward: f32,
		right: f32,
		upwards: f32,
		turn_right: f32,
		pitch_up: f32,
		roll_right: f32,
	}

	pub(crate) mod components {
		use crate::prelude::*;

		/// Component for all thrusters
		#[derive(Debug, Component, Reflect, Clone, InspectorOptions)]
		#[reflect(InspectorOptions)]
		pub struct Thruster {
			/// Factor which is multiplied by [Thruster.current_status] to get the actual force.
			/// Does not allow negative values, because thrusters firing 'backwards' is not yet a desired behavior
			#[inspector(min = 0.0)]
			strength_factor: f32,

			/// Between 0..=1, synced with visuals and physics
			#[inspector(min = 0.0, max = 1.0)]
			current_status: f32,
		}

		impl Default for Thruster {
			fn default() -> Self {
				Self::new_with_strength_factor(1.0)
			}
		}

		impl Thruster {
			pub fn new_with_strength_factor(strength_factor: f32) -> Self {
				Self {
					strength_factor,
					current_status: 0.0,
				}
			}

			/// Creates a [Thruster] using the [Default] values.
			pub fn new() -> Self {
				Self::default()
			}

			pub fn get_strength_factor(&self) -> f32 {
				self.strength_factor.max(0.0)
			}

			pub fn set_strength_factor(&mut self, strength_factor: f32) {
				#[cfg(feature = "debug")]
				if strength_factor < 0.0 {
					warn!("Strength factor {} must be >= 0.0", strength_factor);
				}
				self.strength_factor = strength_factor.max(0.0);
			}

			pub fn get_current_status(&self) -> f32 {
				self.current_status.clamp(0.0, 1.0)
			}

			pub fn set_current_status(&mut self, current_status: f32) {
				#[cfg(feature = "debug")]
				if !(0.0..=1.0).contains(&current_status) {
					warn!(
						"Current status {} must be between 0.0 and 1.0 (inclusive)",
						current_status
					);
				}
				self.current_status = current_status.clamp(0.0, 1.0);
			}
		}
	}
}

mod strategies {
	use crate::prelude::*;

	// #[reflect_trait]
	pub trait Strategy<ID: std::hash::Hash + Eq> {
		fn calculate(&self, blocks: HashMap<&ID, (&Thruster, &ForceAxis)>) -> HashMap<&ID, f32>;
	}

	#[test]
	fn assert_obj_safe() {
		#[derive(Debug, Reflect, Clone, Copy, PartialEq, Eq, Hash)]
		struct ID(u64);
		
		#[allow(dead_code)]
		fn assert_obj_safe(_: &dyn Strategy<ID>) {}
	}
}

mod visuals {}

pub mod examples {
	pub mod basic {
		use bevy_xpbd_3d::components::AsyncCollider;

		use crate::prelude::*;

		pub struct BasicThrusterBundle {
			pub pbr: PbrBundle,
			pub collider: AsyncCollider,
			pub name: Name,
			pub thruster: Thruster,
		}
	}
}
