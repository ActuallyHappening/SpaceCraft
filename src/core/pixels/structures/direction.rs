use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
	Forward,
	Backward,
	Left,
	Right,
	Up,
	Down,
}

impl Direction {
	/// From/Into impl, but use explicit method where possible
	pub fn into_rotation(self) -> Quat {
		impl From<Direction> for Quat {
			fn from(value: Direction) -> Self {
				value.into_rotation()
			}
		}

		match self {
			Self::Backward => Quat::from_rotation_x(90f32.to_radians()),
			Self::Forward => Quat::from_rotation_x(-90f32.to_radians()),
			Self::Left => Quat::from_rotation_z(90f32.to_radians()),
			Self::Right => Quat::from_rotation_z(-90f32.to_radians()),
			Self::Up => Quat::IDENTITY,
			Self::Down => Quat::from_rotation_z(180f32.to_radians()),
		}
	}

	pub fn into_direction_vector(self) -> Vec3 {
		impl From<Direction> for Vec3 {
			fn from(value: Direction) -> Self {
				value.into_direction_vector()
			}
		}

		match self {
			Self::Backward => Vec3::new(0., 0., 1.),
			Self::Forward => Vec3::new(0., 0., -1.),
			Self::Left => Vec3::new(-1., 0., 0.),
			Self::Right => Vec3::new(1., 0., 0.),
			Self::Up => Vec3::new(0., 1., 0.),
			Self::Down => Vec3::new(0., -1., 0.),
		}
	}
}

impl Reflection for Direction {
	fn reflect_horizontally(self) -> Self {
		match self {
			Self::Left => Self::Right,
			Self::Right => Self::Left,
			_ => self,
		}
	}

	fn reflect_vertically(self) -> Self {
		match self {
			Self::Up => Self::Down,
			Self::Down => Self::Up,
			_ => self,
		}
	}
}
