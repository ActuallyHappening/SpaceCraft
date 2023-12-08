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
