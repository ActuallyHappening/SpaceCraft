use super::*;

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Thruster {
	pub facing: Direction,
	pub flags: ThrusterFlags,
}

impl Thruster {
	pub const fn new(facing: Direction, flags: ThrusterFlags) -> Thruster {
		Thruster { facing, flags }
	}
}

impl Reflection for Thruster {
	fn reflect_horizontally(mut self) -> Self {
		self.facing = self.facing.reflect_horizontally();
		self.flags = self.flags.reflect_horizontally();
		self
	}

	fn reflect_vertically(mut self) -> Self {
		self.facing = self.facing.reflect_vertically();
		self.flags = self.flags.reflect_vertically();
		self
	}
}
