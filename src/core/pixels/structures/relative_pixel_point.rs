use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Mul, Serialize, Deserialize)]
pub struct RelativePixelPoint {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl RelativePixelPoint {
	pub const fn new(x: i32, y: i32, z: i32) -> RelativePixelPoint {
		RelativePixelPoint { x, y, z }
	}

	pub fn into_world_vector(self) -> Vec3 {
		Vec3::from(self) * PIXEL_SIZE
	}
}

impl From<RelativePixelPoint> for Vec3 {
	fn from(value: RelativePixelPoint) -> Self {
		Vec3::new(value.x as f32, value.y as f32, value.z as f32)
	}
}

impl From<(i32, i32, i32)> for RelativePixelPoint {
	fn from((x, y, z): (i32, i32, i32)) -> Self {
		Self::new(x, y, z)
	}
}

impl Reflection for RelativePixelPoint {
	fn reflect_horizontally(self) -> Self {
		Self::new(-self.x, self.y, self.z)
	}

	fn reflect_vertically(self) -> Self {
		Self::new(self.x, -self.y, self.z)
	}
}
