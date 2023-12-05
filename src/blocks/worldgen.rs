use crate::prelude::*;

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(FixedUpdate, Self::expand_terrain_structure)
			.add_systems(WorldCreation, Self::creation_spawn_random_world);
	}
}

mod systems {

	use crate::prelude::*;

	use super::{
		discrete_shapes::{DiscreteSphere, OptimizableDiscreteShape},
		terrain_blueprint::TerrainStructureBlueprint,
		terrain_bundle::{TerrainItemBundle, TerrainStructureBundle},
		terrain_type::TerrainType,
		WorldGenPlugin,
	};

	impl WorldGenPlugin {
		pub(super) fn expand_terrain_structure(
			mut blueprints: Query<(Entity, &TerrainStructureBlueprint), Added<TerrainStructureBlueprint>>,
			mut commands: Commands,
			mut mma: MMA,
		) {
			for (entity, blueprint) in blueprints.iter_mut() {
				commands
					.entity(entity)
					.insert(TerrainStructureBundle::stamp_from_blueprint(
						blueprint, &mut mma,
					))
					.with_children(|parent| {
						for child in blueprint.clone().into_children().iter() {
							parent.spawn(TerrainItemBundle::stamp_from_blueprint(child, &mut mma));
						}
					});
			}
		}

		pub(super) fn creation_spawn_random_world(mut commands: Commands) {
			debug!("Spawning initial asteroids");

			let mut rng = rand::thread_rng();
			let structures: Vec<TerrainStructureBlueprint> = (0..10)
				.map(|_| {
					let pos = vec3_polar_random(&mut rng);
					let distance = 20.0..30.0;
					let pos = pos * rng.gen_range(distance);

					let rot = Quat::from_euler(
						EulerRot::XYZ,
						rng.gen_range(0. ..=TAU),
						rng.gen_range(0. ..=TAU),
						rng.gen_range(0. ..=TAU),
					);

					let mut r = |bound: f32| rng.gen_range(-bound .. bound);
					let max_linvel = 1.0;
					let linvel = LinearVelocity(Vec3::new(r(max_linvel), r(max_linvel), r(max_linvel)));
					let max_angvel = 0.5;
					let angvel = AngularVelocity(Vec3::new(r(max_angvel), r(max_angvel), r(max_angvel)));

					TerrainStructureBlueprint {
						transform: Transform::from_translation(pos).with_rotation(rot),
						initial_velocity: Some((linvel, angvel)),
						shape: OptimizableDiscreteShape::Sphere(DiscreteSphere {
							radius: NonZeroU8::new(rng.gen_range(1..=4)).unwrap(),
						}),
						terrain_type: TerrainType::SilicateRock,
					}
				})
				.collect();

			commands.spawn_batch(structures);
		}
	}

	#[test]
	fn test_world_gen_expands() {
		let mut app = test_app();
		app.add_plugins(super::WorldGenPlugin);

		app.world.spawn(TerrainStructureBlueprint::default());
		fn assert_0_item(items: Query<(), (With<Name>, With<Transform>, With<Handle<Mesh>>)>) {
			assert_eq!(items.iter().count(), 0);
		}
		app.world.run_system_once(assert_0_item);

		app.world.run_schedule(FixedUpdate);

		fn assert_1_item(items: Query<(), (With<Name>, With<Transform>, With<Handle<Mesh>>)>) {
			assert_eq!(items.iter().count(), 1);
		}
		app.world.run_system_once(assert_1_item);
	}
}

mod terrain_blueprint {
	use crate::{blocks::manual_builder::RelativePixel, prelude::*};

	use super::{
		discrete_shapes::{DiscreteLocations, OptimizableDiscreteShape},
		terrain_type::TerrainType,
	};

	/// Blueprint for [TerrainItemBundle]
	#[derive(Serialize, Deserialize, Debug, Reflect, Clone)]
	pub struct TerrainItemBlueprint {
		pub terrain_type: TerrainType,
		pub location: RelativePixel,
	}

	/// Blueprint for [TerrainStructureBundle]
	#[derive(Component, Serialize, Deserialize, Debug, Default, Reflect, Clone)]
	#[reflect(Component)]
	pub struct TerrainStructureBlueprint {
		pub transform: Transform,
		pub initial_velocity: Option<(LinearVelocity, AngularVelocity)>,
		pub shape: OptimizableDiscreteShape,
		pub terrain_type: TerrainType,
	}

	impl TerrainStructureBlueprint {
		pub(super) fn into_children(self) -> Vec<TerrainItemBlueprint> {
			self
				.shape
				.get_locations()
				.into_iter()
				.map(|location| TerrainItemBlueprint {
					terrain_type: self.terrain_type.clone(),
					location,
				})
				.collect()
		}
	}
}

