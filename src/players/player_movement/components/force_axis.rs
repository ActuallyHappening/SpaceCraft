use crate::prelude::*;

/// Forces are between [-1, 1],
/// but torque can be infinite so is [SignedFlag] instead
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
