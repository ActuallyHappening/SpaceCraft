use crate::utils::*;

pub fn gen_particles(effects: &mut Assets<EffectAsset>) -> ParticleEffectBundle {
	let mut color_gradient1 = Gradient::new();
	color_gradient1.add_key(0.0, Vec4::splat(1.0));
	color_gradient1.add_key(0.1, Vec4::new(1.0, 1.0, 0.0, 1.0));
	color_gradient1.add_key(0.4, Vec4::new(1.0, 0.0, 0.0, 1.0));
	color_gradient1.add_key(1.0, Vec4::splat(0.0));

	let mut size_gradient1 = Gradient::new();
	size_gradient1.add_key(0.0, Vec2::splat(0.1));
	size_gradient1.add_key(0.5, Vec2::splat(0.5));
	size_gradient1.add_key(0.8, Vec2::splat(0.08));
	size_gradient1.add_key(1.0, Vec2::splat(0.0));

	let writer1 = ExprWriter::new();

	let age1 = writer1.lit(0.).expr();
	let init_age1 = SetAttributeModifier::new(Attribute::AGE, age1);

	let lifetime1 = writer1.lit(0.1).expr();
	let init_lifetime1 = SetAttributeModifier::new(Attribute::LIFETIME, lifetime1);

	// Add constant downward acceleration to simulate gravity
	// let accel1 = writer1.lit(Vec3::Y * -3.).expr();
	// let update_accel1 = AccelModifier::new(accel1);

	let init_pos1 = SetPositionCone3dModifier {
		base_radius: writer1.lit(0.1).expr(),
		top_radius: writer1.lit(0.7).expr(),
		height: writer1.lit(2.).expr(),
		dimension: ShapeDimension::Volume,
	};

	let init_vel1 = SetVelocitySphereModifier {
		center: writer1.lit(Vec3::ZERO).expr(),
		speed: writer1.lit(100.).expr(),
	};

	let effect1 = effects.add(
		EffectAsset::new(
			32768,
			Spawner::rate(500.0.into()).with_starts_active(false),
			writer1.finish(),
		)
		.with_name("emit:rate")
		// .with_property("my_accel", Vec3::new(0., -3., 0.).into())
		.init(init_pos1)
		// Make spawned particles move away from the emitter origin
		.init(init_vel1)
		.init(init_age1)
		.init(init_lifetime1)
		// .update(update_accel1)
		.render(ColorOverLifetimeModifier {
			gradient: color_gradient1,
		})
		.render(SizeOverLifetimeModifier {
			gradient: size_gradient1,
			screen_space_size: false,
		}),
	);

	ParticleEffectBundle {
		effect: ParticleEffect::new(effect1),
		transform: Transform::from_translation(Vec3::ZERO),
		..Default::default()
	}
}

#[allow(dead_code)]
pub fn test_activate_particles(
	mut q_spawner: Query<&mut EffectSpawner>,
	keyboard: Res<Input<KeyCode>>,
) {
	// Note: On first frame where the effect spawns, EffectSpawner is spawned during
	// CoreSet::PostUpdate, so will not be available yet. Ignore for a frame
	// if so.
	debug!("Checking for spawnertest start");
	for mut spawner in q_spawner.iter_mut() {
		if keyboard.pressed(KeyCode::P) {
			debug!("Setting spawner to true");
			spawner.set_active(true);
		} else {
			debug!("Setting spawner to false");
			spawner.set_active(false);
		}
	}
}
