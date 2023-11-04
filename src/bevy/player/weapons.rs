use crate::{
	bevy::netcode::{AuthoritativeUpdate, ClientUpdate},
	utils::*,
};

use super::ControllablePlayer;

const RADIUS: f32 = PIXEL_SIZE / 6.;
const LENGTH: f32 = PIXEL_SIZE;
const BULLET_SPEED: f32 = 50.;

pub struct PlayerWeaponsPlugin;
impl Plugin for PlayerWeaponsPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(
				FixedUpdate,
				(
					(
						// authoritative_handle_firing,
						authoritative_spawn_bullets,
						authoritative_hydrate_bullets,
						// authoritative_update_bullets,
					)
						.chain()
						.in_set(AuthoritativeUpdate),
					(send_player_fire_request,).in_set(ClientUpdate),
				),
			)
			.add_client_event::<PlayerFireInput>(EventType::Ordered);
	}
}

#[derive(Debug, Serialize, Deserialize, Event)]
struct PlayerFireInput;

/// Information about a weapon (entity), including cooldown
#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct WeaponInfo {
	/// Ticked every frame, and if it's finished, the weapon can fire
	cooldown: Duration,
	/// How long it takes to fire a bullet, [cooldown] is set to this upon firing
	fire_rate: Duration,

	/// Info about the bullets spawned by this weapon
	bullets_info: BulletInfo,
}

impl WeaponInfo {
	pub fn sensible_default() -> Self {
		WeaponInfo {
			bullets_info: BulletInfo {
				lifetime: Duration::from_secs(5),
			},
			// starts ready to fire
			cooldown: Duration::ZERO,

			fire_rate: Duration::from_millis(500),
		}
	}
}

/// Static  information about a bullet, used to spawn it
/// and found on actual bullets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletInfo {
	lifetime: Duration,
}

/// Hydrates to a full bullet, then is effectively removed from entities
#[derive(Component, Debug, Serialize, Deserialize)]
struct SpawnBullet {
	transform: Transform,
	info: BulletInfo,
	spawned_by: u64,
}

/// An actual bullet, logic uses this component
#[derive(Debug, Component)]
struct Bullet {
	ttl: Duration,
	spawned_by: u64,
}

fn send_player_fire_request(
	mouse: Res<Input<MouseButton>>,
	mut event_writer: EventWriter<PlayerFireInput>,
) {
	if mouse.just_pressed(MouseButton::Left) {
		event_writer.send(PlayerFireInput);
	}
}

/// What the authoritative server spawns.
/// Is then replicated, and finally hydrated on both authority and clients
#[derive(Bundle)]
struct AuthoritativeBulletBundle {
	physics: PhysicsBundle,
	to_spawn: SpawnBullet,

	name: Name,
	replication: Replication,
}

/// Client side visuals of bullet
#[derive(Bundle)]
struct ClientBulletBundle {
	bullet: Bullet,
	transform: Transform,

	computed_visibility: ComputedVisibility,
	visibility: Visibility,
	global_transform: GlobalTransform,
}

impl ClientBulletBundle {
	fn new(bullet: Bullet, transform: Transform) -> Self {
		ClientBulletBundle {
			bullet,
			transform,
			computed_visibility: ComputedVisibility::default(),
			visibility: Visibility::Inherited,
			global_transform: GlobalTransform::default(),
		}
	}

	/// The bullet mesh / PbrBundle
	fn get_child_pbr(mma: &mut MMA) -> PbrBundle {
		PbrBundle {
			transform: Transform::from_rotation(Quat::from_rotation_x(-TAU / 4.)),
			material: mma.mats.add(StandardMaterial {
				base_color: Color::RED,
				emissive: Color::RED,
				alpha_mode: AlphaMode::Add,
				unlit: true,
				perceptual_roughness: 0.,
				..default()
			}),
			mesh: mma.meshs.add(
				shape::Capsule {
					radius: RADIUS,
					depth: LENGTH,
					rings: 4,
					..default()
				}
				.into(),
			),
			..default()
		}
	}
}

impl AuthoritativeBulletBundle {
	fn new(transform: Transform, info: BulletInfo, spawned_by: u64) -> Self {
		AuthoritativeBulletBundle {
			physics: PhysicsBundle::new(&transform),
			to_spawn: SpawnBullet {
				transform,
				info,
				spawned_by,
			},
			name: Name::new("Bullet"),
			replication: Replication,
		}
	}
}

