#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

pub mod prelude {
	pub use crate::plugins::{ThrusterPlugin, ThrusterSystemSet};
	pub use crate::shared_types::{
		components::{CurrentVelocity, IntendedVelocity, Thruster},
		ForceAxis, Relative6DVector, Vec6,
	};

	pub use bevy_xpbd_3d_parenting::prelude::*;

	pub(crate) use crate::impl_relative_6d_vector;
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
	/// Require the [ParentingPlugin] to be initialized already.
	#[derive(Debug)]
	pub struct ThrusterPlugin {
		schedule: InternedScheduleLabel,
	}

	impl ThrusterPlugin {
		/// Must be the same [Schedule] as [ParentingPlugin], see [bevy_xpbd_3d_parenting] docs
		/// for [ParentingPlugin].
		pub fn new(schedule: impl ScheduleLabel) -> Self {
			Self {
				schedule: schedule.intern(),
			}
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

	pub trait Relative6DVector {
		fn get_forward(&self) -> f32;
		fn get_right(&self) -> f32;
		fn get_upwards(&self) -> f32;
		fn get_turn_right(&self) -> f32;
		fn get_pitch_up(&self) -> f32;
		fn get_roll_right(&self) -> f32;

		fn get_generic(&self) -> Vec6 {
			Vec6 {
				forward: self.get_forward(),
				right: self.get_right(),
				upwards: self.get_upwards(),
				turn_right: self.get_turn_right(),
				pitch_up: self.get_pitch_up(),
				roll_right: self.get_roll_right(),
			}
		}

		fn dot(&self, rhs: &impl Relative6DVector) -> f32 {
			let lhs = self.get_generic();
			let rhs = rhs.get_generic();

			lhs.forward * rhs.forward
				+ lhs.right * rhs.right
				+ lhs.upwards * rhs.upwards
				+ lhs.turn_right * rhs.turn_right
				+ lhs.pitch_up * rhs.pitch_up
				+ lhs.roll_right * rhs.roll_right
		}
	}

	#[macro_export]
	macro_rules! impl_relative_6d_vector {
		// impl Relative6DVector for $type with forward,right,upwards etc fields
		($type:ty) => {
			impl $crate::shared_types::Relative6DVector for $type {
				fn get_forward(&self) -> f32 {
					self.forward
				}

				fn get_right(&self) -> f32 {
					self.right
				}

				fn get_upwards(&self) -> f32 {
					self.upwards
				}

				fn get_turn_right(&self) -> f32 {
					self.turn_right
				}

				fn get_pitch_up(&self) -> f32 {
					self.pitch_up
				}

				fn get_roll_right(&self) -> f32 {
					self.roll_right
				}
			}
		};
	}

	/// Forces are between [-1, 1],
	/// Torques are 'normalized' to [-1, 1] // TODO
	///
	/// The Greek philosopher, Archimedes, said,
	/// “Give me a lever long enough and a fulcrum on which to place it, and I shall move the world.”
	#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
	#[reflect(Component)]
	pub struct ForceAxis {
		forward: f32,
		right: f32,
		upwards: f32,
		turn_right: f32,
		pitch_up: f32,
		roll_right: f32,
	}
	impl_relative_6d_vector!(ForceAxis);

	#[derive(Debug, Reflect, derive_more::Add, derive_more::Sub)]
	pub struct Vec6 {
		pub forward: f32,
		pub right: f32,
		pub upwards: f32,
		pub turn_right: f32,
		pub pitch_up: f32,
		pub roll_right: f32,
	}
	impl_relative_6d_vector!(Vec6);

	impl std::ops::Index<usize> for Vec6 {
		type Output = f32;

		fn index(&self, index: usize) -> &Self::Output {
			match index {
				0 => &self.forward,
				1 => &self.right,
				2 => &self.upwards,
				3 => &self.turn_right,
				4 => &self.pitch_up,
				5 => &self.roll_right,
				_ => panic!("Index out of bounds"),
			}
		}
	}

	pub struct Vec6Iterator {
		this: Vec6,
		index: usize,
	}

	impl IntoIterator for Vec6 {
		type IntoIter = Vec6Iterator;
		type Item = f32;

		fn into_iter(self) -> Self::IntoIter {
			Vec6Iterator {
				this: self,
				index: 0,
			}
		}
	}

	impl Iterator for Vec6Iterator {
		type Item = f32;

		fn next(&mut self) -> Option<Self::Item> {
			if self.index >= 6 {
				return None;
			}

			let result = self.this[self.index];
			self.index += 1;
			Some(result)
		}
	}

	pub(crate) mod components {
		use crate::prelude::*;

		/// Component for all thrusters
		#[derive(Debug, Component, Reflect, InspectorOptions)]
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

			pub fn set_strength_factor(&mut self, strength_factor: f32) -> &mut Self {
				#[cfg(feature = "debug")]
				if strength_factor < 0.0 {
					warn!("Strength factor {} must be >= 0.0", strength_factor);
				}
				self.strength_factor = strength_factor.max(0.0);
				self
			}

			pub fn get_current_status(&self) -> f32 {
				self.current_status.clamp(0.0, 1.0)
			}

			pub fn set_current_status(&mut self, current_status: f32) -> &mut Self {
				#[cfg(feature = "debug")]
				if !(0.0..=1.0).contains(&current_status) {
					warn!(
						"Current status {} must be between 0.0 and 1.0 (inclusive)",
						current_status
					);
				}
				self.current_status = current_status.clamp(0.0, 1.0);
				self
			}
		}

		#[derive(Component, Debug, Default, Reflect)]
		#[reflect(Component)]
		pub struct CurrentVelocity {
			forward: f32,
			right: f32,
			upwards: f32,
			turn_right: f32,
			pitch_up: f32,
			roll_right: f32,
		}

		impl_relative_6d_vector!(CurrentVelocity);

		#[derive(Component, Debug, Default, Reflect)]
		#[reflect(Component)]
		pub struct IntendedVelocity {
			forward: f32,
			right: f32,
			upwards: f32,
			turn_right: f32,
			pitch_up: f32,
			roll_right: f32,
		}

		impl_relative_6d_vector!(IntendedVelocity);
	}

