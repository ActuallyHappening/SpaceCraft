use bevy_xpbd3d_thrusters::prelude::*;

fn assert_all_internal_forces_are_zero(internal_forces: Query<&InternalForce>) {
	for force in internal_forces.iter() {
		assert_eq!(force.inner(), Vec3::ZERO);
	}
}

#[test]
fn starts_with_no_internal_force() {
	let mut world = World::new();

	let thruster = world.spawn(Thruster::default()).id();

	world.run_system(assert_all_internal_forces_are_zero);
}