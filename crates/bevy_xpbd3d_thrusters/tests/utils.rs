#![allow(dead_code)]

pub use bevy::prelude::*;
pub use bevy_xpbd_3d_parenting::prelude::*;
pub use bevy_xpbd3d_thrusters::prelude::*;
pub use bevy_xpbd_3d::prelude::*;
pub use rand::random;

/// This crate heavily relies on other crates
/// like bevy_hierachy and [bevy_xpbd] which take
/// a few frame to initialize
const SETUP_ITERATION: u8 = 5;

pub fn test_app() -> App {
	let mut app = App::new();
	app.add_plugins((
		MinimalPlugins,
		ParentingPlugin::new(Update),
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
