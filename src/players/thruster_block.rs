use crate::prelude::*;

	use crate::blocks::{manual_builder, BlockBlueprint};

	/// Will spawn a particle emitter as a child
	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub struct ThrusterBlock;

	#[derive(Bundle)]
	pub struct ThrusterBlockBundle {
		pbr: PbrBundle,
		collider: AsyncCollider,
		name: Name,
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
				specific_marker: ThrusterBlock,
			}
		}
	}
