use crate::utils::*;

use super::*;

// macro_rules! thrust_type {
// 	($(pub struct $name:ident;)*) => {
// 		$(
// 			#[derive(Component, Debug, Default)]
// 			pub struct $name;
// 		)*
// 	};
// }

// thrust_type!(
// 	pub struct ThrustForwards;
// 	pub struct ThrustRight;
// 	pub struct ThrustUpwards;
// );

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum ThrustType {
	Forward,
	Right,
	Up,

	TurnRight,
	TiltUp,
	RollRight,
}

impl<T: ThrustStage> Thrust<T> {
	pub fn get_from_type(&self, thrust_type: ThrustType) -> &T::DimensionType {
		match thrust_type {
			ThrustType::Forward => &self.forward,
			ThrustType::Right => &self.right,
			ThrustType::Up => &self.up,

			ThrustType::TurnRight => &self.turn_right,
			ThrustType::TiltUp => &self.tilt_up,
			ThrustType::RollRight => &self.roll_right,
		}
	}

	pub fn get_mut_from_type(&mut self, thrust_type: ThrustType) -> &mut T::DimensionType {
		match thrust_type {
			ThrustType::Forward => &mut self.forward,
			ThrustType::Right => &mut self.right,
			ThrustType::Up => &mut self.up,

			ThrustType::TurnRight => &mut self.turn_right,
			ThrustType::TiltUp => &mut self.tilt_up,
			ThrustType::RollRight => &mut self.roll_right,
		}
	}

	pub fn set_from_type(&mut self, thrust_type: ThrustType, value: T::DimensionType) {
		match thrust_type {
			ThrustType::Forward => self.forward = value,
			ThrustType::Right => self.right = value,
			ThrustType::Up => self.up = value,

			ThrustType::TurnRight => self.turn_right = value,
			ThrustType::TiltUp => self.tilt_up = value,
			ThrustType::RollRight => self.roll_right = value,
		}
	}
	
	pub fn for_each(& self, mut f: impl FnMut(&T::DimensionType, ThrustType)) {
		for thrust_type in ThrustType::iter() {
			f(self.get_from_type(thrust_type), thrust_type);
		}
	}
}

impl ThrustType {
	pub fn iter() -> impl Iterator<Item = Self> {
		<Self as IntoEnumIterator>::iter()
	}

	pub const fn ah_circle_name(&self) -> &'static str {
		match self {
			Self::Forward => "forward",
			Self::Right => "right",
			Self::Up => "up",

			Self::TurnRight => "turn",
			Self::TiltUp => "tilt",
			Self::RollRight => "roll",
		}
	}
}
