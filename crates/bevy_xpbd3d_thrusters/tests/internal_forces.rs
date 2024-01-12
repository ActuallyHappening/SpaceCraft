use bevy::{ecs::system::RunSystemOnce as _, prelude::*};
use bevy_xpbd3d_thrusters::prelude::*;

fn assert_all_internal_forces_are(value: Vec3) -> impl Fn(Query<&InternalForce>) {
	move |internal_forces| {
		for force in internal_forces.iter() {
			assert_eq!(force.inner(), value);
		}
	}
}

#[test]
fn starts_with_no_internal_force() {
	// setup
	let mut app = App::new();
	app.add_plugins((MinimalPlugins, ThrusterPlugin::new(Update)));

	// spawns one thruster
	let thruster = app.world.spawn(Thruster::new_with_strength_factor(2.)).id();

	// ticks once
	app.world.run_schedule(Main);

	let mut thruster = app.world.entity_mut(thruster);
	assert_eq!(thruster.get::<InternalForce>().unwrap().inner(), Vec3::ZERO);
}
