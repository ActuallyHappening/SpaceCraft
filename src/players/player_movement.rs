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
						Self::calculate_actual_velocity,
					),
					Self::calculate_thruster_strengths,
				)
					.chain()
					.in_set(PlayerMovementSet),
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

	impl ThrusterStrengths {
		pub(super) fn get_blocks_strength(&self) -> HashMap<&BlockId, &f32> {
			self
				.blocks
				.iter()
				.map(|(id, strength)| (id, strength))
				.collect()
		}

		pub(super) fn new(blocks: impl IntoIterator<Item = (BlockId, f32)>) -> Self {
			Self {
				blocks: blocks.into_iter().collect(),
			}
		}
	}

	/// Can maybe be cached after first computation,
	/// depending on whether player rebuild their ships.
	///
	/// Is not replicated, is derived data.
	#[derive(Debug, Component, Reflect)]
	pub(super) struct ThrusterAxis {
		blocks: HashMap<BlockId, ForceAxis>,
	}

	use force_axis::ForceAxis;

	use super::{Velocity6Dimensions, Velocity6DimensionsMut};
	mod force_axis;

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

		pub(super) fn get_blocks(&self) -> impl Iterator<Item = (BlockId, &ForceAxis)> {
			self.blocks.iter().map(|(id, force_axis)| (*id, force_axis))
		}
	}

	#[derive(Debug, Reflect, Component, Default, Serialize, Deserialize, Clone, Copy)]
	pub(super) struct IntendedVelocity {
		forward: f32,
		right: f32,
		up: f32,
		turn_right: f32,
		tilt_up: f32,
		roll_right: f32,
	}

	impl Velocity6Dimensions for IntendedVelocity {
		fn velocity_forward(&self) -> f32 {
			self.forward
		}

		fn velocity_rightward(&self) -> f32 {
			self.right
		}

		fn velocity_upward(&self) -> f32 {
			self.up
		}

		fn angular_turn_right(&self) -> f32 {
			self.turn_right
		}

		fn angular_tilt_up(&self) -> f32 {
			self.tilt_up
		}

		fn angular_roll_right(&self) -> f32 {
			self.roll_right
		}
	}
	impl Velocity6DimensionsMut for IntendedVelocity {
		fn forward_mut(&mut self) -> &mut f32 {
			&mut self.forward
		}

		fn right_mut(&mut self) -> &mut f32 {
			&mut self.right
		}

		fn up_mut(&mut self) -> &mut f32 {
			&mut self.up
		}

		fn turn_right_mut(&mut self) -> &mut f32 {
			&mut self.turn_right
		}

		fn tilt_up_mut(&mut self) -> &mut f32 {
			&mut self.tilt_up
		}

		fn roll_right_mut(&mut self) -> &mut f32 {
			&mut self.roll_right
		}
	}

	#[derive(Debug, Reflect, Component, Default, Clone, Copy)]
	pub(super) struct ActualVelocity {
		forward: f32,
		right: f32,
		up: f32,
		turn_right: f32,
		tilt_up: f32,
		roll_right: f32,
	}

	impl Velocity6Dimensions for ActualVelocity {
		fn velocity_forward(&self) -> f32 {
			self.forward
		}

		fn velocity_rightward(&self) -> f32 {
			self.right
		}

		fn velocity_upward(&self) -> f32 {
			self.up
		}

		fn angular_turn_right(&self) -> f32 {
			self.turn_right
		}

		fn angular_tilt_up(&self) -> f32 {
			self.tilt_up
		}

		fn angular_roll_right(&self) -> f32 {
			self.roll_right
		}
	}
	impl Velocity6DimensionsMut for ActualVelocity {
		fn forward_mut(&mut self) -> &mut f32 {
			&mut self.forward
		}

		fn right_mut(&mut self) -> &mut f32 {
			&mut self.right
		}

		fn up_mut(&mut self) -> &mut f32 {
			&mut self.up
		}

		fn turn_right_mut(&mut self) -> &mut f32 {
			&mut self.turn_right
		}

		fn tilt_up_mut(&mut self) -> &mut f32 {
			&mut self.tilt_up
		}

		fn roll_right_mut(&mut self) -> &mut f32 {
			&mut self.roll_right
		}
	}
}

