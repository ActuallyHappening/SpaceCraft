use crate::prelude::*;

pub struct BouncingBallsPlugin;

impl Plugin for BouncingBallsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup).replicate::<Transform>();
	}
}

fn setup(mut commands: Commands, mut mma: MMA) {
	// spawn plane on XZ plane
	commands.spawn((
		RigidBody::Static,
		Collider::cuboid(50., 0.02, 50.),
		PbrBundle {
			mesh: mma.meshs.add(shape::Plane::from_size(50.).into()),
			..default()
		},
	)).named("Plane");

	// ball that falls down because of gravity
	commands.spawn((
		Replication,
		RigidBody::Dynamic,
		AngularVelocity(Vec3::new(0., 0., 0.)),
		Collider::cuboid(1., 1., 1.),
		PbrBundle {
			mesh: mma.meshs.add(shape::Cube { size: 1. }.into()),
			material: mma.mats.add(Color::GREEN.into()),
			transform: Transform::from_translation(Vec3::new(0., 30., 0.)),
			..default()
		},
	)).named("Ball");
}
