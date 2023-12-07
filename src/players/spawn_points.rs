use crate::prelude::*;

use self::components::SpawnPoint;

pub struct SpawnPointsPlugin;

impl Plugin for SpawnPointsPlugin {
	fn build(&self, app: &mut App) {
		app
			.register_type::<SpawnPoint>()
			.add_systems(Startup, Self::load_default_materials)
			.add_systems(PostProcessCollisions, Self::filter_non_occupied_collisions)
			.add_systems(
				WorldCreation,
				Self::creation_spawn_points.in_set(WorldCreationSet::SpawnPoints),
			)
			.add_systems(Update, Self::activate_local_spawn_points);
	}
}

mod systems {
	use crate::{players::ControllablePlayer, prelude::*};

	use super::{
		blueprint::SpawnPointBlueprint, bundle::SpawnPointBundle, components::SpawnPoint,
		SpawnPointsPlugin,
	};

	impl SpawnPointsPlugin {
		pub(super) fn filter_non_occupied_collisions(
			mut collisions: ResMut<Collisions>,
			players: Query<&ControllablePlayer>,
			player_blocks: Query<&Parent, With<Collider>>,
			spawn_points: Query<&SpawnPoint>,
		) {
			collisions.retain(|contacts| {
				let check = |e1: Entity, e2: Entity| -> bool {
					if let Ok(spawn_point) = spawn_points.get(e2) {
						match spawn_point.get_id() {
							// this spawn point is not occupied, so no matter
							// what player collides we should allow it
							None => false,
							Some(id) => {
								// check direct player collision
								if let Ok(player) = players.get(e1) {
									// e1 is player, e2 is spawn point, and spawn point is occupied
									// if the spawn point is occupied by the player, then ignore the collision
									if id == player.get_id() {
										// reject self-collision
										true
									} else {
										false
									}
								} else if let Ok(player_block) = player_blocks.get(e1) {
									// if is a child of player
									if let Ok(player_of_child_block) = players.get(player_block.get()) {
										// if the player of the child block is the same as the spawn point
										// then ignore the collision
										if id == player_of_child_block.get_id() {
											true // block of a player that is occupying this spawn point, ignore
										} else {
											false
										}
									} else {
										false // block is not a child of any player
									}
								} else {
									false
								}
							}
						}
					} else {
						false
					}
				};

				!(check(contacts.entity1, contacts.entity2) || check(contacts.entity2, contacts.entity1))
			});
		}

		pub(super) fn creation_spawn_points(mut commands: Commands, mut mma: MMA) {
			const CIRCLE_RADIUS: f32 = SpawnPointBlueprint::DEFAULT_SIZE * 4.0;
			const NUM_STRIPS_MAGNITUDE: isize = 2; // 5 total layers

			// altitudes
			let phi_per_strip_n = |strip_height_n: isize| -> f32 {
				match strip_height_n {
					2 => TAU / 4.,
					1 => TAU / 8.,
					0 => 0.0,
					-1 => -TAU / 8.,
					-2 => -TAU / 4.,
					_ => unreachable!(),
				}
			};

			// rotations along xz axis
			let num_thetas_per_strip_n = |strip_height_n: isize| -> usize {
				match strip_height_n {
					2 | -2 => 1,
					1 | -1 => 4,
					0 => 8,
					_ => unreachable!(),
				}
			};

			let thetas_per_strip_n = |strip_height_n: isize| -> Vec<f32> {
				let num_thetas = num_thetas_per_strip_n(strip_height_n);
				let mut thetas = Vec::with_capacity(num_thetas);
				for theta_n in 0..num_thetas {
					let theta = theta_n as f32 * TAU / num_thetas as f32;
					thetas.push(theta);
				}
				thetas
			};

			let mut starting_positions = Vec::new();

			for strip_height_n in -NUM_STRIPS_MAGNITUDE..=NUM_STRIPS_MAGNITUDE {
				let phi = phi_per_strip_n(strip_height_n);
				for theta in thetas_per_strip_n(strip_height_n) {
					// trace!("Adding vector with theta: {}, phi: {}", theta, phi);
					starting_positions.push(vec3_polar(theta, phi) * CIRCLE_RADIUS);
				}
			}

			// trace!("Starting positions: {:?}", starting_positions);

			let spawn_points: Vec<SpawnPointBundle> = starting_positions
				.iter()
				.map(|pos| {
					let transform = Transform::from_translation(*pos);
					let blueprint = SpawnPointBlueprint::new(transform, None);
					blueprint.stamp(&mut mma)
				})
				.collect();
			commands.spawn_batch(spawn_points);
		}

		pub(super) fn load_default_materials(mut materials: ResMut<Assets<StandardMaterial>>) {
			materials.insert(
				SpawnPointBlueprint::DEFAULT_MATERIAL_HANDLE,
				SpawnPointBlueprint::get_default_material(),
			);
			materials.insert(
				SpawnPointBlueprint::ACTIVE_MATERIAL_HANDLE,
				SpawnPointBlueprint::get_active_material(),
			);
		}

		pub(super) fn activate_local_spawn_points(
			mut spawn_points: Query<(&SpawnPoint, &mut Handle<StandardMaterial>)>,
			local_id: ClientID,
		) {
			if let Some(id) = local_id.get() {
				for (spawn_point, mut material) in spawn_points.iter_mut() {
					if spawn_point.get_id() == Some(id) {
						*material = SpawnPointBlueprint::ACTIVE_MATERIAL_HANDLE;
					} else {
						*material = SpawnPointBlueprint::DEFAULT_MATERIAL_HANDLE;
					}
				}
			}
		}
	}
}

