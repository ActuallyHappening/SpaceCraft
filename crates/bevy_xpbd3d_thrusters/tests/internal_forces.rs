use bevy::prelude::*;
use bevy_xpbd3d_thrusters::prelude::*;

mod utils;
use utils::*;

#[test]
fn starts_with_zero_internal_force() {
	// setup
	let mut app = test_app();

	let strength_factor = random::<f32>().abs();

	// spawns one thruster
	let thruster = app
		.world
		.spawn(Thruster::new_with_strength_factor(strength_factor))
		.id();

	// ticks once
	app.world.run_schedule(Main);

	let thruster = app.world.entity_mut(thruster);
	assert_eq!(thruster.get::<InternalForce>().unwrap().inner(), Vec3::ZERO);
	assert_eq!(
		thruster.get::<Thruster>().unwrap().get_strength_factor(),
		strength_factor
	);
}
