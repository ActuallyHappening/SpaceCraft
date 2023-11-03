use crate::utils::*;

pub fn test(
	// mut commands: Commands,
	mut effects: ResMut<Assets<EffectAsset>>,
	// mut meshes: ResMut<Assets<Mesh>>,
	// mut materials: ResMut<Assets<StandardMaterial>>,
) -> ParticleEffectBundle {
	// let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
	// let mat = materials.add(Color::PURPLE.into());

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
		EffectAsset::new(32768, Spawner::rate(500.0.into()), writer1.finish())
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

	// commands
		// .spawn((
			// Name::new("emit:rate"),
			ParticleEffectBundle {
				effect: ParticleEffect::new(effect1),
				transform: Transform::from_translation(Vec3::ZERO)
					.with_rotation(Quat::from_rotation_z(0f32.to_radians())),
				..Default::default()
			}
		// ))
		// .with_children(|p| {
			// Reference cube to visualize the emit origin
			// p.spawn((
				// PbrBundle {
					// mesh: cube.clone(),
					// material: mat.clone(),
					// ..Default::default()
				// },
				// Name::new("source"),
			// ));
		// });
}

// pub fn test(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
//   let mut color_gradient1 = Gradient::new();
//     color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
//     color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
//     color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
//     color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

//     let mut size_gradient1 = Gradient::new();
//     size_gradient1.add_key(0.0, Vec2::splat(0.1));
//     size_gradient1.add_key(0.3, Vec2::splat(0.1));
//     size_gradient1.add_key(1.0, Vec2::splat(0.0));

//     let writer = ExprWriter::new();

//     // Give a bit of variation by randomizing the age per particle. This will
//     // control the starting color and starting size of particles.
//     let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
//     let init_age = SetAttributeModifier::new(Attribute::AGE, age);

//     // Give a bit of variation by randomizing the lifetime per particle
//     let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
//     let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

//     // Add constant downward acceleration to simulate gravity
//     // let accel = writer.lit(Vec3::Y * -8.).expr();
//     // let update_accel = AccelModifier::new(accel);

//     // Add drag to make particles slow down a bit after the initial explosion
//     let drag = writer.lit(5.).expr();
//     let update_drag = LinearDragModifier::new(drag);

//     let init_pos = SetPositionSphereModifier {
//         center: writer.lit(Vec3::ZERO).expr(),
//         radius: writer.lit(2.).expr(),
//         dimension: ShapeDimension::Volume,
//     };

//     // Give a bit of variation by randomizing the initial speed
//     let init_vel = SetVelocitySphereModifier {
//         center: writer.lit(Vec3::ZERO).expr(),
//         speed: (writer.rand(ScalarType::Float) * writer.lit(20.) + writer.lit(60.)).expr(),
//     };

//     let effect = EffectAsset::new(
//         32768,
//         Spawner::burst(2500.0.into(), 2.0.into()),
//         writer.finish(),
//     )
//     .with_name("firework")
//     .init(init_pos)
//     .init(init_vel)
//     .init(init_age)
//     .init(init_lifetime)
//     .update(update_drag)
//     // .update(update_accel)
//     .render(ColorOverLifetimeModifier {
//         gradient: color_gradient1,
//     })
//     .render(SizeOverLifetimeModifier {
//         gradient: size_gradient1,
//         screen_space_size: false,
//     });

//     let effect1 = effects.add(effect);

//     commands.spawn((
//         Name::new("firework"),
//         ParticleEffectBundle {
//             effect: ParticleEffect::new(effect1),
//             transform: Transform::IDENTITY,
//             ..Default::default()
//         },
//     ));
// }
