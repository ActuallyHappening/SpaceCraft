use crate::prelude::*;

use self::terrain::TerrainType;

/// Spawns a natural terrain structure
#[derive(Component, Serialize, Deserialize, Debug)]
pub enum TerrainStructureBlueprint {
	Dot(TerrainType),
	Sphere(TerrainType),
}

mod terrain {
	use crate::{blocks::manual_builder::RelativePixel, prelude::*};

	#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
	pub enum TerrainType {
		SilicateRock,
	}

	type TT = TerrainType;

	impl TerrainType {
		const fn mesh(&self) -> OptimizableMesh {
			match self {
				TT::SilicateRock => OptimizableMesh::CustomRectangularPrism {
					size: Vec3::splat(PIXEL_SIZE),
				},
			}
		}

		fn material(&self) -> OptimizableMaterial {
			match self {
				TT::SilicateRock => OptimizableMaterial::OpaqueColour(Color::rgb_u8(84, 84, 84)),
			}
		}
	}

	impl BlockBlueprint<TerrainType> {
		pub fn new_terrain(terrain_type: TerrainType, location: RelativePixel) -> Self {
			Self {
				transform: Transform::from_translation(location.into_world_offset()),
				mesh: terrain_type.mesh(),
				material: terrain_type.material(),
				specific_marker: terrain_type,
			}
		}
	}


}
