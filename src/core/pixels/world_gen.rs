use crate::utils::*;
use num_integer::Roots;

#[derive(Debug, EnumIs, EnumDiscriminants, Serialize, Deserialize, Clone, Copy)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorldObjectTypes))]
pub enum WorldObjectType {
	Asteroid { approx_radius: NonZeroU8 },
}

#[derive(Debug, Component, Serialize, Deserialize, Constructor)]
pub struct SpawnWorldObject {
	obj_type: WorldObjectType,
}

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
	pub fn generate_structure(&self) -> Structure {
		match self {
			Self::Asteroid { approx_radius } => Structure::new(points_in_circle(approx_radius).map(
				|p| StructurePart::Pixel {
					px: Pixel::default_from_variant(PixelVariant::Copper),
					relative_location: p,
				},
			)),
		}
	}
}

pub fn spawn_authoritative_initial_world(mut commands: Commands) {
	let mut rng = rand::thread_rng();

	for _ in 0..25 {
		let pos = random_pos(SpaceRegions::VisibleNotInsidePlayer);
		let rot = Quat::from_euler(
			EulerRot::XYZ,
			rng.gen_range(0. ..=TAU),
			rng.gen_range(0. ..=TAU),
			rng.gen_range(0. ..=TAU),
		);
		let velocity = random_velocity(VelocityRanges::Slow);

		let object_type = WorldObjectType::Asteroid {
			approx_radius: NonZeroU8::new(rng.gen_range(1..=4)).unwrap(),
		};

		commands.spawn(
			(
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
				SpawnWorldObject::new(object_type),
			)
				.physics_dynamic()
				.named("WorldGen Asteroid Structure"),
		);
	}
}

pub fn hydrate_spawn_world_object(
	mut commands: Commands,
	world_objects: Query<(Entity, &SpawnWorldObject), Added<SpawnWorldObject>>,
) {
	for (entity, spawn_world_object) in world_objects.iter() {
		debug!("Hydrating world object");

		let structure = spawn_world_object.obj_type.generate_structure();
		let collider = structure.compute_collider();

		let mut entity = commands.entity(entity);
		entity.insert(collider);
		entity.insert(SpawnChildStructure::new(structure));
	}
}
