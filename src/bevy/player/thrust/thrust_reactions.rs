use super::*;

/// Takes into account braking and artificial friction
pub fn process_inputs(
	raw_inputs: Option<Thrust<GenericInputFlags>>,
	artificial_friction_flags: Thrust<ArtificialFrictionFlags>,

	current_velocity: Thrust<RelativeVelocityMagnitudes>,
) -> (Thrust<ThrustReactionsStage>, Thrust<ForceFactors>) {
	match raw_inputs {
		None => braking(current_velocity),
		Some(raw_inputs) => {
			let mut processed_inputs = Thrust::<ThrustReactionsStage>::default();
			let mut force_factors = force_factors();

			const ARTIFICIAL_FRICTION_FORCE_PENALTY: f32 = 0.1;

			for thrust_type in ThrustType::iter() {
				match (
					*artificial_friction_flags.get_from_type(thrust_type),
					raw_inputs.get_from_type(thrust_type),
				) {
					(true, None) => {
						*force_factors.get_mut_from_type(thrust_type) *= ARTIFICIAL_FRICTION_FORCE_PENALTY;
						processed_inputs.set_from_type(
							thrust_type,
							ThrustReactions::ArtificialFriction {
								friction_direction: counteract(*current_velocity.get_from_type(thrust_type)),
							},
						);
					}
					(true | false, input) => {
						processed_inputs.set_from_type(
							thrust_type,
							ThrustReactions::Normal { input: *input, },
						);
					}
				}
			}

			(processed_inputs, force_factors)
		}
	}
}

fn braking(
	current_velocity: Thrust<RelativeVelocityMagnitudes>,
) -> (Thrust<ThrustReactionsStage>, Thrust<ForceFactors>) {
	// breaking, must do opposite of current velocity to counteract / brake
	const BRAKING_FORCE_PENALTY: f32 = 0.25;

	let mut flagged_inputs = Thrust::<ThrustReactionsStage>::default();

	for thrust_type in ThrustType::iter() {
		let current = current_velocity.get_from_type(thrust_type);
		flagged_inputs.set_from_type(
			thrust_type,
			ThrustReactions::Braking {
				braking_direction: counteract(*current),
			},
		);
	}

	(flagged_inputs, force_factors() * BRAKING_FORCE_PENALTY)
}

fn counteract(current: f32) -> Option<bool> {
	const CUTOFF: f32 = 0.02;
	if current > CUTOFF {
		Some(false)
	} else if current < -CUTOFF {
		Some(true)
	} else {
		// already low enough to do nothing
		None
	}
}

/// For each dimension, how to decide what [GenericInputFlags] should be emitted?
/// Also used for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrustReactions {
	/// Enact the inputs received from the player as normal
	Normal { input: Option<bool> },
	/// The player has decided to apply artificial friction
	/// *AND* the player is not trying to move in that direction
	ArtificialFriction { friction_direction: Option<bool> },

	/// Slow down!
	Braking { braking_direction: Option<bool> },
}

impl Default for ThrustReactions {
	fn default() -> Self {
		ThrustReactions::Normal { input: None }
	}
}

impl ThrustReactions {
	fn into_generic_flag(self) -> Option<bool> {
		match self {
			ThrustReactions::Normal { input } => input,
			ThrustReactions::ArtificialFriction { friction_direction } => friction_direction,
			ThrustReactions::Braking { braking_direction } => braking_direction,
		}
	}
}

impl Thrust<ThrustReactionsStage> {
	pub fn into_generic_flags(self) -> Thrust<GenericInputFlags> {
		let mut generic_flags = Thrust::<GenericInputFlags>::default();

		for thrust_type in ThrustType::iter() {
			generic_flags.set_from_type(
				thrust_type,
				self.get_from_type(thrust_type).clone().into_generic_flag(),
			);
		}

		generic_flags
	}
}
