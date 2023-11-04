use crate::{
	bevy::netcode::{AuthoritativeUpdate, ClientUpdate},
	utils::*,
};

use super::ControllablePlayer;

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

/// Not used directly as a Component, see [Weapon]
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
				ttl: Duration::from_secs(5),
			},
			// starts ready to fire
			cooldown: Duration::ZERO,

			fire_rate: Duration::from_millis(500),
		}
	}
}

/// Static / dynamic information about a bullet, used to spawn it
/// and found on actual bullets
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BulletInfo {
	ttl: Duration,
}

/// Hydrates to a full bullet
#[derive(Component, Debug, Serialize, Deserialize)]
struct SpawnBullet {
	transform: Transform,
	info: BulletInfo,
}

fn send_player_fire_request(
	mouse: Res<Input<MouseButton>>,
	mut event_writer: EventWriter<PlayerFireInput>,
) {
	if mouse.just_pressed(MouseButton::Left) {
		event_writer.send(PlayerFireInput);
	}
}

#[derive(Bundle)]
struct AuthoritativeBulletBundle {
	pbr: PbrBundle,
	physics: PhysicsBundle,

	name: Name,
	replication: Replication,
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
	fn new(direction: Quat) -> Self {
		PhysicsBundle {
			velocity: Velocity {
				linvel: Vec3::ZERO,
				angvel: Vec3::ZERO,
			},
			rigid_body: RigidBody::KinematicVelocityBased,
			collider: Collider::ball(PIXEL_SIZE / 10.),
			sensor: Sensor,
			active_events: ActiveEvents::COLLISION_EVENTS,
		}
	}
}

fn authoritative_spawn_bullets(
	mut requests: EventReader<FromClient<PlayerFireInput>>,
	players: Query<(&ControllablePlayer, &Children)>,
	player_weapons: Query<(&Weapon, &GlobalTransform)>,
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
				if let Ok((Weapon { info: weapon_info, .. }, global_transform)) = player_weapons.get(*child) {
					info!("Player {:?} is firing his weapon at {:?} with info {:?}", client_id, global_transform, weapon_info);
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

fn authoritative_hydrate_bullets() {}
