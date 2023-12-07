use bevy::render::render_resource::ShaderRef;
use bevy::{prelude::*, render::render_resource::AsBindGroup};

mod instancing;
mod post_processing;

fn main() {
	// post_processing::main();

	instancing::main();

	// let mut app = App::new();

	// app.add_plugins((DefaultPlugins, bevy_editor_pls::EditorPlugin::default(), WhiteMaterial {}));

	// app.add_systems(Startup, setup);

	// app.run();
}

#[derive(Asset, Debug, AsBindGroup, TypePath, Clone)]
struct WhiteMaterial {}

impl Plugin for WhiteMaterial {
	fn build(&self, app: &mut App) {
		app.add_plugins(MaterialPlugin::<WhiteMaterial>::default());
	}
}

impl Material for WhiteMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/white_material.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<WhiteMaterial>>,
) {
	commands.spawn(MaterialMeshBundle {
		mesh: meshes.add(Mesh::from(shape::UVSphere {
			radius: 2.0,
			..default()
		})),
		material: materials.add(WhiteMaterial {}),
		..default()
	});

	commands.spawn(Camera3dBundle {
		transform: Transform::from_translation(Vec3::new(0., 0., 10.)),
		..default()
	});
}
