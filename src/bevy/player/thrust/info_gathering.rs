use super::*;

impl ThrustType {
	/// Returns (positive, negative) keybinds
	pub const fn keybinds(self) -> (KeyCode, KeyCode) {
		match self {
			ThrustType::Forward => (KeyCode::R, KeyCode::F),
			ThrustType::Right => (KeyCode::H, KeyCode::S),
			ThrustType::Up => (KeyCode::Y, KeyCode::W),

			ThrustType::TurnRight => (KeyCode::G, KeyCode::D),
			ThrustType::TiltUp => (KeyCode::T, KeyCode::E),
			ThrustType::RollRight => (KeyCode::B, KeyCode::X),
		}
	}

	fn gather_flag(self, inputs: &Input<KeyCode>) -> Option<bool> {
		let (positive, negative) = self.keybinds();
		match (inputs.pressed(positive), inputs.pressed(negative)) {
			(true, true) | (false, false) => None,
			(true, false) => Some(true),
			(false, true) => Some(false),
		}
	}

	fn gather_flags(inputs: &Input<KeyCode>) -> Thrust<GenericInputFlags> {
		let mut ret = Thrust::default();

		for ttype in ThrustType::iter() {
			let flag = ttype.gather_flag(inputs);
			ret.set_from_type(ttype, flag);
		}

		ret
	}
}

/// [Option::None] when braking
pub(super) fn gather_input_flags(
	keyboard_input: Res<Input<KeyCode>>,
) -> Option<Thrust<GenericInputFlags>> {
	if keyboard_input.pressed(KeyCode::ShiftLeft) {
		None
	} else {
		Some(ThrustType::gather_flags(&keyboard_input))
	}
}

pub fn get_base_normal_vectors(
	player_transform: Query<&Transform, With<ControllablePlayer>>,
) -> Thrust<BasePositionNormalVectors> {
	let player = player_transform.single();

	let forward = player.forward();
	let up = player.up();

	// the meat of the system
	Thrust::<BasePositionNormalVectors> {
		forward,
		up,
		right: forward.cross(up),

		turn_right: -up,
		tilt_up: forward.cross(up),
		roll_right: forward,
		_stage: PhantomData,
	}
}

pub(super) fn max_velocity_magnitudes() -> Thrust<MaxAllowableVelocityMagnitudes> {
	impl ControllablePlayer {
		const MAX_LINEAR_VELOCITY: f32 = 10.;
		const MAX_ANGULAR_VELOCITY: f32 = 0.2;
	}

	Thrust::<MaxAllowableVelocityMagnitudes> {
		forward: 20.,
		up: ControllablePlayer::MAX_LINEAR_VELOCITY,
		right: ControllablePlayer::MAX_LINEAR_VELOCITY,

		tilt_up: ControllablePlayer::MAX_ANGULAR_VELOCITY * 2.,
		turn_right: ControllablePlayer::MAX_ANGULAR_VELOCITY,
		roll_right: ControllablePlayer::MAX_ANGULAR_VELOCITY,
		_stage: PhantomData,
	}
}

pub(super) const fn force_factors() -> Thrust<ForceFactors> {
	impl ControllablePlayer {
		const MOVE_FACTOR: f32 = 5_000_000.;
		const TURN_FACTOR: f32 = 5_000_000.;
	}

	Thrust::<ForceFactors> {
		forward: ControllablePlayer::MOVE_FACTOR,
		up: ControllablePlayer::MOVE_FACTOR,
		right: ControllablePlayer::MOVE_FACTOR,

		turn_right: ControllablePlayer::TURN_FACTOR,
		tilt_up: ControllablePlayer::TURN_FACTOR,
		roll_right: ControllablePlayer::TURN_FACTOR,
		_stage: PhantomData,
	}
}

// Should be ran after [PlayerMovement] system set
// pub fn get_current_final_vectors(player: Query<&MainPlayer>) -> Thrust<FinalVectors> {
// 	player.single().final_vectors.clone()
// }

// pub fn get_current_relative_strengths(player: Query<&ControllablePlayer>) -> Thrust<RelativeStrength> {
// 	player.single().relative_strength.clone()
// }

// pub fn get_current_af_flags(
// 	player: Query<&ControllablePlayer>,
// ) -> Thrust<ArtificialFrictionFlags> {
// 	player.single().artificial_friction_flags.clone()
// }
