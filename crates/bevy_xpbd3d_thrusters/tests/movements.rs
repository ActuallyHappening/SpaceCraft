mod utils;
use utils::*;

#[test]
fn thruster_moves_parent() {
	// initialization
	let mut app = test_app();

	// static parameters
	// facing forwards, so pushing backwards
	let thruster_direction = Quat::IDENTITY;

	// random parameters
	let strength_factor = random::<f32>().abs() + 1.0;
	let parent_transform = Transform::from_xyz(random(), random(), random());
	let child_transform =
		Transform::from_xyz(random(), random(), random()).with_rotation(thruster_direction);

	// spawn parent
	let mut parent = app.world.spawn((parent_transform,));

	// spawn child
	parent.with_children(|parent| {
		parent.spawn((
			child_transform,
			Thruster::new_with_strength_factor(strength_factor)
				.set_current_status(1.0)
				.clone(),
		));
	});
	let parent = parent.id();

	// tick once
	app.world.run_schedule(Main);

	// extract entities
	let parent = app.world.entity_mut(parent);
	let child = *parent.get::<Children>().unwrap().first().unwrap();
	let parent = parent.id();

	// assertions
	// assert child doesn't move at all
	assert_eq!(
		*app.world.entity_mut(child).get::<Transform>().unwrap(),
		child_transform
	);
	// assert parent was pushed 'forward' into the screen
	assert!(*app.world.entity_mut(parent).get::<Transform>().unwrap().translation.z > 0.0);
}
