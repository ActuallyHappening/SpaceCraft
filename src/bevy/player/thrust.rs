//! Thrust is processed in various ways.
//!
//! Braking:
//! - Tries to stop player in space across all dimensions
//! - Ignores all other player inputs while braked

use crate::utils::*;

use super::ControllablePlayer;

mod helpers;
use helpers::*;

mod stages;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
pub use stages::*;

mod info_gathering;
pub use info_gathering::*;
mod info_processors;
pub use info_processors::*;
mod info_enactors;
pub use info_enactors::*;
mod thrust_reactions;
pub use thrust_reactions::*;

pub mod types;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thrust<S: ThrustStage> {
	/// Positive is forward obviously
	pub forward: <S as self::ThrustStage>::DimensionType,

	pub up: <S as self::ThrustStage>::DimensionType,

	pub right: <S as self::ThrustStage>::DimensionType,

	pub turn_right: <S as self::ThrustStage>::DimensionType,

	/// Upwards is positive
	pub tilt_up: <S as self::ThrustStage>::DimensionType,

	/// Right is positive
	pub roll_right: <S as self::ThrustStage>::DimensionType,

	_stage: PhantomData<S>,
}

pub trait ThrustStage {
	type DimensionType: std::fmt::Debug + Clone + Serialize + DeserializeOwned;
}

impl<D, T> Default for Thrust<D>
where
	D: ThrustStage<DimensionType = T> + std::default::Default,
	D::DimensionType: Default,
{
	fn default() -> Self {
		Thrust::<D> {
			forward: T::default(),
			up: T::default(),
			right: T::default(),

			turn_right: T::default(),
			tilt_up: T::default(),
			roll_right: T::default(),
			_stage: PhantomData,
		}
	}
}

/// Combines the normal and relative thrusts into the final thrust vectors,
/// and saves the necessary information to various places including in the [MainPlayer] component
#[allow(clippy::type_complexity)]
pub fn save_thrust_stages(
	In((relative_strength, normal_vectors, max, thrust_responses,)): In<(
		Thrust<RelativeStrength>,
		Thrust<BasePositionNormalVectors>,
		Thrust<ForceFactors>,
		Thrust<ThrustReactionsStage>,
	)>,
	mut player_data: Query<&mut ControllablePlayer, With<ControllablePlayer>>,
) -> Thrust<FinalVectors> {
	let final_vectors = normal_vectors * relative_strength.clone() * max;

	let mut player = player_data.single_mut();

	// todo! Save thrust stages here!
	// player.relative_strength = relative_strength;
	// player.thrust_responses = thrust_responses;

	final_vectors
}

pub fn manually_threading_player_movement(
	In((current_velocity, artificial_friction_flags)): In<(
		Thrust<RelativeVelocityMagnitudes>,
		Thrust<ArtificialFrictionFlags>,
	)>,
	keyboard_input: Res<Input<KeyCode>>,
	player_transform: Query<&Transform, With<ControllablePlayer>>,
	player_velocity: Query<&Velocity, With<ControllablePlayer>>,
	player_data: Query<&mut ControllablePlayer, With<ControllablePlayer>>,
	time: Res<Time>,
	player_physics: Query<&mut ExternalForce, With<ControllablePlayer>>,
) {
	let base_normal = get_base_normal_vectors(player_transform);

	let raw_inputs = gather_input_flags(keyboard_input);
	let (input_flags, force_factors) = process_inputs(raw_inputs, artificial_friction_flags, current_velocity);

	let flagged_inputs = input_flags.clone().into_generic_flags() * base_normal.clone();
	let relative_strengths = get_relative_strengths(
		In((flagged_inputs, max_velocity_magnitudes())),
		player_velocity,
	);
	let final_vectors = save_thrust_stages(
		In((
			relative_strengths,
			base_normal,
			force_factors,
			input_flags,
		)),
		player_data,
	);

	apply_thrust(In(final_vectors), player_physics, time);
}

// #[bevycheck::system]
// pub fn manual_get_final_thrust(
// 	keyboard_input: Res<Input<KeyCode>>,
// 	player_pos: Query<&Transform, With<MainPlayer>>,
// 	player_current: Query<(&Velocity, &Transform), With<MainPlayer>>,
// 	player_save: Query<&mut MainPlayer, With<MainPlayer>>,
// ) -> Thrust<FinalVectors> {
// 	let normals = flag_normal_vectors(In(gather_player_movement(keyboard_input)), player_pos);
// 	let relative = get_relative_strengths(
// 		In((get_max_velocity_magnitudes(), normals.clone())),
// 		player_current,
// 	);

// 	save_thrust_stages(In((normals, relative, get_force_factors())), player_save)
// }
