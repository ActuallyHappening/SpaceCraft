use crate::prelude::*;

/// Camera block that is spawned into the world
#[derive(Bundle)]
pub struct CameraBlockBundle {
	pbr: PbrBundle,
	name: Name,
	id: BlockId,
	marker: CameraBlockMarker,
}

/// Blueprint for [CameraBlockBundle]
#[derive(Debug, Reflect, Serialize, Deserialize, Clone)]
pub struct CameraBlockBlueprint {
	pub id: BlockId,
}

/// Marker for [BlockBlueprint]s that are [CameraBlockBlueprint]s,
/// which spawn [CameraBlockBundle]s.
#[derive(Component)]
pub struct CameraBlockMarker;

impl BlockBlueprint<CameraBlockBlueprint> {
	pub fn new_camera(
		position: impl Into<manual_builder::RelativePixel>,
		facing: impl Into<Quat>,
	) -> Self {
		Self {
			transform: Transform::from_rotation(facing.into())
				.translate(position.into().into_world_offset()),
			mesh: OptimizableMesh::Sphere {
				radius: PIXEL_SIZE / 2.,
			},
			material: OptimizableMaterial::OpaqueColour(Color::BLACK),
			specific_marker: CameraBlockBlueprint {
				id: BlockId::random(),
			},
		}
	}
}

impl Blueprint for BlockBlueprint<CameraBlockBlueprint> {
	type Bundle = CameraBlockBundle;
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
			name: Name::new("CameraBlock"),
			id: specific_marker.id,
			marker: CameraBlockMarker,
		}
	}
}
