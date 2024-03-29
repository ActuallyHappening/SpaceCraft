use crate::prelude::*;

pub use blueprint::ThrusterBlockBlueprint;

use super::PlayerMovementSet;

// Plugin
pub struct ThrusterPlugin;

impl Plugin for ThrusterPlugin {
	fn build(&self, app: &mut App) {
		app
			.register_type::<Thruster>()
			.add_systems(
				Blueprints,
				Self::spawn_thruster_visuals.in_set(BlueprintExpansion::ThrusterBlocks),
			)
			.add_systems(
				GameLogic,
				((
					Self::sync_thruster_data,
					Self::sync_thruster_with_internal_forces,
				)
					.chain()
					.in_set(PlayerMovementSet::EnactThrusters),),
			)
			.add_systems(Update, Self::sync_thruster_with_visuals);
	}
}

/// Component for all thrusters (on a player)
#[derive(Debug, Component, Reflect, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(super) struct Thruster {
	block_id: BlockId,

	#[inspector(min = 0.0)]
	strength_factor: f32,

	/// Between 0..=1, synced with visuals and physics
	#[inspector(min = 0.0, max = 1.0)]
	current_status: f32,
}

impl Thruster {
	/// Get the current status of the thruster, i.e. factor between 0..=1 of how strongly it is firing.
	pub fn get_status(&self) -> f32 {
		self.current_status.clamp(0., 1.)
	}

	pub fn set_status(&mut self, status: f32) {
		self.current_status = status.clamp(0., 1.);
	}
}

impl GetBlockId for Thruster {
	fn get_block_id(&self) -> BlockId {
		self.block_id
	}
}

mod bundle {

	use bevy_xpbd_3d_parenting::InternalForce;

	use crate::prelude::*;

	use super::{blueprint::ThrusterBlockBlueprint, Thruster};

	/// Thruster that is spawned into the world
	#[derive(Bundle)]
	pub struct ThrusterBlockBundle {
		pbr: PbrBundle,
		collider: AsyncCollider,
		// body: RigidBody,
		name: Name,
		thruster: Thruster,
		internal_force: InternalForce,
	}

	impl Blueprint for BlockBlueprint<ThrusterBlockBlueprint> {
		type Bundle = ThrusterBlockBundle;
		type StampSystemParam<'w, 's> = MMA<'w>;

		fn stamp(&self, mma: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle {
			let BlockBlueprint {
				transform,
				mesh,
				material,
				specific_marker,
			} = self;
			Self::Bundle {
				pbr: PbrBundle {
					transform: *transform,
					mesh: mesh.clone().into_mesh(mma),
					material: material.clone().into_material(&mut mma.mats),
					..default()
				},
				collider: AsyncCollider(ComputedCollider::ConvexHull),
				// body: RigidBody::Dynamic,
				name: Name::new("ThrusterBlock"),
				thruster: specific_marker.clone().into(),
				internal_force: InternalForce(Vec3::Z),
			}
		}
	}
}

mod systems {
	use bevy_xpbd_3d_parenting::InternalForce;

	use crate::{players::player_movement::GetThrusterData, prelude::*};

	use super::{Thruster, ThrusterPlugin};

	impl ThrusterPlugin {
		/// Reads data from [GetThrusterData], and applies it to the [Thruster]s.
		pub(super) fn sync_thruster_data(
			data: GetThrusterData,
			mut thrusters: Query<(Entity, &mut Thruster)>,
		) {
			let thrusters_by_id: HashMap<BlockId, Entity> = thrusters
				.iter_mut()
				.map(|(e, thruster)| (thruster.get_block_id(), e))
				.collect();

			for (block_id, strength) in data.get_all().iter() {
				if let Some(thruster) = thrusters_by_id.get(*block_id) {
					thrusters
						.get_mut(*thruster)
						.unwrap()
						.1
						.set_status(**strength);
				} else {
					warn!("Thruster with id {:?} not found", block_id);
				}
			}
		}

		/// Reads data from [Thruster]s, and syncs visuals with particle effects
		pub(super) fn sync_thruster_with_visuals(
			thrusters: Query<(&Children, &Thruster)>,
			mut particle_effects: Query<(&mut EffectProperties, &mut EffectSpawner)>,
		) {
			for (thrusters, thrust) in thrusters.iter() {
				for thruster in thrusters {
					if let Ok((properties, mut spawner)) = particle_effects.get_mut(*thruster) {
						EffectProperties::set_if_changed(
							properties,
							Self::LIFETIME_ATTR,
							thrust.current_status.clamp(0., 1.).into(),
						);
						if thrust.current_status <= 0. {
							spawner.set_active(false);
						} else {
							spawner.set_active(true);
						}
					}
				}
			}
		}

		/// Reads data from [Thruster]s, and applies it to the physics simulation
		pub(super) fn sync_thruster_with_internal_forces(
			mut thrusters: Query<(&Thruster, &mut InternalForce)>,
		) {
			for (thruster, mut internal_force) in thrusters.iter_mut() {
				internal_force.set(Vec3::Z * thruster.get_status() * thruster.strength_factor);
			}
		}

		/// Can change, typically stays constant
		const ACCELERATION_ATTR: &str = "dynamic_accel";
		/// Makes a visual difference in the colour and range of the particles.
		/// Between 0 (for no lifetime) and 1 (for full lifetime)
		const LIFETIME_ATTR: &str = "dynamic_lifetime";

