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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructureBlock;

#[derive(Bundle)]
pub struct StructureBlockBundle {
	spatial: SpatialBundle,
	collider: AsyncCollider,
	name: Name,
}

/// Since raw [Mesh] cannot be serialized
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizableMesh {
	StandardBlock,
	FromAsset(String),
}

impl OptimizableMesh {
	pub fn get_mesh(&self, ass: &mut AssetServer) -> Handle<Mesh> {
		match self {
			Self::FromAsset(name) => ass.load(name),
			Self::StandardBlock => ass.add(shape::Cube { size: PIXEL_SIZE }.into()),
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