mod blueprint {
	use crate::prelude::*;

	#[derive(Debug, Component, Reflect, Clone, Serialize, Deserialize)]
	pub struct SpawnPointBlueprint {
		pub at: Transform,
		pub size: f32,

		#[reflect(ignore)]
		pub initial_occupation: Option<ClientId>,
	}

	impl SpawnPointBlueprint {
		pub fn new(at: Transform, initially_occupied: Option<ClientId>) -> Self {
			Self {
				at,
				initial_occupation: initially_occupied,
				size: SpawnPointBlueprint::DEFAULT_SIZE,
			}
		}

		pub const DEFAULT_SIZE: f32 = 3.0;
	}

	// impl Default for SpawnPointBlueprint {
	// 	fn default() -> Self {
	// 		Self {
	// 			at: Transform::default(),
	// 			size: 3.0,
	// 			initial_occupation: None,
	// 		}
	// 	}
	// }
}

mod bundle {
	use crate::prelude::*;

	use super::{blueprint::SpawnPointBlueprint, components::SpawnPoint};

	#[derive(Bundle)]
	pub struct SpawnPointBundle {
		pbr: PbrBundle,
		name: Name,
		spawn_point: SpawnPoint,
		rigid_body: RigidBody,
		collider: AsyncCollider,
	}

	impl SpawnPointBlueprint {
		pub const DEFAULT_MATERIAL_HANDLE: Handle<StandardMaterial> =
			Handle::weak_from_u128(1234760192378468914256943588769860234);

		pub fn get_default_material() -> StandardMaterial {
			StandardMaterial {
				base_color: Color::BLUE.with_a(0.5),
				emissive: Color::BLUE.with_a(0.2),
				specular_transmission: 0.9,
				thickness: 5.0,
				ior: 1.33,
				// attenuation_distance: 0.0,
				// attenuation_color: (),
				// normal_map_texture: (),
				// flip_normal_map_y: (),
				alpha_mode: AlphaMode::Blend,
				// opaque_render_method: (),
				..default()
			}
		}

		pub const ACTIVE_MATERIAL_HANDLE: Handle<StandardMaterial> =
			Handle::weak_from_u128(12347601923784689142569435843281790);

		pub fn get_active_material() -> StandardMaterial {
			StandardMaterial {
				base_color: Color::GREEN.with_a(0.2),
				emissive: Color::GREEN.with_a(0.2),
				specular_transmission: 0.9,
				thickness: 5.0,
				ior: 1.33,
				// attenuation_distance: 0.0,
				// attenuation_color: (),
				// normal_map_texture: (),
				// flip_normal_map_y: (),
				alpha_mode: AlphaMode::Blend,
				// opaque_render_method: (),
				..default()
			}
		}
	}

	impl Blueprint for SpawnPointBlueprint {
		type Bundle = SpawnPointBundle;
		type StampSystemParam<'w, 's> = MMA<'w>;

		fn stamp(&self, mma: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle {
			let SpawnPointBlueprint {
				at,
				size,
				initial_occupation,
			} = self;

			SpawnPointBundle {
				pbr: PbrBundle {
					transform: *at,
					mesh: mma.meshs.add(
						shape::UVSphere {
							radius: *size,
							..default()
						}
						.into(),
					),
					material: SpawnPointBlueprint::DEFAULT_MATERIAL_HANDLE,
					..default()
				},
				name: Name::new("SpawnPoint"),
				spawn_point: SpawnPoint::new(*initial_occupation),
				rigid_body: RigidBody::Kinematic,
				collider: AsyncCollider::default(),
			}
		}
	}
}

pub use components::AvailableSpawnPoints;
mod components {
	use crate::prelude::*;

	#[derive(Component, Debug, Reflect)]
	pub(super) struct SpawnPoint {
		/// Player that occupies this spawn point.
		#[reflect(ignore)]
		player_occupation: Option<ClientId>,
	}

	impl SpawnPoint {
		pub(super) fn new(occupation: Option<ClientId>) -> Self {
			Self {
				player_occupation: occupation,
			}
		}

		pub(super) fn get_id(&self) -> Option<ClientId> {
			self.player_occupation
		}

		fn set_id(&mut self, id: ClientId) {
			self.player_occupation = Some(id);
		}
	}

	#[derive(SystemParam)]
	// #[system_param(mutable)]
	pub struct AvailableSpawnPoints<'w, 's> {
		spawn_points: Query<'w, 's, (&'static mut SpawnPoint, &'static Transform)>,
		state: Res<'w, NetcodeConfig>,
	}

	impl AvailableSpawnPoints<'_, '_> {
		/// Returns a valid spawn location, handling side effects.
		///
		/// Maybe: Handle spawning a new spawn location in the future?
		pub fn try_get_spawn_location(mut self, player_occupying: ClientId) -> Option<Transform> {
			if !self.state.is_authoritative() {
				error!(
					"Cannot assign spawn points from a non-authoritative state: {:?}",
					self.state
				);
			}

			let mut available_points = self
				.spawn_points
				.iter_mut()
				.filter(|(sp, _)| sp.player_occupation.is_none());

			let Some((mut spawn_point, global_transform)) = available_points.next() else {
				return None;
			};

			spawn_point.set_id(player_occupying);

			Some(*global_transform)
		}
	}
}
