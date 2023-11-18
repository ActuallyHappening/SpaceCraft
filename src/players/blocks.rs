use crate::prelude::*;

/// The unique identifier for a persistent block in the world
#[derive(Reflect, Debug, Clone, Copy, Component)]
#[reflect(Component)]
pub struct BlockId(u128);

impl Default for BlockId {
	fn default() -> Self {
		warn!("Generating default BlockId!!");
		Self(0)
	}
}

/// Represents a 'block', which is useful for spawning standardized structures
/// like thrusters.
///
/// Assumes all blocks have a position, material and mesh. These restrictions may be lifted
/// if ever there was a need.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockBlueprint<T> {
	pub transform: Transform,
	pub mesh: OptimizableMesh,
	pub material: OptimizableMaterial,
	pub specific_marker: T,
}

pub mod manual_builder {
	use crate::prelude::*;

	pub enum Facing {
		Up,
	}

	pub type RelativePixel = IVec3;
}

pub use structure_block::{StructureBlock, StructureBlockBundle};
mod structure_block {
	use crate::prelude::*;

	use super::manual_builder;
	use super::BlockBlueprint;

	#[derive(Debug, Serialize, Deserialize, Clone, IntoStaticStr)]
	pub enum StructureBlock {
		Aluminum,
	}

	impl StructureBlock {
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

	impl FromBlueprint for StructureBlockBundle {
		type Blueprint = BlockBlueprint<StructureBlock>;

		fn stamp_from_blueprint(
			BlockBlueprint {
				transform,
				mesh,
				material,
				specific_marker,
			}: &Self::Blueprint,
			mma: &mut MMA,
		) -> Self {
			Self {
				pbr: PbrBundle {
					transform: *transform,
					mesh: mesh.get_mesh(mma),
					material: material.get_material(&mut mma.mats),
					..default()
				},
				collider: AsyncCollider(ComputedCollider::ConvexHull),
				name: Name::new(format!("StructureBlock {}", specific_marker.name())),
			}
		}
	}

	impl BlockBlueprint<StructureBlock> {
		pub fn new(block: StructureBlock, location: manual_builder::RelativePixel) -> Self {
			BlockBlueprint {
				transform: Transform::from_translation(location.as_vec3() * PIXEL_SIZE),
				mesh: super::OptimizableMesh::StandardBlock,
				material: super::OptimizableMaterial::OpaqueColour(Color::SILVER),
				specific_marker: block,
			}
		}
	}
}

/// Since raw [Mesh] cannot be serialized
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizableMesh {
	StandardBlock,
	FromAsset(String),
}

impl OptimizableMesh {
	pub fn get_mesh(&self, mma: &mut MMA) -> Handle<Mesh> {
		match self {
			Self::FromAsset(name) => mma.ass.load(name),
			Self::StandardBlock => mma.meshs.add(shape::Cube { size: PIXEL_SIZE }.into()),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizableMaterial {
	OpaqueColour(Color),
}

impl OptimizableMaterial {
	pub fn get_material(&self, mat: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
		match self {
			Self::OpaqueColour(col) => mat.add((*col).into()),
		}
	}
}
