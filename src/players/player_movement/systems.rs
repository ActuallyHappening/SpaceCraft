use super::{
	components::{ActualVelocity, IntendedVelocity, ThrusterAxis, ThrusterStrengths},
	input_processing::PlayerInputs,
	utils::ActualVelocityQuery,
	PlayerInput, PlayerMovementPlugin, Velocity6DimensionsMut,
};
use crate::{
	players::{thruster_block::Thruster, PlayerBlueprint},
	prelude::*,
};

impl PlayerMovementPlugin {
	/// Adds the [ThrusterAxis] component to players.
	pub(super) fn compute_thruster_axis(
		players: Query<
			(Entity, &Children, &PlayerBlueprint, &CenterOfMass),
			Or<(
				Changed<PlayerBlueprint>,
				Changed<CenterOfMass>,
				Changed<Children>,
			)>,
		>,
		thrusters: Query<(&Transform, &Thruster)>,
		mut commands: Commands,
	) {
		for (player, children, blueprint, center_of_mass) in players.iter() {
			let block_ids: HashSet<BlockId> = blueprint.derive_thruster_ids().collect();
			let mut thrusters: HashMap<BlockId, &Transform> = children
				.iter()
				.filter_map(|e| thrusters.get(*e).ok())
				.filter(|(_, thruster)| block_ids.contains(&thruster.get_block_id()))
				.map(|(t, id)| (id.get_block_id(), t))
				.collect();

			let thruster_axis = ThrusterAxis::new(center_of_mass, thrusters.drain());
			// MARK optimize by using &mut instead
			commands.entity(player).insert(thruster_axis);
		}
	}

	/// Adds the [IntendedVelocity] component to players.
	pub(super) fn calculate_intended_velocity(mut commands: Commands, player_inputs: PlayerInputs) {
		for (player, inputs) in player_inputs.iter() {
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

			// MARK optimize by using &mut instead
			commands.entity(player).insert(intended_velocity);
		}
	}

	/// Calculates the [ActualVelocity] component for each player
	pub(super) fn calculate_actual_velocity(
		players: Query<(Entity, ActualVelocityQuery), With<PlayerBlueprint>>,
		mut commands: Commands,
	) {
		for (player, actual) in players.iter() {
			let actual = actual.into_actual_velocity();
			commands.entity(player).insert(actual);
		}
	}

	pub(super) fn calculate_thruster_strengths(
		players: Query<
			(Entity, &ThrusterAxis, &IntendedVelocity, &ActualVelocity),
			With<PlayerBlueprint>,
		>,
		mut commands: Commands,
	) {
		for (player, axis, intended, actual) in players.iter() {
			let desired_delta = &intended.sub(*actual);
			let strengths = ThrusterStrengths::new(axis.get_blocks().map(|(id, force_axis)| {
				let mut dot = force_axis.dot(*desired_delta);
				const CUTOFF: f32 = 0.1;
				if dot.abs() < CUTOFF {
					dot = 0.;
				}
				(id, dot)
			}));
			commands.entity(player).insert(strengths);
			// debug!("Thruster strengths: {:?}", strengths);
		}
	}
}
