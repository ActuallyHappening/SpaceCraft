use std::ops::Mul;

use crate::utils::*;

#[derive(Component, Debug, Clone)]
pub struct Star {
	/// Smaller, the lgner the blink durationl
	blink_speed: f32,
	/// Between [0, 1] for 0 = no blinking at all and 1 = full blinking
	blink_strength: f32,
}

pub fn spawn_random_star(commands: &mut Commands, MMA { meshs, mats, .. }: &mut MMA) {
	// commands
	// 	.spawn(PointLightBundle {
	// 		point_light: PointLight {
	// 			color: random_color(),
	// 			intensity: 1e20,
	// 			range: 1e20,
	// 			// radius: ,
	// 			..default()
	// 		},
	// 		transform: Transform::from_translation(random_pos()),
	// 		..default()
	// 	})
	// 	.with_children(|commands| {
	commands.spawn(
		PbrBundle {
			material: mats.add(StandardMaterial {
				base_color: Color::WHITE.with_a(0.1),
				emissive: Color::WHITE,
				unlit: true,
				alpha_mode: AlphaMode::Add,
				..default()
			}),
			mesh: meshs.add(
				shape::Icosphere {
					radius: 150.0,
					subdivisions: 4,
				}
				.try_into()
				.unwrap(),
			),
			transform: Transform::from_translation(random_pos(SpaceRegions::FarAway)),
			..default()
		}
		.insert(Star::new())
		.named("Star"),
	);
	// });
}

// fn random_color() -> Color {
// 	Color::rgb(random(), random(), random())
// }

#[derive(Default)]
pub struct StarMaterials(HashMap<u32, Handle<StandardMaterial>>);
const PRECISION: u32 = 1000;

pub fn blink_stars(
	mut mats: Local<StarMaterials>,
	mut stars: Query<(&Star, &mut Handle<StandardMaterial>)>,
	time: Res<Time>,
	mut mma: MM,
) {
	let seconds_since_start = time.elapsed_seconds_wrapped();
	for (star, mut mat) in stars.iter_mut() {
		let val = (seconds_since_start / (1. / star.blink_speed))
			.sin()
			.add(1.)
			.div(2.)
			.mul(star.blink_strength)
			.mul(PRECISION as f32)
			.round() as u32;

		// debug!("Val: {}", val);

		let handle = if let Some(handle) = mats.0.get(&val) {
			handle.clone()
		} else {
			let handle = mma.mats.add(StandardMaterial {
				base_color: Color::WHITE.with_a(val as f32 / PRECISION as f32),
				emissive: Color::WHITE,
				alpha_mode: AlphaMode::Add,
				unlit: true,
				..default()
			});
			mats.0.insert(val, handle.clone());
			handle
		};
		*mat = handle;
	}
}

impl Star {
	pub fn new() -> Star {
		let mut rng = rand::thread_rng();

		Star {
			blink_speed: rng.gen_range(0.01..0.5),
			blink_strength: rng.gen_range(0.1..1.),
		}
	}
}
