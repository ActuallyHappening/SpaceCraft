use crate::utils::*;

#[derive(Debug, Constructor)]
pub struct RotationArm<const N: usize> {
	pub offset: Quat,
}

impl<const N: usize> RigDriver for RotationArm<N> {
	fn update(&mut self, params: bevy_dolly::dolly::rig::RigUpdateParams) -> Transform {
		Transform {
			rotation: params.parent.rotation.mul_quat(self.offset),
			translation: params.parent.translation,
			scale: Vec3::ONE,
		}
	}
}

#[derive(Debug, Constructor)]
pub struct RotationAccumulator {
	rot: Quat,
}
impl RigDriver for RotationAccumulator {
	fn update(&mut self, params: bevy_dolly::dolly::rig::RigUpdateParams) -> Transform {
		Transform {
			rotation: params.parent.rotation.mul_quat(self.rot),
			translation: params.parent.translation,
			scale: Vec3::ONE,
		}
	}
}
impl RotationAccumulator {
	#[allow(dead_code)]
	pub fn add_rot(&mut self, rot: Quat) {
		self.rot = self.rot.mul_quat(rot);
	}
}

#[derive(Debug, Constructor)]
pub struct OrbitArm {
	offset: Vec3,

	local_up_forward: Option<(Vec3, Vec3)>,

	total_temporary_sideways_rot: f32,
	permanent_sidways_rot: f32,

	total_temporary_vertical_rot: f32,
	permanent_vertical_rot: f32,
}

impl OrbitArm {
	pub fn new_from_arm(offset: Vec3) -> Self {
		Self {
			offset,
			local_up_forward: None,
			total_temporary_sideways_rot: 0.,
			total_temporary_vertical_rot: 0.,
			permanent_sidways_rot: 0.,
			permanent_vertical_rot: 0.,
		}
	}

	fn orbit_horizontal(&mut self, angle: f32) -> &mut Self {
		self.total_temporary_sideways_rot += angle;
		self
	}

	fn orbit_vertical(&mut self, angle: f32) -> &mut Self {
		self.total_temporary_vertical_rot += angle;
		self
	}

	pub fn orbit(
		&mut self,
		local_up: Vec3,
		local_forward: Vec3,
		delta_horizontal_rot: f32,
		delta_vertical_rot: f32,
	) -> &mut Self {
		self.local_up_forward = Some((local_up, local_forward));

		self.orbit_horizontal(delta_horizontal_rot);
		self.orbit_vertical(delta_vertical_rot);

		self
	}

	pub fn permanent_orbit_horizontal(&mut self, angle: f32) -> &mut Self {
		self.permanent_sidways_rot += angle;
		self
	}
	pub fn permanent_orbit_vertical(&mut self, angle: f32) -> &mut Self {
		self.permanent_vertical_rot += angle;
		self
	}

	pub fn adjust_arm_length(&mut self, delta: f32) {
		let current = self.offset.length();
		let new = current.add(delta).clamp(0.1, 100.);
		self.offset = self.offset.normalize() * new;
	}

	pub fn reset(&mut self) {
		self.total_temporary_sideways_rot = 0.;
		self.total_temporary_vertical_rot = 0.;
		self.permanent_sidways_rot = 0.;
		self.permanent_vertical_rot = 0.;
	}
	pub fn reset_percentage(&mut self, percentage: f32) {
		let percent = 1. - percentage;
		self.total_temporary_sideways_rot *= percent;
		self.total_temporary_vertical_rot *= percent;
	}
}

impl RigDriver for OrbitArm {
	fn update(&mut self, params: bevy_dolly::dolly::rig::RigUpdateParams) -> Transform {
		match self.local_up_forward {
			None => {
				warn!("You must call `.orbit()` on [OrbitalArm] before using it every frame!");
				params.parent.with_translation(self.offset)
			}
			Some((up, forward)) => {
				let mut transform = *params.parent;

				// let rot = Quat::from_rotation_y();
				transform.rotate_axis(up, self.total_temporary_sideways_rot + self.permanent_sidways_rot);
				transform.rotate_axis(up.cross(forward), self.total_temporary_vertical_rot + self.permanent_vertical_rot);

				transform.translation += transform.rotation * self.offset; // arm
				self.local_up_forward = None;
				transform
			}
		}
	}
}
