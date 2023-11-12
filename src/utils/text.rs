use bevy::sprite::Mesh2dHandle;
use meshtext::{MeshGenerator, MeshText, TextSection};

use crate::prelude::*;

/// Configuration for creating 2d text
#[derive(Debug, Clone)]
pub struct Text2d {
	pub material: ColorMaterial,
	pub transform: Transform,
	pub font: GlobalFont,
	pub pixel_size: f32,
}

impl Default for Text2d {
	fn default() -> Self {
		Self {
			material: Color::PURPLE.into(),
			transform: Default::default(),
			font: GlobalFont::Default,
			pixel_size: 16.,
		}
	}
}

/// Encapsulates 2D text with an offset-ed transform
#[derive(Bundle, Default)]
pub struct Text2dBundle {
	mesh: Mesh2dHandle,
	material: Handle<ColorMaterial>,

	spatial: SpatialBundle,
}

impl Text2dBundle {
	pub fn new(text: impl Into<Cow<'static, str>>, config: Text2d, mma: &mut MM2) -> Self {
		let mut generator: MeshGenerator<_> = config.font.into();

		// 2d generation because z-value = 0
		let transform = Mat4::from_scale(Vec3::new(config.pixel_size, config.pixel_size, 0.)).to_cols_array();
		let text_mesh: MeshText = generator
			.generate_section(&text.into(), true, Some(&transform))
			.unwrap();

		let vertices = text_mesh.vertices;
		let positions: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
		let uvs = vec![[0f32, 0f32]; positions.len()];

		let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
		mesh.compute_flat_normals();

		let offset = Vec2::new(text_mesh.bbox.size().x / -2., text_mesh.bbox.size().y / -2.);

		Text2dBundle {
			mesh: mma.meshs.add(mesh).into(),
			material: mma.mats.add(config.material),
			spatial: SpatialBundle{
				transform: config.transform.translate(Vec3::new(offset.x, offset.y, 0.)),
				..default()
			}
		}
	}
}