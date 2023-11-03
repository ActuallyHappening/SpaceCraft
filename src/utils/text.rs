use super::*;

use bevy::sprite::Mesh2dHandle;
use meshtext::{MeshGenerator, MeshText, TextSection};

/// Encapsulates 2D text with an offset-ed transform
#[derive(Bundle, Clone, Default)]
pub struct Text2dBundle {
	pub mesh: Mesh2dHandle,
	pub material: Handle<ColorMaterial>,
	/// Is given an offset to center the text by default
	transform: Transform,
	pub global_transform: GlobalTransform,
	/// User indication of whether an entity is visible
	pub visibility: Visibility,
	/// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
	pub computed_visibility: ComputedVisibility,
}

impl Text2dBundle {
	pub fn new(
		text: impl Into<Cow<'static, str>>,
		font: Font,
		font_size: f32,
		colour: Color,
		mma: &mut MM2,
	) -> Text2dBundle {
		let (mesh, offset) = get_text_mesh(text, font_size, font);

		Text2dBundle {
			mesh: mma.meshs.add(mesh).into(),
			transform: Transform::from_translation(offset),
			material: mma.mats.add(colour.into()),
			..default()
		}
	}

	pub fn translate(mut self, delta: Vec3) -> Self {
		self.transform.translation += delta;
		self
	}

	pub fn translate_x(mut self, delta_x: f32) -> Self {
		self.transform.translation.x += delta_x;
		self
	}
	pub fn translate_y(mut self, delta_y: f32) -> Self {
		self.transform.translation.y += delta_y;
		self
	}
	pub fn translate_z(mut self, delta_z: f32) -> Self {
		self.transform.translation.z += delta_z;
		self
	}
}

/// Generates text mesh
fn get_text_mesh_with_bbox(
	text: impl Into<Cow<'static, str>>,
	pixel_size: f32,
	font: Font,
) -> (Mesh, meshtext::BoundingBox) {
	let mut generator = {
		match font {
			Font::Medium => MeshGenerator::new(include_bytes!("../../assets/fonts/FiraMono-Medium.ttf")),
		}
	};

	let transform = Mat4::from_scale(Vec3::new(pixel_size, pixel_size, 0.)).to_cols_array();
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

	(mesh, text_mesh.bbox)
}

#[extension(pub trait BoundingBoxExt)]
impl meshtext::BoundingBox {
	fn get_required_text_offset(self) -> Vec3 {
		Vec3::X * (self.size().x / -2.) + Vec3::Y * (self.size().y / -2.)
	}
}

/// Returns mesh + offset (to ensure coordinates start in center of text).
/// Without taking offset into account, text will be rendered with *top right* corner at center of entity.
fn get_text_mesh(text: impl Into<Cow<'static, str>>, pixel_size: f32, font: Font) -> (Mesh, Vec3) {
	let (mesh, bbox) = get_text_mesh_with_bbox(text, pixel_size, font);
	(mesh, bbox.get_required_text_offset())
}
