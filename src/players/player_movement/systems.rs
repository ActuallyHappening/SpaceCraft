use super::{
	components::{ActualVelocity, IntendedVelocity, ThrusterAxis, ThrusterStrengths},
	utils::ActualVelocityQuery,
	PlayerInput, PlayerMovementPlugin, Velocity6DimensionsMut,
};
use crate::{
	players::{player::PlayerBlueprintComponent, thruster_block::Thruster},
	prelude::*,
};

impl PlayerMovementPlugin {
	/// Adds the [ThrusterAxis] component to players.
	pub(super) fn compute_thruster_axis(
		mut players: Query<
			(
				&mut ThrusterAxis,
				&Children,
				&PlayerBlueprintComponent,
				&CenterOfMass,
			),
			Or<(
				Changed<PlayerBlueprintComponent>,
				Changed<CenterOfMass>,
				Changed<Children>,
			)>,
		>,
		thrusters: Query<(&Transform, &Thruster)>,
	) {
		for (mut player, children, blueprint, center_of_mass) in players.iter_mut() {
			let block_ids: HashSet<BlockId> = blueprint.derive_thruster_ids().collect();
			let mut thrusters: HashMap<BlockId, &Transform> = children
				.iter()
				.filter_map(|e| thrusters.get(*e).ok())
				.filter(|(_, thruster)| block_ids.contains(&thruster.get_block_id()))
				.map(|(t, id)| (id.get_block_id(), t))
				.collect();

			let thruster_axis = ThrusterAxis::new(center_of_mass, thrusters.drain());
			*player = thruster_axis;
		}
	}

	/// Adds the [IntendedVelocity] component to players.
	pub(super) fn calculate_intended_velocity(
		mut players: Query<(&mut IntendedVelocity, &ActionState<PlayerInput>)>,
	) {
		for (mut player, inputs) in players.iter_mut() {
			let mut intended_velocity = IntendedVelocity::default();

			if inputs.pressed(PlayerInput::Forward) {
				intended_velocity.add_forward(PlayerInput::FORCE_FACTOR);
			}
			if inputs.pressed(PlayerInput::Backward) {
				intended_velocity.add_backward(PlayerInput::FORCE_FACTOR);
			}

			if inputs.pressed(PlayerInput::Left) {
				intended_velocity.add_turn_left(PlayerInput::ROTATION_FACTOR);
			}
			if inputs.pressed(PlayerInput::Right) {
				intended_velocity.add_turn_right(PlayerInput::ROTATION_FACTOR);
			}

			*player = intended_velocity;
		}
	}

	/// Calculates the [ActualVelocity] component for each player
	pub(super) fn calculate_actual_velocity(
		mut players: Query<(&mut ActualVelocity, ActualVelocityQuery), With<PlayerBlueprintComponent>>,
	) {
		for (mut player, actual) in players.iter_mut() {
			let actual = actual.into_actual_velocity();
			*player = actual;
		}
	}

	/// Calculates [ThrusterStrengths] from [ThrusterAxis],
	/// [IntendedVelocity], and [ActualVelocity].
	// #[bevycheck::system]
	pub(super) fn calculate_thruster_strengths(
		mut players: Query<
			(
				&mut ThrusterStrengths,
				&ThrusterAxis,
				&IntendedVelocity,
				&ActualVelocity,
			),
			With<PlayerBlueprintComponent>,
		>,
	) {
		for (mut player, axis, intended, actual) in players.iter_mut() {
			let desired_delta = &intended.sub(*actual);
			let strengths = ThrusterStrengths::new(axis.get_blocks().map(|(id, force_axis)| {
				let mut dot = force_axis.dot(*desired_delta);
				const CUTOFF: f32 = 0.1;
				if dot.abs() < CUTOFF {
					dot = 0.;
				}
				(id, dot)
			}));
			*player = strengths;
		}
	}
}
