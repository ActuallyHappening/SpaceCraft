use crate::{prelude::*, players::player_movement::utils::{Velocity6Dimensions, Velocity6DimensionsMut}};

/// Forces are between [-1, 1],
/// but torque can be infinite so is [SignedFlag] instead
///
/// The Greek philosopher, Archimedes, said,
/// “Give me a lever long enough and a fulcrum on which to place it, and I shall move the world.”
#[derive(Debug, Reflect, Default, Clone, Copy)]
pub struct ForceAxis {
	forward: f32,
	right: f32,
	upwards: f32,
	turn_right: f32,
	pitch_up: f32,
	roll_right: f32,
}

// /// Returns [-1, 1] for factor that rotation is in the direction
// /// of base
// pub fn factor_direction_in(rotation: impl Into<Quat>, base: impl Into<Quat>) -> f32 {
// 	let rotation: Quat = rotation.into();
// 	let base = base.into();

// 	let angle = rotation.angle_between(base);

// 	// 0.0 -> 1.0
// 	// TAU / 4 -> 0.0
// 	// TAU / 2 -> -1.0
// 	let ret: f32 = angle.cos();

// 	#[cfg(test)]
// 	println!(
// 		"Angle between: {:?} & {:?} = {} (ret = {})",
// 		rotation, base, angle, ret
// 	);

// 	ret
// }

impl ForceAxis {
	pub fn from_iter(
		mut forces: impl FnMut(Vec3) -> f32,
		mut torques: impl FnMut(Vec3) -> f32,
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
		let force = ef.force().normalize();
		let forces = |dir: Vec3| force.dot(dir);

		let torque = ef.torque().normalize();
		let torques = |dir: Vec3| torque.dot(dir);

		Self::from_iter(forces, torques)
	}

	// /// How much strength should a thruster exert?
	// /// Negative means reverse
	// pub fn dot(global_goal: ForceAxis, specific_thruster: ForceAxis) -> f32 {
	// 	let mut ret = 0.0;

	// 	ret += global_goal.forward * specific_thruster.forward;
	// 	ret += global_goal.right * specific_thruster.right;
	// 	ret += global_goal.upwards * specific_thruster.upwards;

	// 	ret += global_goal.turn_right.signed_f32() * specific_thruster.turn_right.signed_f32();
	// 	ret += global_goal.pitch_up.signed_f32() * specific_thruster.pitch_up.signed_f32();
	// 	ret += global_goal.roll_right.signed_f32() * specific_thruster.roll_right.signed_f32();

	// 	ret
	// }
}

// #[derive(Debug, Reflect, Default)]
// pub enum SignedFlag {
// 	Flag(bool),
// 	#[default]
// 	Zero,
// }

// impl SignedFlag {
// 	pub fn new(num: f32) -> Self {
// 		const EPSILON: f32 = 0.01;
// 		if num.abs() < EPSILON {
// 			Self::Zero
// 		} else {
// 			Self::Flag(num > 0.0)
// 		}
// 	}

// 	pub fn new_with_epsilon(num: f32, epsilon: f32) -> Self {
// 		if num.abs() < epsilon {
// 			Self::Zero
// 		} else {
// 			Self::Flag(num > 0.0)
// 		}
// 	}

// 	pub fn into_option(self) -> Option<bool> {
// 		match self {
// 			Self::Flag(b) => Some(b),
// 			Self::Zero => None,
// 		}
// 	}

// 	/// Whether [SignedFlag::Zero] or not
// 	pub fn flagged(self) -> bool {
// 		match self {
// 			Self::Flag(_) => true,
// 			Self::Zero => false,
// 		}
// 	}

// 	pub fn flagged_true(self) -> bool {
// 		match self {
// 			Self::Flag(b) => b,
// 			Self::Zero => false,
// 		}
// 	}

// 	pub fn flagged_false(self) -> bool {
// 		match self {
// 			Self::Flag(b) => !b,
// 			Self::Zero => false,
// 		}
// 	}

// 	pub fn signed_f32(self) -> f32 {
// 		match self {
// 			Self::Flag(b) => b as i32 as f32,
// 			Self::Zero => 0.0,
// 		}
// 	}
// }

impl Velocity6Dimensions for ForceAxis {
	fn velocity_upward(&self) -> f32 {
		self.upwards
	}
	fn velocity_rightward(&self) -> f32 {
		self.right
	}
	fn velocity_forward(&self) -> f32 {
		self.forward
	}
	fn angular_turn_right(&self) -> f32 {
		self.turn_right
	}
	fn angular_tilt_up(&self) -> f32 {
		self.pitch_up
	}
	fn angular_roll_right(&self) -> f32 {
		self.roll_right
	}
}

impl Velocity6DimensionsMut for ForceAxis {
	fn forward_mut(&mut self) -> &mut f32 {
		&mut self.forward
	}
	fn right_mut(&mut self) -> &mut f32 {
		&mut self.right
	}
	fn up_mut(&mut self) -> &mut f32 {
		&mut self.upwards
	}
	fn roll_right_mut(&mut self) -> &mut f32 {
		&mut self.roll_right
	}
	fn tilt_up_mut(&mut self) -> &mut f32 {
		&mut self.pitch_up
	}
	fn turn_right_mut(&mut self) -> &mut f32 {
		&mut self.turn_right
	}
}

#[cfg(test)]
mod test {
	use crate::blocks::manual_builder::{Facing, RelativePixel};
	use crate::prelude::*;

	use super::ForceAxis;

	// #[test]
	// fn force_axis_dot() {
	// 	let global_goal = ForceAxis {
	// 		forward: 1.0,
	// 		turn_right: SignedFlag::Flag(true),
	// 		..default()
	// 	};
	// 	let specific_thruster = ForceAxis {
	// 		turn_right: SignedFlag::Flag(true),
	// 		..default()
	// 	};

	// 	assert_near!(ForceAxis::dot(global_goal, specific_thruster), 1.0);

	// 	let global_goal = ForceAxis {
	// 		forward: -1.0,
	// 		..default()
	// 	};
	// 	let specific_thruster = ForceAxis {
	// 		forward: 1.0,
	// 		..default()
	// 	};

	// 	assert_near!(ForceAxis::dot(global_goal, specific_thruster), -1.0);
	// }

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

		assert!(force_axis.turn_right == 1.0);
		assert!(force_axis.pitch_up == 0.0);
		assert!(force_axis.roll_right == 0.0);
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

		// assert_near!(factor_direction_in(Facing::Right, Facing::Forwards), 0.0);
		// assert_near!(factor_direction_in(Facing::Right, Facing::Right), 1.0);
		// assert_near!(factor_direction_in(Facing::Right, Facing::Left), -1.0);
	}
}
