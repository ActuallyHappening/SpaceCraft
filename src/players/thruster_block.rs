use crate::prelude::*;

use crate::blocks::{manual_builder, BlockBlueprint};

/// Will spawn a particle emitter as a child
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThrusterBlock {
	id: BlockId,
	strength: f32,
}

impl ThrusterBlock {
	pub fn new() -> Self {
		Self {
			id: BlockId::random(),
			strength: 10_000.,
		}
	}
}

/// Component for all thrusters (on a player)
#[derive(Debug, Component, Reflect, Clone)]
pub struct Thruster {
	id: BlockId,
	strength_factor: f32,
	/// Between 0..=1, synced with visuals and physics
	pub current_status: f32,
}

impl From<ThrusterBlock> for Thruster {
	fn from(ThrusterBlock { id, strength }: ThrusterBlock) -> Self {
		Thruster {
			id,
			strength_factor: strength,
			current_status: 0.,
		}
	}
}

#[derive(Bundle)]
pub struct ThrusterBlockBundle {
	pbr: PbrBundle,
	collider: AsyncCollider,
	name: Name,
	thruster: Thruster,
}

impl FromBlueprint for ThrusterBlockBundle {
	type Blueprint = BlockBlueprint<ThrusterBlock>;

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
			name: Name::new("ThrusterBlock"),
			thruster: specific_marker.clone().into(),
		}
	}
}

impl BlockBlueprint<ThrusterBlock> {
	pub fn new_thruster(location: manual_builder::RelativePixel, facing: impl Into<Quat>) -> Self {
		let rotation = facing.into();
		BlockBlueprint {
			transform: Transform {
				translation: location.as_vec3() * PIXEL_SIZE
					+ Transform::from_rotation(rotation).forward() * PIXEL_SIZE / 2.,
				rotation,
				..default()
			},
			mesh: OptimizableMesh::CustomRectangularPrism {
				size: Vec3::splat(PIXEL_SIZE / 2.),
			},
			material: OptimizableMaterial::OpaqueColour(Color::RED),
			specific_marker: ThrusterBlock::new(),
		}
	}
}