mod discrete_shapes {
	use crate::prelude::*;
	use std::num::NonZeroU8;

	use crate::blocks::manual_builder::RelativePixel;

	#[derive(Debug, Serialize, Deserialize, Reflect, Default, Clone)]
	pub enum OptimizableDiscreteShape {
		Sphere(DiscreteSphere),
		#[default]
		Dot,
	}

	#[derive(Debug, Serialize, Deserialize, Reflect, Clone)]
	pub struct DiscreteSphere {
		pub radius: NonZeroU8,
	}

	pub trait DiscreteLocations {
		fn get_locations(self) -> HashSet<RelativePixel>;
	}

	impl DiscreteLocations for DiscreteSphere {
		fn get_locations(self) -> HashSet<RelativePixel> {
			let mut locations = HashSet::new();
			let DiscreteSphere { radius } = self;
			let radius = radius.get() as i32;
			for x in -radius..=radius {
				for y in -radius..=radius {
					for z in -radius..=radius {
						let offset = IVec3::new(x, y, z);
						if offset.as_vec3().length() <= radius as f32 {
							locations.insert(RelativePixel(offset));
						}
					}
				}
			}
			locations
		}
	}

	impl DiscreteLocations for OptimizableDiscreteShape {
		fn get_locations(self) -> HashSet<RelativePixel> {
			match self {
				Self::Dot => [RelativePixel::default()].into_iter().collect(),
				Self::Sphere(sphere) => sphere.get_locations(),
			}
		}
	}
}

mod terrain_bundle {
	use crate::prelude::*;

	use super::terrain_blueprint::{TerrainItemBlueprint, TerrainStructureBlueprint};

	/// A single terrain unit
	#[derive(Bundle)]
	pub struct TerrainItemBundle {
		pbr: PbrBundle,
		name: Name,
		collider: AsyncCollider,
	}

	impl FromBlueprint for TerrainItemBundle {
		type Blueprint = TerrainItemBlueprint;

		fn stamp_from_blueprint(
			TerrainItemBlueprint {
				terrain_type,
				location,
			}: &Self::Blueprint,
			mma: &mut MMA,
		) -> Self {
			Self {
				pbr: PbrBundle {
					transform: Transform::from_translation(location.into_world_offset()),
					mesh: terrain_type.mesh().into_mesh(mma),
					material: terrain_type.material().into_material(&mut mma.mats),
					..default()
				},
				name: Name::new(terrain_type.name()),
				collider: AsyncCollider(ComputedCollider::TriMesh),
			}
		}
	}

	/// The parent of many [TerrainItemBundle]s
	#[derive(Bundle)]
	pub struct TerrainStructureBundle {
		spatial: SpatialBundle,
		name: Name,
		rigid_body: RigidBody,
		linvel: LinearVelocity,
		angvel: AngularVelocity,
		mass_properties: MassPropertiesBundle,
	}

	impl FromBlueprint for TerrainStructureBundle {
		type Blueprint = TerrainStructureBlueprint;

		fn stamp_from_blueprint(
			TerrainStructureBlueprint { transform, initial_velocity, .. }: &Self::Blueprint,
			_mma: &mut MMA,
		) -> Self {
			let linvel = initial_velocity
				.as_ref()
				.map(|(linvel, _)| *linvel)
				.unwrap_or_default();

			let angvel = initial_velocity
				.as_ref()
				.map(|(_, angvel)| *angvel)
				.unwrap_or_default();

			Self {
				spatial: SpatialBundle::from_transform(*transform),
				name: Name::new("TerrainStructure"),
				rigid_body: RigidBody::Dynamic,
				linvel,
				angvel,
				mass_properties: MassPropertiesBundle::new_computed(&Collider::ball(1.0), 1.0),
			}
		}
	}
}

mod terrain_type {
	use crate::{blocks::manual_builder::RelativePixel, prelude::*};

	#[derive(
		Debug, Serialize, Deserialize, PartialEq, Eq, Hash, IntoStaticStr, Reflect, Clone, Default,
	)]
	pub enum TerrainType {
		#[default]
		SilicateRock,
	}

	type TT = TerrainType;

	impl TerrainType {
		pub(super) const fn mesh(&self) -> OptimizableMesh {
			match self {
				TT::SilicateRock => OptimizableMesh::CustomRectangularPrism {
					size: Vec3::splat(PIXEL_SIZE),
				},
			}
		}

		pub(super) fn material(&self) -> OptimizableMaterial {
			match self {
				TT::SilicateRock => OptimizableMaterial::OpaqueColour(Color::rgb_u8(84, 84, 84)),
			}
		}

		pub fn name(&self) -> &'static str {
			self.into()
		}
	}
}
