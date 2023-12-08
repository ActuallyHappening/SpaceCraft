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
			.add_systems(
				FixedUpdate,
				(
					(
						Self::compute_thruster_axis,
						Self::calculate_intended_velocity,
					),
					Self::chose_thrusters,
				)
					.chain()
					.in_set(PlayerMovementSet),
			)
			.add_plugins(InputManagerPlugin::<PlayerInput>::default())
			.register_type::<components::ThrusterAxis>()
			.register_type::<components::ThrusterStrengths>()
			.register_type::<components::IntendedVelocity>();
	}
}

/// Public usage that is exported from this crate
mod api {
	use super::components::ThrusterStrengths;
	use crate::prelude::*;

	pub use super::input_processing::PlayerInput;

	#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
	pub struct PlayerMovementSet;

	#[derive(SystemParam)]
	pub struct GetThrusterData<'w, 's> {
		players: Query<'w, 's, &'static ThrusterStrengths>,
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

mod input_processing {
	use crate::{players::player::ControllablePlayer, prelude::*};

	// pub struct InputProcessingPlugin;

	// impl Plugin for InputProcessingPlugin {
	// 	fn build(&self, app: &mut App) {
	// 		app
	// 			.add_systems(PreUpdate, process_action_diffs::<PlayerInput, Key>)
	// 			.add_systems(
	// 				PostUpdate,
	// 				generate_action_diffs::<PlayerInput, Key>.run_if(NetcodeConfig::not_headless()),
	// 			)
	// 			// .register_type::<Key>()
	// 			.add_client_event::<ActionDiff<PlayerInput, Key>>(SendType::Unreliable);
	// 	}
	// }

	#[derive(
		ActionLike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, Serialize, Deserialize,
	)]
	pub enum PlayerInput {
		Forward,
		Backward,
		Left,
		Right,
	}

	impl PlayerInput {
		pub const FORCE_FACTOR: f32 = 2.;
		pub const ROTATION_FACTOR: f32 = 2.;
	}

	#[derive(SystemParam, Debug)]
	pub struct PlayerInputs<'w, 's> {
		query: Query<'w, 's, (Entity, &'static ActionState<PlayerInput>)>,
	}

	impl PlayerInputs<'_, '_> {
		// pub fn get_from_id(&self, player_id: ClientId) -> Option<&ActionState<PlayerInput>> {
		// 	self
		// 		.query
		// 		.iter()
		// 		.find(|(_, player)| player.get_network_id() == player_id)
		// 		.map(|(action_state, _)| action_state)
		// }

		// pub fn get(&self, e: Entity) -> Option<&ActionState<PlayerInput>> {
		// 	self.query.get(e).ok().map(|(action_state, _)| action_state)
		// }

		pub fn iter(&self) -> impl Iterator<Item = (Entity, &ActionState<PlayerInput>)> {
			self.query.iter()
		}
	}

	impl PlayerInput {
		pub fn new() -> InputManagerBundle<Self> {
			InputManagerBundle {
				action_state: ActionState::default(),
				input_map: InputMap::new([
					(KeyCode::W, PlayerInput::Forward),
					(KeyCode::S, PlayerInput::Backward),
					(KeyCode::A, PlayerInput::Left),
					(KeyCode::D, PlayerInput::Right),
				]),
			}
		}
	}
}

mod systems {
	use super::{
		components::{IntendedVelocity, ThrusterAxis},
		input_processing::PlayerInputs,
		PlayerInput, PlayerMovementPlugin,
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
				commands.entity(player).insert(thruster_axis);
			}
		}

		/// Adds the [IntendedVelocity] component to players.
		pub(super) fn calculate_intended_velocity(mut commands: Commands, player_inputs: PlayerInputs) {
			for (player, inputs) in player_inputs.iter() {
				let mut intended_velocity = IntendedVelocity {
					linear_velocity: Vec3::ZERO,
					angular_velocity: Vec3::ZERO,
				};

				if inputs.pressed(PlayerInput::Forward) {
					intended_velocity.linear_velocity += Vec3::Z;
				}
				if inputs.pressed(PlayerInput::Backward) {
					intended_velocity.linear_velocity -= Vec3::Z;
				}
				if inputs.pressed(PlayerInput::Left) {
					intended_velocity.angular_velocity -= Vec3::X;
				}
				if inputs.pressed(PlayerInput::Right) {
					intended_velocity.angular_velocity += Vec3::X;
				}

				intended_velocity.angular_velocity *= PlayerInput::ROTATION_FACTOR;
				intended_velocity.linear_velocity *= PlayerInput::FORCE_FACTOR;

				commands.entity(player).insert(intended_velocity);
			}
		}

		pub(super) fn chose_thrusters() {}
	}
}

mod components {
	use crate::prelude::*;

	/// Stores all of the data concerning thruster movements.
	/// Placed on players.
	///
	/// # Is replicated
	#[derive(Component, Debug, Reflect, Serialize, Deserialize)]
	pub(super) struct ThrusterStrengths {
		blocks: HashMap<BlockId, f32>,
	}

	/// Can maybe be cached after first computation,
	/// depending on whether player rebuild their ships.
	///
	/// Is not replicated, is derived data.
	#[derive(Debug, Component, Reflect)]
	pub(super) struct ThrusterAxis {
		blocks: HashMap<BlockId, ForceAxis>,
	}

	#[derive(Debug, Reflect, Component, Serialize, Deserialize)]
	pub(super) struct IntendedVelocity {
		pub(super) linear_velocity: Vec3,
		pub(super) angular_velocity: Vec3,
	}

	use force_axis::ForceAxis;
	mod force_axis;

	impl ThrusterStrengths {
		pub(super) fn get_blocks_strength(&self) -> HashMap<&BlockId, &f32> {
			self
				.blocks
				.iter()
				.map(|(id, strength)| (id, strength))
				.collect()
		}
	}

	impl ThrusterAxis {
		pub(super) fn new<'w>(
			center_of_mass: &'w CenterOfMass,
			blocks: impl IntoIterator<Item = (BlockId, &'w Transform)>,
		) -> Self {
			Self {
				blocks: blocks
					.into_iter()
					.map(|(id, t)| (id, ForceAxis::new(t, center_of_mass)))
					.collect(),
			}
		}
	}
}