	// mod bundles {
	// 	use super::components;
	// 	use crate::prelude::*;

	// 	// /// The bare minimum bundle for a functional thruster
	// 	// #[derive(Bundle, Debug)]
	// 	// pub struct CrudeThrusterBundle {
	// 	// 	pub thruster: components::Thruster,
	// 	// 	pub transform: TransformBundle,

	// 	// }
	// }
}

mod strategies {
	use bevy::ecs::query::WorldQuery;

	use crate::prelude::*;

	#[derive(Debug, Reflect, WorldQuery)]
	pub struct ThrusterInfo<'w> {
		pub thruster: &'w Thruster,
		pub force_axis: &'w ForceAxis,
	}

	#[derive(Debug, Reflect, WorldQuery)]
	pub struct ParentInfo<'w> {
		pub current_velocity: &'w CurrentVelocity,
		pub intended_velocity: &'w IntendedVelocity,
	}

	impl ParentInfo<'_> {
		pub fn difference(&self) -> impl Relative6DVector + std::fmt::Debug + Reflect {
			self.intended_velocity.get_generic() - self.current_velocity.get_generic()
		}
	}

	/// A (pure) strategy for calculating the [Thruster]s' strengths.
	/// Has no side effects, or dependence on [World].
	// #[reflect_trait]
	pub trait PureStrategy<ID: std::hash::Hash + Eq> {
		fn calculate<'w>(
			&self,
			blocks: HashMap<&'w ID, ThrusterInfo<'w>>,
			parent: ParentInfo<'w>,
		) -> HashMap<&'w ID, f32>;
	}

	#[test]
	fn assert_obj_safe() {
		#[derive(Debug, Reflect, Clone, PartialEq, Eq, Hash)]
		struct ID(u64);

		#[allow(dead_code)]
		fn assert_obj_safe(_: &dyn PureStrategy<ID>) {}
	}

	pub struct ExactAxisStrategy;

	impl<ID: std::hash::Hash + Eq> PureStrategy<ID> for ExactAxisStrategy {
		fn calculate<'w>(
			&self,
			mut blocks: HashMap<&'w ID, ThrusterInfo<'w>>,
			parent: ParentInfo<'w>,
		) -> HashMap<&'w ID, f32> {
			let mut result = HashMap::with_capacity(blocks.len());
			let aim = parent.difference();

			for (id, info) in blocks.drain() {
				result.insert(id, info.force_axis.dot(&aim));
			}

			result
		}
	}

	#[cfg(test)]
	mod tests {
		#[test]
		fn exact_axis_strategy_works() {
			// TODO
			todo!()
		}
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
