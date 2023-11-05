use crate::{
	bevy::netcode::{AuthoritativeUpdate, ClientUpdate},
	utils::*,
};

use super::ControllablePlayer;

const RADIUS: f32 = PIXEL_SIZE / 10.;
const LENGTH: f32 = PIXEL_SIZE * 0.8;
const BULLET_SPEED: f32 = 1000.;

pub struct PlayerWeaponsPlugin;
impl Plugin for PlayerWeaponsPlugin {
	fn build(&self, app: &mut App) {
		app
			.replicate::<SpawnBullet>()
			.add_server_event::<BulletCollision>(EventType::Unordered)
			.add_systems(
				FixedUpdate,
				(
					(
						authoritative_spawn_bullets,
						authoritative_tick_weapons,
						authoritative_bullet_collide,
						authoritative_tick_bullets,
						authoritative_handle_bullet_collisions,
					)
						.chain()
						.in_set(AuthoritativeUpdate),
					(send_player_fire_request, hydrate_bullets).in_set(ClientUpdate),
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
	/// takes the global transform of the weapon
	fn new(
		origin_transform: GlobalTransform,
		player_global_transform: GlobalTransform,
		info: BulletInfo,
		spawned_by: u64,
	) -> Self {
		let global_transform = origin_transform.reparented_to(&GlobalTransform::IDENTITY);
		AuthoritativeBulletBundle {
			physics: PhysicsBundle::new(
				global_transform,
				origin_transform.reparented_to(&player_global_transform),
			),
			to_spawn: SpawnBullet {
				transform: global_transform,
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
	fn new(global_transform: Transform, player_relative_transform: Transform) -> Self {
		let base = -Vec3::Z * (LENGTH / 2.);
		let start = player_relative_transform.rotation.mul_vec3(base);
		let end = player_relative_transform.rotation.mul_vec3(-base);

		PhysicsBundle {
			velocity: Velocity {
				linvel: global_transform.forward().normalize() * BULLET_SPEED,
				// linvel: Vec3::ZERO,
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
	players: Query<(&ControllablePlayer, &Children, &GlobalTransform)>,
	mut player_weapons: Query<(&mut Weapon, &GlobalTransform)>,
	mut commands: Commands,
) {
	for FromClient {
		client_id,
		event: _,
	} in requests.iter()
	{
		if let Some((_, children, player_global_transform)) =
			players.iter().find(|(p, _, _)| p.network_id == *client_id)
		{
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
							*global_transform,
							*player_global_transform,
							weapon_info.bullets_info.clone(),
							*client_id,
						));
					} else {
						// #[cfg(feature = "debugging")]
						// warn!("Trying to shoot while still on cooldown");
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

fn hydrate_bullets(
	new_bullets: Query<(Entity, &SpawnBullet), Added<SpawnBullet>>,
	mut commands: Commands,
	mut mma: MMA,
) {
	for (new_bullet, spawn_info) in new_bullets.iter() {
		let mut new_bullet = commands.entity(new_bullet);

		// debug!("Hydrating bullet spawned by {:?}", spawn_info.spawned_by);

		// prep / hydration of standard components
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

fn authoritative_tick_weapons(mut weapons: Query<&mut Weapon>, time: Res<FixedTime>) {
	for mut weapon in weapons.iter_mut() {
		weapon.info.cooldown = weapon.info.cooldown.saturating_sub(time.period);
	}
}

/// ticks bullets ttl and de-spawns them if they are out of time
fn authoritative_tick_bullets(
	mut bullets: Query<(Entity, &mut Bullet)>,
	time: Res<FixedTime>,
	mut commands: Commands,
) {
	for (bullet_entity, mut bullet) in bullets.iter_mut() {
		bullet.ttl = bullet.ttl.saturating_sub(time.period);
		if bullet.ttl == Duration::ZERO {
			commands.entity(bullet_entity).despawn_recursive();
		}
	}
}

#[derive(Debug, Event, Serialize, Deserialize)]
enum BulletCollision {
	/// Bullet collided with a player that didn't shoot the original bullet
	BulletWithPlayer {
		bullet_entity: Entity,
		shooter_id: u64,
		player_entity: Entity,
		player_hit_id: u64,
	},
}

fn authoritative_bullet_collide(
	mut collision_events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
	players: Query<&ControllablePlayer>,
	bullets: Query<&Bullet>,
	mut event_writer: EventWriter<BulletCollision>,
) {
	for event in collision_events.iter() {
		if let CollisionEvent::Started(e1, e2, _) = event {
			let player_entity;
			let player_hit_id;
			if let Ok(ControllablePlayer { network_id, .. }) = players.get(*e1) {
				player_entity = *e1;
				player_hit_id = *network_id;
			} else if let Ok(ControllablePlayer { network_id, .. }) = players.get(*e2) {
				player_entity = *e2;
				player_hit_id = *network_id;
			} else {
				// player is not involved
				return;
			}

			let bullet_entity;
			let shooter_id;
			if let Ok(bullet) = bullets.get(*e1) {
				bullet_entity = *e1;
				shooter_id = bullet.spawned_by;
			} else if let Ok(bullet) = bullets.get(*e2) {
				bullet_entity = *e2;
				shooter_id = bullet.spawned_by;
			} else {
				// bullet is not involved
				return;
			}

			if player_entity == bullet_entity {
				error!("Player is also a bullet?");
				return;
			}

			if shooter_id == player_hit_id {
				// player shot themselves, ignore
				return;
			}

			let event = BulletCollision::BulletWithPlayer {
				bullet_entity,
				shooter_id,
				player_entity,
				player_hit_id,
			};

			// debug!("Player was hit! {:?}", event);

			event_writer.send(event);
		}
	}
}

// technically this event reader is an anti-pattern according to bevy_replicon docs, but it works so ehh
fn authoritative_handle_bullet_collisions(
	mut local_event_reader: EventReader<BulletCollision>,
	bullets: Query<(Entity, &Bullet)>,
	mut players: Query<&mut ControllablePlayer>,
	mut commands: Commands,
) {
	for event in local_event_reader.iter() {
		match event {
			BulletCollision::BulletWithPlayer {
				bullet_entity,
				shooter_id,
				player_entity,
				player_hit_id,
			} => {
				if let Ok((bullet_entity, bullet)) = bullets.get(*bullet_entity) {
					if bullet.spawned_by != *shooter_id {
						error!("Wrong shooter? spawned by = {}, shooter_id = {}", bullet.spawned_by, shooter_id);
					}
					commands.entity(bullet_entity).despawn_recursive();
				} else {
					error!("Bullet entity does not exist!");
				}

				if let Ok(mut player) = players.get_mut(*player_entity) {
					if player.network_id != *player_hit_id {
						error!("Wrong player? network_id = {}, player_hit_id = {}", player.network_id, player_hit_id);
					}
					player.health = player.health.saturating_sub(1);
				}
			}
		}
	}
}