		pub(super) fn spawn_thruster_visuals(
			added_thrusters: Query<Entity, (With<Thruster>, Added<FreshlyExpanded>)>,
			mut commands: Commands,
			mut effects: ResMut<Assets<EffectAsset>>,
		) {
			for thruster in added_thrusters.iter() {
				trace!("Expanding thruster visual");
				commands
					.entity(thruster)
					.despawn_descendants()
					.with_children(|parent| {
						// debug!("Spawning thruster visuals");

						let mut color_gradient = Gradient::new();
						color_gradient.add_key(0.0, Vec4::splat(1.0));
						color_gradient.add_key(0.4, Vec4::new(1.0, 1.0, 0.0, 1.0));
						color_gradient.add_key(0.7, Vec4::new(1.0, 0.0, 0.0, 1.0));
						color_gradient.add_key(1.0, Vec4::new(0.2, 0., 0., 1.));

						let mut size_gradient = Gradient::new();
						size_gradient.add_key(0.0, Vec2::splat(0.1));
						size_gradient.add_key(0.5, Vec2::splat(0.5));
						size_gradient.add_key(1.0, Vec2::splat(0.08));

						let writer = ExprWriter::new();

						let age = (writer.lit(1.) - writer.prop(Self::LIFETIME_ATTR)).expr();
						let init_age1 = SetAttributeModifier::new(Attribute::AGE, age);

						let lifetime = writer.lit(1.).expr();
						let init_lifetime1 = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

						// Add constant downward acceleration to simulate gravity
						// let accel1 = writer1.lit(Vec3::Y * -3.).expr();
						// let update_accel1 = AccelModifier::new(accel1);

						let init_pos1 = SetPositionCone3dModifier {
							base_radius: writer.lit(PIXEL_SIZE * 0.1).expr(),
							top_radius: writer.lit(PIXEL_SIZE * 0.7).expr(),
							height: writer.lit(PIXEL_SIZE * 2.).expr(),
							dimension: ShapeDimension::Volume,
						};

						let init_vel1 = SetVelocitySphereModifier {
							center: writer.lit(Vec3::ZERO).expr(),
							speed: writer.prop(Self::ACCELERATION_ATTR).expr(),
						};

						let effect = effects.add(
							EffectAsset::new(
								32768,
								Spawner::rate(500.0.into()).with_starts_active(false),
								writer.finish(),
							)
							.with_name("emit:rate")
							.with_property(Self::ACCELERATION_ATTR, Value::from(PIXEL_SIZE * 10.))
							.with_property(Self::LIFETIME_ATTR, Value::from(0.5))
							// .with_property("my_accel", Vec3::new(0., -3., 0.).into())
							.init(init_pos1)
							// Make spawned particles move away from the emitter origin
							.init(init_vel1)
							.init(init_age1)
							.init(init_lifetime1)
							// .update(update_accel1)
							.render(ColorOverLifetimeModifier {
								gradient: color_gradient,
							})
							.render(SizeOverLifetimeModifier {
								gradient: size_gradient,
								screen_space_size: false,
							})
							.render(OrientModifier {
								mode: OrientMode::ParallelCameraDepthPlane,
							}),
						);

						parent.spawn((
							Name::new("Thruster Visuals"),
							ParticleEffectBundle {
								effect: ParticleEffect::new(effect),
								transform: Transform::from_rotation(Quat::from_rotation_x(-TAU / 4.)),
								..default()
							},
							EffectProperties::default(),
						));
					});
			}
		}
	}
}

mod blueprint {
	use crate::prelude::*;

	use super::Thruster;

	/// Builder for [ThrusterBlockBundle].
	///
	/// Will spawn a particle emitter as a child
	#[derive(Debug, Reflect, Serialize, Deserialize, Clone)]
	pub struct ThrusterBlockBlueprint {
		id: BlockId,
		strength: f32,
	}

	impl ThrusterBlockBlueprint {
		pub fn new() -> Self {
			Self {
				id: BlockId::random(),
				strength: 10.,
			}
		}
	}

	impl GetBlockId for ThrusterBlockBlueprint {
		fn get_block_id(&self) -> BlockId {
			self.id
		}
	}

	impl From<ThrusterBlockBlueprint> for Thruster {
		fn from(ThrusterBlockBlueprint { id, strength }: ThrusterBlockBlueprint) -> Self {
			Thruster {
				block_id: id,
				strength_factor: strength,
				current_status: 0.,
			}
		}
	}

	impl BlockBlueprint<ThrusterBlockBlueprint> {
		pub fn new_thruster(
			location: impl Into<manual_builder::RelativePixel>,
			facing: impl Into<Quat>,
		) -> Self {
			let rotation = facing.into();
			BlockBlueprint {
				transform: Transform {
					translation: location.into().into_world_offset()
						- Transform::from_rotation(rotation).forward() * PIXEL_SIZE / 2.,
					rotation,
					..default()
				},
				mesh: OptimizableMesh::CustomRectangularPrism {
					size: Vec3::splat(PIXEL_SIZE / 2.),
				},
				material: OptimizableMaterial::OpaqueColour(Color::RED),
				specific_marker: ThrusterBlockBlueprint::new(),
			}
		}
	}
}