use utils::*;
mod utils {
	use bevy::ecs::query::WorldQuery;

	use crate::prelude::*;

	use super::components::ActualVelocity;

	#[derive(WorldQuery)]
	pub(super) struct ActualVelocityQuery {
		lin: &'static LinearVelocity,
		ang: &'static AngularVelocity,
		rotation: &'static Transform,
	}

	impl<'w> ActualVelocityQueryItem<'w> {
		pub fn into_actual_velocity(self) -> ActualVelocity {
			let lin = self.rotation.rotation.mul_vec3(self.lin.0);
			let ang = self.ang.0;
			ActualVelocity::from_vec3(lin, ang)
		}
	}

	pub(super) trait Velocity6Dimensions: Default + std::fmt::Debug + Copy {
		fn linear_velocity(&self) -> Vec3 {
			Vec3::new(self.forward(), self.right(), self.up())
		}
		fn angular_velocity(&self) -> Vec3 {
			Vec3::new(-self.tilt_up(), self.turn_right(), self.roll_right())
		}

		fn velocity_forward(&self) -> f32;
		fn forward(&self) -> f32 {
			self.velocity_forward()
		}
		fn velocity_backward(&self) -> f32 {
			-self.velocity_forward()
		}
		fn velocity_back(&self) -> f32 {
			self.velocity_backward()
		}
		fn back(&self) -> f32 {
			self.velocity_backward()
		}

		fn velocity_rightward(&self) -> f32;
		fn velocity_right(&self) -> f32 {
			self.velocity_rightward()
		}
		/// Velocity
		fn right(&self) -> f32 {
			self.velocity_rightward()
		}
		fn velocity_leftward(&self) -> f32 {
			-self.velocity_rightward()
		}
		fn velocity_left(&self) -> f32 {
			self.velocity_leftward()
		}
		/// Velocity
		fn left(&self) -> f32 {
			self.velocity_leftward()
		}

		fn velocity_upward(&self) -> f32;
		fn velocity_up(&self) -> f32 {
			self.velocity_upward()
		}
		/// Velocity
		fn up(&self) -> f32 {
			self.velocity_upward()
		}
		fn velocity_downward(&self) -> f32 {
			-self.velocity_upward()
		}
		fn velocity_down(&self) -> f32 {
			self.velocity_downward()
		}
		/// Velocity
		fn down(&self) -> f32 {
			self.velocity_downward()
		}

		fn angular_turn_right(&self) -> f32;
		fn turn_right(&self) -> f32 {
			self.angular_turn_right()
		}
		fn angular_turn_left(&self) -> f32 {
			-self.angular_turn_right()
		}
		fn turn_left(&self) -> f32 {
			self.angular_turn_left()
		}

		fn angular_tilt_up(&self) -> f32;
		fn tilt_up(&self) -> f32 {
			self.angular_tilt_up()
		}
		fn angular_tilt_down(&self) -> f32 {
			-self.angular_tilt_up()
		}
		fn tilt_down(&self) -> f32 {
			self.angular_tilt_down()
		}

		fn angular_roll_right(&self) -> f32;
		fn roll_right(&self) -> f32 {
			self.angular_roll_right()
		}
		fn angular_roll_left(&self) -> f32 {
			-self.angular_roll_right()
		}
		fn roll_left(&self) -> f32 {
			self.angular_roll_left()
		}
	}

	pub(super) trait Velocity6DimensionsMut: Velocity6Dimensions {
		fn forward_mut(&mut self) -> &mut f32;
		fn right_mut(&mut self) -> &mut f32;
		fn up_mut(&mut self) -> &mut f32;
		fn turn_right_mut(&mut self) -> &mut f32;
		fn tilt_up_mut(&mut self) -> &mut f32;
		fn roll_right_mut(&mut self) -> &mut f32;

