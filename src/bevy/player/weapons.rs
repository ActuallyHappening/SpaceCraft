use crate::utils::*;

use super::ControllablePlayer;

/// Not used directly as a Component, see [Weapon]
#[derive(Debug, Default, Component, Clone, Serialize, Deserialize)]
pub struct WeaponFlags {
	/// edited from systems
	pub(self) try_fire_this_frame: Option<bool>,
}

#[derive(Component, Debug)]
pub struct Bullet {
	timer: Timer
}

pub fn should_fire_this_frame(mouse: Res<Input<MouseButton>>) -> bool {
	mouse.just_pressed(MouseButton::Left)
}

pub fn toggle_fire(In(should_fire): In<bool>, mut weapons: Query<&mut Weapon>) {
	for mut weapon in weapons.iter_mut() {
		weapon.flags.try_fire_this_frame = Some(should_fire);
	}
}

pub fn handle_firing(
	mut weapons: Query<(&mut Weapon, &GlobalTransform)>,
	mut commands: Commands,
	mut mma: MM,
) {
	for (mut weapon, transform) in weapons.iter_mut() {
		if let Some(try_fire) = weapon.flags.try_fire_this_frame {
			weapon.flags.try_fire_this_frame = None;
			if try_fire {
				let transform = transform.reparented_to(&GlobalTransform::IDENTITY);

				// info!("Firing weapon at: {:?}", transform);

				commands
					.spawn(
						PbrBundle {
							transform,
							..default()
						}
						.insert(Bullet::default()),
					)
					.with_children(|parent| {
						parent.spawn(PbrBundle {
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
									radius: PIXEL_SIZE / 10.,
									depth: PIXEL_SIZE * 0.9,
									rings: 4,
									..default()
								}
								.into(),
							),
							..default()
						});
					});
			}
		}
	}
}

pub fn update_bullets(mut bullets: Query<(Entity, &mut Transform, &mut Bullet)>, time: Res<Time>, mut commands: Commands) {
	for (entity, mut transform, mut bullet) in bullets.iter_mut() {
		let translation = transform.translation;
		let forward = transform.forward();

		transform.translation = translation + forward * 10.;
		bullet.timer.tick(time.delta());
		if bullet.timer.finished() {
			commands.entity(entity).despawn_recursive();
		}
	}
}

impl Default for Bullet {
	fn default() -> Self {
		Self {
			timer: Timer::new(Duration::from_secs(5), TimerMode::Once)
		}
	}
}