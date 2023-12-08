//! Responsible for chosing which thrusters go on,
//! and taking input from user.
//!
//! Is designed to be generic over where thrusters are placed
//! and their rotations, so that building your own ships is possible.

use crate::prelude::*;

pub use api::*;

use self::components::ThrusterStrengths;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
	fn build(&self, app: &mut App) {
		app
			.replicate::<ThrusterStrengths>()
			.add_systems(
				FixedUpdate,
				(Self::compute_thruster_axis, Self::chose_thrusters)
					.chain()
					.in_set(PlayerMovementSet),
			)
			.register_type::<components::ThrusterAxis>()
			.register_type::<components::ThrusterStrengths>();
	}
}

/// Public usage that is exported from this crate
mod api {
	use super::components::ThrusterStrengths;
	use crate::prelude::*;

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

mod systems {
	use super::{components::ThrusterAxis, PlayerMovementPlugin};
	use crate::{
		players::{thruster_block::Thruster, PlayerBlueprint},
		prelude::*,
	};

	impl PlayerMovementPlugin {
		/// Triggers whenever the children of a player are change.
		/// This makes sure the axis are only computed whenever a new thruster / block is added
		pub(super) fn compute_thruster_axis(
			players: Query<
				(Entity, &Children, &PlayerBlueprint, &CenterOfMass),
				Or<(Changed<PlayerBlueprint>, Changed<CenterOfMass>)>,
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

	/// Forces are between [0, 1],
	/// but torque can be infinite so is either 0 or 1
	///
	/// The Greek philosopher, Archimedes, said,
	/// “Give me a lever long enough and a fulcrum on which to place it, and I shall move the world.”
	#[derive(Debug, Reflect)]
	pub(super) struct ForceAxis {
		forward: f32,
		right: f32,
		upwards: f32,
		turn_right: SignedFlag,
		pitch_up: SignedFlag,
		roll_right: SignedFlag,
	}

	#[derive(Debug, Reflect, Default)]
	pub(super) enum SignedFlag {
		Flag(bool),
		#[default]
		Zero,
	}

	impl SignedFlag {
		pub fn new(num: f32) -> Self {
			const EPSILON: f32 = 0.01;
			if num.abs() < EPSILON {
				Self::Zero
			} else {
				Self::Flag(num > 0.0)
			}
		}

		pub fn into_option(self) -> Option<bool> {
			match self {
				Self::Flag(b) => Some(b),
				Self::Zero => None,
			}
		}

		/// Whether [SignedFlag::Zero] or not
		pub fn flagged(self) -> bool {
			match self {
				Self::Flag(_) => true,
				Self::Zero => false,
			}
		}

		pub fn flagged_true(self) -> bool {
			match self {
				Self::Flag(b) => b,
				Self::Zero => false,
			}
		}

		pub fn flagged_false(self) -> bool {
			match self {
				Self::Flag(b) => !b,
				Self::Zero => false,
			}
		}
	}

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

	/// Returns [-1, 1] for factor that rotation is in the direction
	/// of base
	pub fn factor_direction_in(rotation: impl Into<Quat>, base: impl Into<Quat>) -> f32 {
		let rotation: Quat = rotation.into();
		let base = base.into();

		let angle = rotation.angle_between(base);

		// 0.0 -> 1.0
		// TAU / 4 -> 0.0
		// TAU / 2 -> -1.0
		let ret: f32 = angle.cos();

		#[cfg(test)]
		println!(
			"Angle between: {:?} & {:?} = {} (ret = {})",
			rotation, base, angle, ret
		);

		ret
	}

	impl ForceAxis {
		pub fn from_iter(
			mut forces: impl FnMut(Vec3) -> f32,
			mut torques: impl FnMut(Vec3) -> SignedFlag,
		) -> Self {
			Self {
				forward: forces(-Vec3::Z),
				right: forces(Vec3::X),
				upwards: forces(Vec3::Y),
				turn_right: torques(-Vec3::Y),
				pitch_up: torques(Vec3::X),
				roll_right: torques(Vec3::Z),
			}
		}

		/// Takes the transform of a thruster, including its relative translation and rotation,
		/// and the center of mass of the player, and computes what effect in
		/// each of the 3 force and 3 torque axis it would have on the player
		pub(super) fn new(
			Transform {
				translation,
				rotation,
				..
			}: &Transform,
			center_of_mass: &CenterOfMass,
		) -> Self {
			let relative_force = rotation.mul_vec3(Vec3::Z);

			#[cfg(test)]
			println!(
				"Relative force: {:?}, translation: {:?}",
				relative_force, translation
			);

			let ef = *ExternalForce::new(Vec3::ZERO).apply_force_at_point(
				relative_force,
				*translation,
				center_of_mass.0,
			);
			let force = ef.force();
			let forces = |dir: Vec3| force.dot(dir) / force.length();

			let torque = ef.torque();
			let torques = |dir: Vec3| SignedFlag::new(torque.dot(dir));

			Self::from_iter(forces, torques)
		}
	}

	#[cfg(test)]
	mod test {
		use crate::blocks::manual_builder::{Facing, RelativePixel};
		use crate::prelude::*;

		use super::{factor_direction_in, ForceAxis};

		#[test]
		fn force_axis() {
			assert_vec3_near!(Facing::Forwards.into_quat().mul_vec3(Vec3::Z), Vec3::Z);
			assert_vec3_near!(Facing::Right.into_quat().mul_vec3(Vec3::Z), -Vec3::X);

			// thruster in back facing right,
			// turning ship rightwards
			let thruster_location = Transform {
				translation: RelativePixel::new(0, 0, 1).into_world_offset(),
				rotation: Facing::Right.into_quat(),
				..default()
			};
			let force_axis = ForceAxis::new(&thruster_location, &CenterOfMass(Vec3::ZERO));
			println!("Force axis {:?}", force_axis);

			assert!(force_axis.turn_right.flagged_true());
			assert!(!force_axis.pitch_up.flagged());
			assert!(!force_axis.roll_right.flagged());
			assert!(force_axis.right < 0.0);
			assert!(force_axis.upwards == 0.0);
			assert!(force_axis.forward == 0.0);
		}

		#[test]
		fn apply_force_at_point() {
			// force rightwards at back of ship
			let ef = *ExternalForce::new(Vec3::ZERO).apply_force_at_point(-Vec3::X, Vec3::Z, Vec3::ZERO);
			println!("EF: {:?}", ef);
			assert_vec3_near!(ef.torque(), Vec3::new(0.0, -1.0, 0.0));

			// force downward at back of ship
			let ef = *ExternalForce::new(Vec3::ZERO).apply_force_at_point(-Vec3::Y, Vec3::Z, Vec3::ZERO);
			println!("EF: {:?}", ef);
			assert_vec3_near!(ef.torque(), Vec3::new(1.0, 0.0, 0.0));

			// force upwards at right of ship
			let ef = *ExternalForce::new(Vec3::ZERO).apply_force_at_point(Vec3::Y, Vec3::X, Vec3::ZERO);
			println!("EF: {:?}", ef);
			assert_vec3_near!(ef.torque(), Vec3::new(0.0, 0.0, 1.0));
		}

		#[test]
		fn basic_rotations() {
			assert_near!(
				Facing::Right
					.into_quat()
					.angle_between(Facing::Right.into_quat()),
				0.0
			);
			assert_near!(
				Facing::Right
					.into_quat()
					.angle_between(Facing::Forwards.into_quat()),
				TAU / 4.
			);

			assert_near!(factor_direction_in(Facing::Right, Facing::Forwards), 0.0);
			assert_near!(factor_direction_in(Facing::Right, Facing::Right), 1.0);
			assert_near!(factor_direction_in(Facing::Right, Facing::Left), -1.0);
		}
	}
}
