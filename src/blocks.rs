use crate::prelude::*;

mod worldgen;

pub struct BlockPlugins;

impl PluginGroup for BlockPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>().add(worldgen::WorldGenPlugin)
	}
}

/// The unique identifier for a persistent block in the world
#[derive(Reflect, Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct BlockId(u64);

impl Default for BlockId {
	fn default() -> Self {
		warn!("Generating default BlockId!!");
		Self(0)
	}
}

impl BlockId {
	pub fn random() -> Self {
		Self(random())
	}
}

/// Represents a 'block', which is useful for spawning standardized structures
/// like thrusters.
///
/// Assumes all blocks have a position, material and mesh. These restrictions may be lifted
/// if ever there was a need.
#[derive(Debug, Serialize, Deserialize, Clone, Deref)]
pub struct BlockBlueprint<T> {
	pub transform: Transform,
	pub mesh: OptimizableMesh,
	pub material: OptimizableMaterial,
	#[deref]
	pub specific_marker: T,
}

pub mod manual_builder {
	use crate::prelude::*;

	pub enum Facing {
		Up,
		Down,
		Left,
		Right,
		Forwards,
		Backwards,
	}

	impl Facing {
		pub fn into_quat(self) -> Quat {
			match self {
				Self::Forwards => Quat::IDENTITY,
				Self::Backwards => Quat::from_rotation_y(TAU / 2.),
				Self::Right => Quat::from_rotation_y(-TAU / 4.),
				Self::Left => Quat::from_rotation_y(TAU / 4.),
				Self::Up => Quat::from_rotation_x(TAU / 4.),
				Self::Down => Quat::from_rotation_x(-TAU / 4.),
			}
		}
	}

	impl From<Facing> for Quat {
		fn from(facing: Facing) -> Self {
			facing.into_quat()
		}
	}

	#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
	pub struct RelativePixel(pub IVec3);

	impl From<IVec3> for RelativePixel {
		fn from(vec: IVec3) -> Self {
			Self(vec)
		}
	}

	impl RelativePixel {
		pub fn new(x: i32, y: i32, z: i32) -> Self {
			Self::from(IVec3::new(x, y, z))
		}

		pub fn into_world_offset(self) -> Vec3 {
			self.0.as_vec3().mul(PIXEL_SIZE)
		}
	}

	#[cfg(test)]
	mod test {
		use super::*;

		#[test]
		fn facing_forwards() {
			let quat = Facing::Forwards.into_quat();
			let zero = Transform::from_rotation(quat);

			let forwards = zero.forward().normalize();

			assert_eq!(forwards, -Vec3::Z);
		}

		#[test]
		fn facing_right() {
			let quat = Facing::Right.into_quat();
			let zero = Transform::from_rotation(quat);

			let forwards = zero.forward().normalize();

			assert_eq!(forwards, Vec3::X);
		}

		#[test]
		fn facing_up() {
			let quat = Facing::Up.into_quat();
			let zero = Transform::from_rotation(quat);

			let forwards = zero.forward().normalize();

			assert_eq!(forwards, Vec3::Y);
		}
	}
}

pub use structure_block::{StructureBlockBlueprint, StructureBlockBundle};
mod structure_block {
	use crate::prelude::*;

	use super::manual_builder;
	use super::BlockBlueprint;

	/// Used for building structures
	#[derive(Debug, Serialize, Deserialize, Clone, IntoStaticStr)]
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
}

/// Since raw [Mesh] cannot be serialized
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizableMesh {
	StandardBlock,
	CustomRectangularPrism {
		/// Lengths of each side
		size: Vec3,
	},
	Sphere {
		radius: f32,
	},
	FromAsset(String),
}

impl OptimizableMesh {
	pub fn into_mesh(self, mma: &mut MMA) -> Handle<Mesh> {
		match self {
			Self::FromAsset(name) => mma.ass.load(name),
			Self::StandardBlock => mma.meshs.add(shape::Cube { size: PIXEL_SIZE }.into()),
			Self::CustomRectangularPrism { size } => mma
				.meshs
				.add(shape::Box::new(size.x, size.y, size.z).into()),
			Self::Sphere { radius } => mma.meshs.add(
				shape::UVSphere {
					radius,
					..default()
				}
				.into(),
			),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizableMaterial {
	OpaqueColour(Color),
	None,
}

impl OptimizableMaterial {
	pub fn into_material(self, mat: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
		match self {
			Self::OpaqueColour(col) => mat.add(col.into()),
			Self::None => mat.add(Color::WHITE.with_a(0.).into()),
		}
	}
}
