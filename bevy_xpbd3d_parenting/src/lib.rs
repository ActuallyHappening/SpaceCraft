use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

pub struct PhysicsParentingPlugin;

impl Plugin for PhysicsParentingPlugin {
	fn build(&self, app: &mut App) {}
}

/// Marker component, symbolizing that the `ExternalForce` of this component should
/// be applied to its parent
pub struct ParentingExternalForce;

#[cfg(test)]
mod test {
	use bevy::winit::WinitPlugin;

	use super::*;

	#[test]
	fn normal_external_force() {
		let mut app = App::new();

		app.add_plugins((
			DefaultPlugins
				.set(bevy::log::LogPlugin {
					filter: "bevy_xpbd_3d=trace,bevy_xpbd3d_parenting=trace".into(),
					level: bevy::log::Level::INFO,
				})
				.disable::<WinitPlugin>(),
			PhysicsPlugins::default(),
		));

		let parent = app
			.world
			.spawn((
				RigidBody::Dynamic,
				MassPropertiesBundle::new_computed(&Collider::ball(1.0), 1.0),
			))
			.id();

		app.update();

		assert!(app.world.get::<Transform>(parent) == Some(&Transform::IDENTITY));
	}
}
