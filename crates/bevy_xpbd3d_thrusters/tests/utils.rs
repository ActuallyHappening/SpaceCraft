#![allow(dead_code)]

pub use bevy::prelude::*;
pub use bevy_xpbd_3d_parenting::{prelude::*, InternalForce};
pub use bevy_xpbd3d_thrusters::prelude::*;
pub use rand::random;

pub fn test_app() -> App {
	let mut app = App::new();
	app.add_plugins((
		MinimalPlugins,
		PhysicsParentingPlugin::default(),
		ThrusterPlugin::new(Update),
	));
	app
}

pub fn assert_all_internal_forces_are(value: Vec3) -> impl Fn(Query<&InternalForce>) {
	move |internal_forces| {
		for force in internal_forces.iter() {
			assert_eq!(force.inner(), value);
		}
	}
}
