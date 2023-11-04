use crate::{
	bevy::netcode::{AuthoritativeUpdate, ClientUpdate},
	utils::*,
};

pub struct PlayerWeaponsPlugin;
impl Plugin for PlayerWeaponsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			FixedUpdate,
			(
				(
					// authoritative_handle_firing,
					authoritative_hydrate_bullets,
					// authoritative_update_bullets,
				)
					.chain()
					.in_set(AuthoritativeUpdate),
				(send_player_fire_request,).in_set(ClientUpdate),
			),
		);
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
	name: Name,
	replication: Replication,
}

fn authoritative_hydrate_bullets() {}

fn authoritative_handle_firing(
	mut weapons: Query<(&mut Weapon, &GlobalTransform)>,
	mut commands: Commands,
	mut mma: MM,
) {
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
}

fn authoritative_update_bullets(
	// mut bullets: Query<(Entity, &mut Transform, &mut BulletTimeout)>,
	// time: Res<Time>,
	// mut commands: Commands,
) {
	// for (entity, mut transform, mut bullet) in bullets.iter_mut() {
	// 	let translation = transform.translation;
	// 	let forward = transform.forward();

	// 	transform.translation = translation + forward * 10.;
	// 	bullet.timer.tick(time.delta());
	// 	if bullet.timer.finished() {
	// 		commands.entity(entity).despawn_recursive();
	// 	}
	// }
}
