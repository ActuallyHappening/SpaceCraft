use bevy_xpbd3d_parenting::InternalForce;

use crate::prelude::*;

use crate::blocks::{manual_builder, BlockBlueprint};

use super::ControllablePlayer;

// Plugin
pub struct ThrusterPlugin;

impl Plugin for ThrusterPlugin {
	fn build(&self, app: &mut App) {
		app
			.register_type::<Thruster>()
			.add_systems(
				FixedUpdate,
				((Self::spawn_thruster_visuals)
					.in_set(BlueprintExpansion::Thruster),),
			)
			.add_systems(Update, Self::sync_thruster_with_visuals);
	}
}

/// Component for all thrusters (on a player)
#[derive(Debug, Component, Reflect, Clone)]
struct Thruster {
	id: BlockId,
	strength_factor: f32,
	/// Between 0..=1, synced with visuals and physics
	pub current_status: f32,
}

#[derive(Bundle)]
pub struct ThrusterBlockBundle {
	pbr: PbrBundle,
	collider: AsyncCollider,
	// body: RigidBody,
	name: Name,
	thruster: Thruster,
	internal_force: InternalForce,
}

impl ThrusterPlugin {
	fn sync_thruster_with_visuals(
		thrusters: Query<(&Children, &Thruster)>,
		mut particle_effects: Query<(&mut CompiledParticleEffect, &mut EffectSpawner)>,
	) {
		for (thrusters, thrust) in thrusters.iter() {
			for thruster in thrusters {
				if let Ok((mut particles, mut spawner)) = particle_effects.get_mut(*thruster) {
					particles.set_property(
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

	/// Can change, typically stays constant
	const ACCELERATION_ATTR: &str = "dynamic_accel";
	/// Makes a visual difference in the colour and range of the particles.
	/// Between 0 (for no lifetime) and 1 (for full lifetime)
	const LIFETIME_ATTR: &str = "dynamic_lifetime";

	fn spawn_thruster_visuals(
		added_thrusters: Query<Entity, Added<Thruster>>,
		mut commands: Commands,
		mut effects: ResMut<Assets<EffectAsset>>,
	) {
		for thruster in added_thrusters.iter() {
			commands.entity(thruster).with_children(|parent| {
				debug!("Spawning thruster visuals");

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
						transform: Transform::from_rotation(Quat::from_rotation_x(TAU / 4.)),
						..default()
					},
				));
			});
		}
	}
}

// #region block

/// Will spawn a particle emitter as a child
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThrusterBlock {
	id: BlockId,
	strength: f32,
}

impl ThrusterBlock {
	pub fn new() -> Self {
		Self {
			id: BlockId::random(),
			strength: 0.5,
		}
	}
}

impl From<ThrusterBlock> for Thruster {
	fn from(ThrusterBlock { id, strength }: ThrusterBlock) -> Self {
		Thruster {
			id,
			strength_factor: strength,
			current_status: 0.5,
		}
	}
}

impl FromBlueprint for ThrusterBlockBundle {
	type Blueprint = BlockBlueprint<ThrusterBlock>;

	fn stamp_from_blueprint(
		BlockBlueprint {
			transform,
			mesh,
			material,
			specific_marker,
		}: &Self::Blueprint,
		mma: &mut MMA,
	) -> Self {
		Self {
			pbr: PbrBundle {
				transform: *transform,
				mesh: mesh.get_mesh(mma),
				material: material.get_material(&mut mma.mats),
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

impl BlockBlueprint<ThrusterBlock> {
	pub fn new_thruster(location: manual_builder::RelativePixel, facing: impl Into<Quat>) -> Self {
		let rotation = facing.into();
		BlockBlueprint {
			transform: Transform {
				translation: location.as_vec3() * PIXEL_SIZE
					+ Transform::from_rotation(rotation).forward() * PIXEL_SIZE / 2.,
				rotation,
				..default()
			},
			mesh: OptimizableMesh::CustomRectangularPrism {
				size: Vec3::splat(PIXEL_SIZE / 2.),
			},
			material: OptimizableMaterial::OpaqueColour(Color::RED),
			specific_marker: ThrusterBlock::new(),
		}
	}
}

// #endregion