		fn from_vec3(lin: Vec3, ang: Vec3) -> Self {
			let mut ret = Self::default();
			*ret.forward_mut() = -lin.z;
			*ret.right_mut() = lin.x;
			*ret.up_mut() = lin.y;
			*ret.turn_right_mut() = ang.y;
			*ret.tilt_up_mut() = -ang.x;
			*ret.roll_right_mut() = ang.z;
			ret
		}

		/// Returns the factor by which the two velocities are similar.
		/// 1 => ang and/or lin perfectly match
		/// 0 => no match
		/// negative is opposite
		fn dot(self, rhs: impl Velocity6DimensionsMut) -> f32 {
			let lhs = self.try_normalize();
			let other = rhs.try_normalize();

			match (lhs, other) {
				(Some(lhs), Some(rhs)) => {
					lhs.forward() * rhs.forward()
						+ lhs.right() * rhs.right()
						+ lhs.up() * rhs.up()
						+ lhs.turn_right() * rhs.turn_right()
						+ lhs.tilt_up() * rhs.tilt_up()
						+ lhs.roll_right() * rhs.roll_right()
				}
				_ => 0.,
			}
		}

		fn sub(&self, rhs: impl Velocity6DimensionsMut) -> Self {
			let mut ret = Self::default();
			*ret.forward_mut() = self.forward() - rhs.forward();
			*ret.right_mut() = self.right() - rhs.right();
			*ret.up_mut() = self.up() - rhs.up();
			*ret.turn_right_mut() = self.turn_right() - rhs.turn_right();
			*ret.tilt_up_mut() = self.tilt_up() - rhs.tilt_up();
			*ret.roll_right_mut() = self.roll_right() - rhs.roll_right();
			ret
		}

		fn try_normalize(self) -> Option<Self> {
			let mut sum = self.forward().powi(2)
				+ self.right().powi(2)
				+ self.up().powi(2)
				+ self.turn_right().powi(2)
				+ self.tilt_up().powi(2)
				+ self.roll_right().powi(2);
			if sum == 0. {
				return None;
			}
			sum = sum.sqrt();
			Some(Self::from_vec3(
				Vec3::new(self.forward() / sum, self.right() / sum, self.up() / sum),
				Vec3::new(
					self.turn_right() / sum,
					self.tilt_up() / sum,
					self.roll_right() / sum,
				),
			))
		}

		fn normalize_or_zero(&mut self) {
			let mut sum = self.forward().powi(2)
				+ self.right().powi(2)
				+ self.up().powi(2)
				+ self.turn_right().powi(2)
				+ self.tilt_up().powi(2)
				+ self.roll_right().powi(2);
			if sum == 0. {
				return;
			}
			sum = sum.sqrt();
			*self.forward_mut() /= sum;
			*self.right_mut() /= sum;
			*self.up_mut() /= sum;
			*self.turn_right_mut() /= sum;
			*self.tilt_up_mut() /= sum;
			*self.roll_right_mut() /= sum;
		}

		/// Velocity
		fn add_forward(&mut self, amount: f32) {
			*self.forward_mut() += amount;
		}
		/// Velocity
		fn add_backward(&mut self, amount: f32) {
			self.add_forward(-amount);
		}

		/// Velocity
		fn add_rightward(&mut self, amount: f32) {
			*self.right_mut() += amount;
		}
		/// Velocity
		fn add_right(&mut self, amount: f32) {
			self.add_rightward(amount);
		}
		/// Velocity
		fn add_leftward(&mut self, amount: f32) {
			self.add_rightward(-amount);
		}
		/// Velocity
		fn add_left(&mut self, amount: f32) {
			self.add_leftward(amount);
		}

		fn add_turn_right(&mut self, amount: f32) {
			*self.turn_right_mut() += amount;
		}
		fn add_turn_left(&mut self, amount: f32) {
			self.add_turn_right(-amount);
		}

		fn add_tilt_up(&mut self, amount: f32) {
			*self.tilt_up_mut() += amount;
		}
		fn add_tilt_down(&mut self, amount: f32) {
			self.add_tilt_up(-amount);
		}

		fn add_roll_right(&mut self, amount: f32) {
			*self.roll_right_mut() += amount;
		}
		fn add_roll_left(&mut self, amount: f32) {
			self.add_roll_right(-amount);
		}
	}
}
