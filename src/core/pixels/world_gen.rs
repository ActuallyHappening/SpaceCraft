use crate::utils::*;
use num_integer::Roots;

#[derive(Debug, Component, EnumIs, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorldObjectTypes))]
pub enum WorldObjectType {
	Asteroid { approx_radius: NonZeroU8 },
}

// #[extension(trait Vec3Ext)]
// impl Vec3 {
// 	fn in_circle(self, circle_radius: f32) -> bool {
// 		(self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt() <= circle_radius
// 	}
// }

impl RelativePixelPoint {
	fn in_circle(&self, radius: &NonZeroU8) -> bool {
		((self.x.pow(2) + self.y.pow(2) + self.z.pow(2)) as f32).sqrt() <= radius.get() as f32
	}
}

fn points_in_circle(radius: &NonZeroU8) -> impl Iterator<Item = RelativePixelPoint> + '_ {
	let (min, max) = (-(radius.get() as i32), radius.get() as i32);
	(min..=max)
		.flat_map(move |x| {
			(min..=max).flat_map(move |y| (min..=max).map(move |z| RelativePixelPoint::new(x, y, z)))
		})
		.filter(move |p| p.in_circle(radius))
}

impl WorldObjectType {
	pub fn generate_structure(self) -> Structure {
		match self {
			Self::Asteroid { approx_radius } => Structure::new(points_in_circle(&approx_radius).map(
				|p| StructurePart::Pixel {
					px: PixelVariant::Copper.get_default_pixel(),
					relative_location: p,
				},
			)),
		}
	}
}

pub fn spawn_initial_world(mut commands: Commands) {
	let mut rng = rand::thread_rng();

	for _ in 0..10 {
		let pos = random_pos(SpaceRegions::VisibleNotInsidePlayer);
		let rot = Quat::from_euler(
			EulerRot::XYZ,
			rng.gen_range(0. ..=TAU),
			rng.gen_range(0. ..=TAU),
			rng.gen_range(0. ..=TAU),
		);
		let velocity = random_velocity(VelocityRanges::Slow);

		let object_type = WorldObjectType::Asteroid {
			approx_radius: NonZeroU8::new(rng.gen_range(1..=10)).unwrap(),
		};

		let structure = object_type.generate_structure();

		let collider = structure.compute_collider();

		commands.spawn(
			(
				collider,
				velocity,
				PbrBundle {
					transform: Transform {
						translation: pos,
						rotation: rot,
						scale: Vec3::ONE,
					},
					..default()
				},
				Replication,
				SpawnChildStructure::new(structure),
			)
				.physics_dynamic().named("WorldGen Asteroid Structure"),
		);
		// .with_children(|parent| {
		// 	bundles
		// 		.into_iter()
		// 		.for_each(|b| b.default_spawn_to_parent(parent));
		// });
	}
}
