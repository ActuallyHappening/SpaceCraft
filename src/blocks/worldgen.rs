use crate::prelude::*;

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(FixedUpdate, Self::expand_terrain_structure);
	}
}

mod systems {
	use bevy::winit::WinitPlugin;

	use crate::{blocks::manual_builder::RelativePixel, prelude::*};

	use super::{
		terrain_blueprint::TerrainStructureBlueprint,
		terrain_bundle::{TerrainItemBundle, TerrainStructureBundle},
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
						for child in blueprint.children.iter() {
							parent.spawn(TerrainItemBundle::stamp_from_blueprint(child, &mut mma));
						}
					});
			}
		}
	}

	#[test]
	fn test_world_gen_expands() {
		let mut app = App::new();

		app.add_plugins((
			DefaultPlugins.build().disable::<WinitPlugin>(),
			// MinimalPlugins,
			super::WorldGenPlugin,
		));

		let spawn_location = RelativePixel(IVec3::new(0, 0, 0));

		app.world.spawn(TerrainStructureBlueprint::new_homogenous(
			Transform::default(),
			super::discrete_shapes::OptimizableDiscreteShape::Dot {
				pos: spawn_location,
			},
			super::terrain_type::TerrainType::SilicateRock,
		));

		app.world.run_system_once(assert_0_item);
		app.world.run_schedule(FixedUpdate);

		fn assert_1_item(items: Query<(), (With<Name>, With<Transform>, With<Handle<Mesh>>)>) {
			assert_eq!(items.iter().count(), 1);
		}
		fn assert_0_item(items: Query<(), (With<Name>, With<Transform>, With<Handle<Mesh>>)>) {
			assert_eq!(items.iter().count(), 0);
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
	#[derive(Serialize, Deserialize, Debug, Reflect)]
	pub struct TerrainItemBlueprint {
		pub terrain_type: TerrainType,
		pub location: RelativePixel,
	}

	/// Blueprint for [TerrainStructureBundle]
	#[derive(Component, Serialize, Deserialize, Debug, Default, Reflect)]
	#[reflect(Component)]
	pub struct TerrainStructureBlueprint {
		pub transform: Transform,
		pub(super) children: Vec<TerrainItemBlueprint>,
	}

	impl TerrainStructureBlueprint {
		pub fn new_homogenous(
			transform: Transform,
			shape: OptimizableDiscreteShape,
			terrain_type: TerrainType,
		) -> Self {
			let children = shape
				.get_locations()
				.into_iter()
				.map(|location| TerrainItemBlueprint {
					terrain_type: terrain_type.clone(),
					location,
				})
				.collect();
			Self {
				transform,
				children,
			}
		}
	}
}

mod discrete_shapes {
	use crate::prelude::*;
	use std::num::NonZeroU8;

	use crate::blocks::manual_builder::RelativePixel;

	#[derive(Debug, Serialize, Deserialize)]
	pub enum OptimizableDiscreteShape {
		Sphere(DiscreteSphere),
		Dot { pos: RelativePixel },
	}

	#[derive(Debug, Serialize, Deserialize)]
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
				Self::Dot { pos } => [pos].into_iter().collect(),
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
	}

	impl FromBlueprint for TerrainStructureBundle {
		type Blueprint = TerrainStructureBlueprint;

		fn stamp_from_blueprint(
			TerrainStructureBlueprint { transform, .. }: &Self::Blueprint,
			_mma: &mut MMA,
		) -> Self {
			Self {
				spatial: SpatialBundle::from_transform(*transform),
				name: Name::new("TerrainStructure"),
			}
		}
	}
}

mod terrain_type {
	use crate::{blocks::manual_builder::RelativePixel, prelude::*};

	#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, IntoStaticStr, Reflect, Clone)]
	pub enum TerrainType {
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
