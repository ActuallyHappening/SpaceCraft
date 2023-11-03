use crate::utils::*;
use bevy::render::{mesh, render_resource::PrimitiveTopology};

pub struct Triangle {
	height: f32,
	width: f32,
}

impl Triangle {
	/// Creates a triangle, centered at the midpoint of the base of the triangle
	pub const fn new(width: f32, height: f32) -> Self {
		Self { width, height }
	}
}

impl From<Triangle> for Mesh {
	fn from(Triangle { width, height }: Triangle) -> Self {
		let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

		// Positions of the vertices
		// See https://bevy-cheatbook.github.io/features/coords.html
		mesh.insert_attribute(
			Mesh::ATTRIBUTE_POSITION,
			vec![
				[-width / 2., 0., 0.],
				[width / 2., 0., 0.],
				[0., height, 0.],
			],
		);

		// In this example, normals and UVs don't matter,
		// so we just use the same value for all of them
		// mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 3]);
		mesh.duplicate_vertices();
		// mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 3]);
		mesh.compute_flat_normals();

		// A triangle using vertices 0, 2, and 1.
		// Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
		mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1])));

		mesh
	}
}

#[allow(dead_code)]
pub fn spawn_test_triangle(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(PbrBundle {
		mesh: meshes.add(Triangle::new(10., 10.).into()),
		material: materials.add(Color::RED.into()),
		transform: Transform::from_xyz(0., 0., 0.),
		..Default::default()
	});
}