#[derive(Bundle)]
struct PhysicsBundle {
	velocity: Velocity,
	rigid_body: RigidBody,
	collider: Collider,
	sensor: Sensor,
	active_events: ActiveEvents,
}

impl PhysicsBundle {
	fn new(transform: &Transform) -> Self {
		let base = -Vec3::Z * (LENGTH / 2.);
		let start = transform.rotation.mul_vec3(base);
		let end = transform.rotation.mul_vec3(-base);

		PhysicsBundle {
			velocity: Velocity {
				// linvel: transform.forward().normalize() * BULLET_SPEED,
				linvel: Vec3::ZERO,
				angvel: Vec3::ZERO,
			},
			rigid_body: RigidBody::KinematicVelocityBased,
			collider: Collider::capsule(start, end, RADIUS),
			sensor: Sensor,
			active_events: ActiveEvents::COLLISION_EVENTS,
		}
	}
}

fn authoritative_spawn_bullets(
	mut requests: EventReader<FromClient<PlayerFireInput>>,
	players: Query<(&ControllablePlayer, &Children)>,
	mut player_weapons: Query<(&mut Weapon, &GlobalTransform)>,
	mut commands: Commands,
) {
	for FromClient {
		client_id,
		event: _,
	} in requests.iter()
	{
		if let Some((_, children)) = players.iter().find(|(p, _)| p.network_id == *client_id) {
			// children of the right player
			for child in children {
				if let Ok((weapon, global_transform)) = player_weapons.get_mut(*child) {
					let weapon = weapon.into_inner();
					let weapon_info = &mut weapon.info;

					if weapon_info.cooldown == Duration::ZERO {
						// resets weapon cooldown
						weapon_info.cooldown = weapon_info.fire_rate;

						// spawn authoritative bullet for physics sim
						commands.spawn(AuthoritativeBulletBundle::new(
							global_transform.reparented_to(&GlobalTransform::IDENTITY),
							weapon_info.bullets_info.clone(),
							*client_id,
						));
					}
				}
			}
		} else {
			warn!(
				"Trying to shoot bullet for player {:?} but no player by that ID exists!",
				client_id
			);
		}
	}
}

fn authoritative_hydrate_bullets(
	new_bullets: Query<(Entity, &SpawnBullet), Added<SpawnBullet>>,
	mut commands: Commands,
	mut mma: MMA,
) {
	for (new_bullet, spawn_info) in new_bullets.iter() {
		let mut new_bullet = commands.entity(new_bullet);

		debug!("Spawning a bullet at {:#?}", spawn_info.transform);

		// prep / hydration of standard components
		new_bullet.remove::<SpawnBullet>();
		new_bullet.insert(ClientBulletBundle::new(
			Bullet {
				ttl: spawn_info.info.lifetime,
				spawned_by: spawn_info.spawned_by,
			},
			spawn_info.transform,
		));
		new_bullet.with_children(|parent| {
			parent.spawn(ClientBulletBundle::get_child_pbr(&mut mma));
		});
	}
}

// fn authoritative_handle_firing(
// 	mut weapons: Query<(&mut Weapon, &GlobalTransform)>,
// 	mut commands: Commands,
// 	mut mma: MM,
// ) {
// for (mut weapon, transform) in weapons.iter_mut() {
// 	if let Some(try_fire) = weapon.flags.try_fire_this_frame {
// 		weapon.flags.try_fire_this_frame = None;
// 		if try_fire {
// 			let transform = transform.reparented_to(&GlobalTransform::IDENTITY);

// 			// info!("Firing weapon at: {:?}", transform);

// 			commands
// 				.spawn(
// 					PbrBundle {
// 						transform,
// 						..default()
// 					}
// 					.insert(BulletTimeout::default()),
// 				)
// 				.with_children(|parent| {
// 					parent.spawn(PbrBundle {
// 						transform: Transform::from_rotation(Quat::from_rotation_x(-TAU / 4.)),
// 						material: mma.mats.add(StandardMaterial {
// 							base_color: Color::RED,
// 							emissive: Color::RED,
// 							alpha_mode: AlphaMode::Add,
// 							unlit: true,
// 							perceptual_roughness: 0.,
// 							..default()
// 						}),
// 						mesh: mma.meshs.add(
// 							shape::Capsule {
// 								radius: PIXEL_SIZE / 10.,
// 								depth: PIXEL_SIZE * 0.9,
// 								rings: 4,
// 								..default()
// 							}
// 							.into(),
// 						),
// 						..default()
// 					});
// 				});
// 		}
// 	}
// }
// }
