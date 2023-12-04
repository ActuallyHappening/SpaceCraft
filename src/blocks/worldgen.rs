use crate::prelude::*;

mod terrain_blueprint {
	use crate::{prelude::*, blocks::manual_builder::RelativePixel};

	use super::{terrain_type::TerrainType, discrete_shapes::OptimizableDiscreteShape};

	/// Blueprint for [TerrainItemBundle]
	#[derive(Serialize, Deserialize, Debug)]
	pub struct TerrainItemBlueprint {
		pub terrain_type: TerrainType,
		pub location: RelativePixel,
	}

	/// Blueprint for [TerrainStructureBundle]
	#[derive(Component, Serialize, Deserialize, Debug)]
	pub struct TerrainStructureBlueprint {
		pub transform: Transform,
		pub(super) children: Vec<TerrainItemBlueprint>,
	}
}

mod discrete_shapes {
	use crate::prelude::*;
	use std::num::NonZeroU8;

	use crate::blocks::manual_builder::RelativePixel;

	#[derive(Debug, Serialize, Deserialize)]
	pub enum OptimizableDiscreteShape {
		Sphere(DiscreteSphere),
		Dot { pos: RelativePixel },
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct DiscreteSphere {
		pub radius: NonZeroU8,
	}

	pub trait DiscreteLocations {
		fn get_locations(self) -> HashSet<RelativePixel>;
	}

	impl DiscreteLocations for DiscreteSphere {
		fn get_locations(self) -> HashSet<RelativePixel> {
			let mut locations = HashSet::new();
			let DiscreteSphere { radius } = self;
			let radius = radius.get() as i32;
			for x in -radius..=radius {
				for y in -radius..=radius {
					for z in -radius..=radius {
						let offset = IVec3::new(x, y, z);
						if offset.as_vec3().length() <= radius as f32 {
							locations.insert(RelativePixel(offset));
						}
					}
				}
			}
			locations
		}
	}

	impl DiscreteLocations for OptimizableDiscreteShape {
		fn get_locations(self) -> HashSet<RelativePixel> {
			match self {
				Self::Dot { pos } => [pos].into_iter().collect(),
				Self::Sphere(sphere) => sphere.get_locations(),
			}
		}
	}
}

mod terrain_bundle {
	use crate::prelude::*;

	use super::terrain_blueprint::{TerrainStructureBlueprint, TerrainItemBlueprint};

	/// A single terrain unit
	#[derive(Bundle)]
	pub struct TerrainItemBundle {
		pbr: PbrBundle,
		name: Name,
		collider: AsyncCollider,
	}

	impl FromBlueprint for TerrainItemBundle {
		type Blueprint = TerrainItemBlueprint;

		fn stamp_from_blueprint(TerrainItemBlueprint { terrain_type, location }: &Self::Blueprint, mma: &mut MMA) -> Self {
			Self {
				pbr: PbrBundle {
					transform: Transform::from_translation(location.into_world_offset()),
					mesh: mma.meshs.add(terrain_type.mesh()),
					..default()
				},
				name: Name::new(terrain_type.name()),
				collider: AsyncCollider(ComputedCollider::TriMesh)
			}	
		}
	}

	/// The parent of many [TerrainItemBundle]s
	#[derive(Bundle)]
	pub struct TerrainStructureBundle {
		spatial: SpatialBundle,
		name: Name,
	}

	impl FromBlueprint for TerrainStructureBundle {
		type Blueprint = TerrainStructureBlueprint;

		fn stamp_from_blueprint(TerrainStructureBlueprint { transform, .. }: &Self::Blueprint, _mma: &mut MMA) -> Self {
			Self {
				spatial: SpatialBundle::from_transform(*transform),
				name: Name::new("TerrainStructure"),
			}
		}
	}
}

mod terrain_type {
	use crate::{blocks::manual_builder::RelativePixel, prelude::*};

	#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, IntoStaticStr)]
	pub enum TerrainType {
		SilicateRock,
	}

	type TT = TerrainType;

	impl TerrainType {
		pub(super) const fn mesh(&self) -> OptimizableMesh {
			match self {
				TT::SilicateRock => OptimizableMesh::CustomRectangularPrism {
					size: Vec3::splat(PIXEL_SIZE),
				},
			}
		}

		pub(super) fn material(&self) -> OptimizableMaterial {
			match self {
				TT::SilicateRock => OptimizableMaterial::OpaqueColour(Color::rgb_u8(84, 84, 84)),
			}
		}

		pub fn name(&self) -> &'static str {
			self.into()
		}
	}
}
