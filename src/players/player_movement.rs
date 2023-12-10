//! Responsible for chosing which thrusters go on,
//! and taking input from user.
//!
//! Is designed to be generic over where thrusters are placed
//! and their rotations, so that building your own ships is possible.

use crate::prelude::*;

pub use api::*;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
	fn build(&self, app: &mut App) {
		app
			.replicate::<components::IntendedVelocity>()
			.configure_sets(
				FixedUpdate,
				(
					PlayerMovementSet::ComputeStrengths,
					PlayerMovementSet::EnactThrusters,
				)
					.chain()
					.in_set(GlobalSystemSet::PlayerMovement),
			)
			.add_systems(
				FixedUpdate,
				(
					(
						Self::compute_thruster_axis,
						Self::calculate_intended_velocity,
						Self::calculate_actual_velocity,
					),
					Self::calculate_thruster_strengths,
				)
					.chain()
					.in_set(PlayerMovementSet::ComputeStrengths),
			)
			.add_plugins(InputManagerPlugin::<PlayerInput>::default())
			.register_type::<components::ThrusterAxis>()
			.register_type::<components::ThrusterStrengths>()
			.register_type::<components::IntendedVelocity>()
			.register_type::<components::ActualVelocity>();
	}
}

/// Public usage that is exported from this crate
mod api {
	use super::components::{ActualVelocity, IntendedVelocity, ThrusterAxis, ThrusterStrengths};
	use crate::prelude::*;

	pub use super::input_processing::PlayerInput;

	#[derive(SystemParam, Debug)]
	pub struct GetThrusterData<'w, 's> {
		players: Query<'w, 's, &'static ThrusterStrengths>,
	}

	#[derive(Bundle)]
	pub struct PlayerBundleMovementExt {
		input: InputManagerBundle<PlayerInput>,
		thruster_strengths: ThrusterStrengths,
		thruster_axis: ThrusterAxis,
		intended_velocity: IntendedVelocity,
		actual_velocity: ActualVelocity,
	}

	impl PlayerBundleMovementExt {
		pub fn new() -> Self {
			Self {
				input: PlayerInput::new(),
				thruster_strengths: Default::default(),
				thruster_axis: ThrusterAxis::default(),
				intended_velocity: IntendedVelocity::default(),
				actual_velocity: ActualVelocity::default(),
			}
		}
	}

	impl GetThrusterData<'_, '_> {
		/// Returns all of the thruster data for **EVERY* player
		pub fn get_all(&self) -> HashMap<&BlockId, &f32> {
			self
				.players
				.iter()
				.flat_map(ThrusterStrengths::get_blocks_strength)
				.collect()
		}
	}
}

mod input_processing;

mod systems;

mod components;

use utils::*;

use super::PlayerMovementSet;
mod utils;
