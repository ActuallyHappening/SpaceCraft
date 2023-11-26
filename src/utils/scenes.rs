use crate::prelude::*;

pub struct HelperScene;

#[derive(Component, Reflect, Default)]
pub struct HelperSceneMarker {
	save_recursive: bool,
}

#[derive(Resource, Reflect, Clone)]
pub struct HelperSceneRes {
	path: String,
}

impl Default for HelperSceneRes {
	fn default() -> Self {
		Self {
			path: "cool.scene.ron".into(),
		}
	}
}

impl Plugin for HelperScene {
	fn build(&self, app: &mut App) {
		app
			.register_type::<HelperSceneMarker>()
			.register_type::<HelperSceneRes>()
			.init_resource::<HelperSceneRes>()
			.add_systems(Update, Self::update);
	}
}

impl HelperScene {
	fn update(world: &mut World) {
		let mut scene_world = World::new();

		let type_registry = world.resource::<AppTypeRegistry>().clone();
		scene_world.insert_resource(type_registry);

		let config = world.resource::<HelperSceneRes>().clone();

		for (entity, marker) in world.query::<(Entity, &HelperSceneMarker)>().iter(world) {
			if marker.save_recursive {
				// add to scene world this entity, and any children
				// scene_world

				// todo: organize a way to save the scene better
			}
		}
	}
}
