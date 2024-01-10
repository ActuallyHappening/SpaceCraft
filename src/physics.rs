use crate::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			PhysicsPlugins::new(FixedUpdate),
			bevy_xpbd3d_parenting::PhysicsParentingPlugin::default(),
		));
		#[cfg(feature = "debug")]
		app.add_plugins(PhysicsDebugPlugin::default());

		app.insert_resource(Gravity(Vec3::ZERO));
	}
}

mod api {}
