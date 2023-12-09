use crate::prelude::*;

use super::manual_builder;
use super::BlockBlueprint;

/// Used for building structures
#[derive(Debug, Reflect, Serialize, Deserialize, Clone, IntoStaticStr)]
pub enum StructureBlockBlueprint {
	Aluminum,
}

impl StructureBlockBlueprint {
	pub fn name(&self) -> &'static str {
		self.into()
	}
}

#[derive(Bundle)]
pub struct StructureBlockBundle {
	pbr: PbrBundle,
	collider: AsyncCollider,
	name: Name,
}

impl Blueprint for BlockBlueprint<StructureBlockBlueprint> {
	type Bundle = StructureBlockBundle;
	type StampSystemParam<'w, 's> = MMA<'w>;

	fn stamp(&self, mma: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle {
		let BlockBlueprint {
			transform,
			mesh,
			material,
			specific_marker,
		} = self;
		Self::Bundle {
			pbr: PbrBundle {
				transform: *transform,
				mesh: mesh.clone().into_mesh(mma),
				material: material.clone().into_material(&mut mma.mats),
				..default()
			},
			collider: AsyncCollider(ComputedCollider::ConvexHull),
			name: Name::new(format!("StructureBlock {}", specific_marker.name())),
		}
	}
}

impl BlockBlueprint<StructureBlockBlueprint> {
	pub fn new_structure(
		block: StructureBlockBlueprint,
		location: impl Into<manual_builder::RelativePixel>,
	) -> Self {
		BlockBlueprint {
			transform: Transform::from_translation(location.into().into_world_offset()),
			mesh: super::OptimizableMesh::StandardBlock,
			material: super::OptimizableMaterial::OpaqueColour(Color::SILVER),
			specific_marker: block,
		}
	}
}
