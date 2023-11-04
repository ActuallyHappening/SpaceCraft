use crate::bevy::WeaponInfo;

use super::*;

#[derive(Debug, Clone, Component, Serialize, Deserialize, Constructor)]
pub struct Weapon {
	pub facing: Direction,
	pub info: WeaponInfo,
}

// impl Weapon {
// 	pub fn new(facing: Direction, info: WeaponInfo) -> Self {
// 		Self {
// 			facing,
// 			info,
// 		}
// 	}
// }

impl Reflection for Weapon {
	fn reflect_horizontally(mut self) -> Self {
		self.facing = self.facing.reflect_horizontally();
		self
	}

	fn reflect_vertically(mut self) -> Self {
		self.facing = self.facing.reflect_vertically();
		self
	}
}
