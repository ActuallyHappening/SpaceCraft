use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructurePart {
	Pixel {
		px: Pixel,
		relative_location: RelativePixelPoint,
	},
	Thruster {
		thrust: Thruster,
		relative_location: RelativePixelPoint,
	},
	Weapon {
		weapon: Weapon,
		relative_location: RelativePixelPoint,
	},
}

impl StructurePart {
	pub fn compute_shape(&self) -> Option<(Vec3, Quat, Collider)> {
		match self {
			StructurePart::Thruster { .. } | StructurePart::Weapon { .. } => None,
			StructurePart::Pixel {
				relative_location,
				..
			} => Some({
				let pos = relative_location.clone().into_world_vector();
				let rot = Quat::IDENTITY;
				let shape = Collider::cuboid(PIXEL_SIZE / 2., PIXEL_SIZE / 2., PIXEL_SIZE / 2.);
				(pos, rot, shape)
			}),
		}
	}
}

impl<L> From<(Thruster, L)> for StructurePart
where
	L: Into<RelativePixelPoint>,
{
	fn from((thrust, relative_location): (Thruster, L)) -> Self {
		Self::Thruster {
			thrust,
			relative_location: relative_location.into(),
		}
	}
}

impl<L> From<(Weapon, L)> for StructurePart
where
	L: Into<RelativePixelPoint>,
{
	fn from((weapon, relative_location): (Weapon, L)) -> Self {
		Self::Weapon {
			weapon,
			relative_location: relative_location.into(),
		}
	}
}

impl<P, L> From<(P, L)> for StructurePart
where
	P: Into<Pixel>,
	L: Into<RelativePixelPoint>,
{
	fn from((px, relative_location): (P, L)) -> Self {
		Self::Pixel {
			px: px.into(),
			relative_location: relative_location.into(),
		}
	}
}
