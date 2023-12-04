use crate::prelude::*;

pub use player_blueprint::PlayerBlueprint;

pub struct PlayerPlugin;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlayerMovement {
	/// Syncs controllable player's thruster inputs
	/// with the data in each thruster
	SyncThrustersData,

	/// Applies the forces from the thruster's data
	/// into actual forces
	EnactThrusters,
}

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.replicate::<PlayerBlueprint>()
			.add_systems(
				FixedUpdate,
				(
					Self::handle_spawn_player_blueprints.in_set(BlueprintExpansion::Player),
					Self::sync_thruster_data.in_set(GlobalSystemSet::PlayerMovement(
						PlayerMovement::SyncThrustersData,
					)),
				),
			)
			.register_type::<ControllablePlayer>();
	}
}

/// The marker component for player entities.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false)]
pub struct ControllablePlayer {
	#[reflect(ignore)]
	network_id: ClientId,

	movement_input: Option<HashMap<BlockId, f32>>,
}

impl ControllablePlayer {
	pub fn get_id(&self) -> ClientId {
		self.network_id
	}

	fn with_thruster_mapping(mut self, thruster_ids: Vec<BlockId>) -> Self {
		assert!(self.movement_input.is_none());
		self.movement_input = Some(thruster_ids.into_iter().map(|id| (id, 0.)).collect());
		self
	}
}

mod systems {
	use crate::{
		cameras::{CameraBlockBundle, ChangeCameraConfig},
		players::{player::player_blueprint::PlayerBundle, thruster_block::{ThrusterBlockBundle, Thruster}},
		prelude::*,
	};

	use super::{PlayerBlueprint, PlayerPlugin, ControllablePlayer};

	impl PlayerPlugin {
		pub(super) fn sync_thruster_data(players: Query<(&ControllablePlayer, &Children)>, mut thrusters: Query<&mut Thruster>) {
			for (controllable_player, children) in players.iter() {
				if let Some(movement_input) = &controllable_player.movement_input {
					for child in children.iter() {
						if let Ok(mut thruster) = thrusters.get_mut(*child) {
							if let Some(input) = movement_input.get(&thruster.get_id()) {
								thruster.set_status(*input);
							}
						}
					}
				} else {
					warn!("Player is spawned but without an input_map");
				}
			}
		}

		pub(super) fn handle_spawn_player_blueprints(
			player_blueprints: Query<(Entity, &PlayerBlueprint), Added<PlayerBlueprint>>,
			mut commands: Commands,
			mut mma: MMA,
			mut set_primary_camera: EventWriter<ChangeCameraConfig>,
		) {
			for (player, player_blueprint) in player_blueprints.iter() {
				debug!(
					"Expanding player blueprint for {:?}",
					player_blueprint.get_network_id()
				);
				commands
					.entity(player)
					.insert(PlayerBundle::stamp_from_blueprint(
						player_blueprint,
						&mut mma,
					))
					.with_children(|parent| {
						for blueprint in &player_blueprint.structure_children {
							parent.spawn(StructureBlockBundle::stamp_from_blueprint(
								blueprint, &mut mma,
							));
						}

						for blueprint in &player_blueprint.thruster_children {
							parent.spawn(ThrusterBlockBundle::stamp_from_blueprint(
								blueprint, &mut mma,
							));
						}

						let camera_entity = parent
							.spawn(CameraBlockBundle::stamp_from_blueprint(
								&player_blueprint.primary_camera,
								&mut mma,
							))
							.id();
						set_primary_camera.send(ChangeCameraConfig::SetPrimaryCamera {
							follow_camera_block: camera_entity,
						});
						debug!("Using player's primary camera block as the primary camera");
					});
			}
		}
	}
}

mod player_blueprint {
	use crate::{
		blocks::manual_builder::Facing, cameras::CameraBlockBlueprint,
		players::thruster_block::ThrusterBlockBlueprint, prelude::*,
	};

	use super::ControllablePlayer;

	/// What is used to construct a [PlayerBundle]
	#[derive(Component, Serialize, Deserialize, Clone, Debug)]
	pub struct PlayerBlueprint {
		network_id: ClientId,
		transform: Transform,
		pub(super) structure_children: Vec<BlockBlueprint<StructureBlockBlueprint>>,
		pub(super) thruster_children: Vec<BlockBlueprint<ThrusterBlockBlueprint>>,
		pub(super) primary_camera: BlockBlueprint<CameraBlockBlueprint>,
	}

	impl PlayerBlueprint {
		pub fn default_at(network_id: ClientId, transform: Transform) -> Self {
			PlayerBlueprint {
				network_id,
				transform,
				structure_children: vec![
					BlockBlueprint::new_structure(StructureBlockBlueprint::Aluminum, IVec3::ZERO),
					BlockBlueprint::new_structure(StructureBlockBlueprint::Aluminum, IVec3::new(0, 0, -1)),
				],
				thruster_children: vec![
					BlockBlueprint::new_thruster(IVec3::new(-1, 0, 0), Facing::Left),
					BlockBlueprint::new_thruster(IVec3::new(1, 0, 0), Facing::Right),
				],
				primary_camera: BlockBlueprint::new_camera(IVec3::new(0, 1, 0), Facing::Forwards),
			}
		}

		pub fn get_network_id(&self) -> ClientId {
			self.network_id
		}
	}

	/// Parent entity of a player.
	/// Doesn't actually have its own mesh
	#[derive(Bundle)]
	pub(super) struct PlayerBundle {
		spatial: SpatialBundle,
		replication: Replication,
		mass: MassPropertiesBundle,
		body: RigidBody,
		name: Name,
		controllable_player: ControllablePlayer,
		external_force: ExternalForce,
	}

	impl FromBlueprint for PlayerBundle {
		type Blueprint = PlayerBlueprint;

		fn stamp_from_blueprint(
			PlayerBlueprint {
				network_id,
				transform,
				// ignores most
				thruster_children,
				..
			}: &PlayerBlueprint,
			_mma: &mut MMA,
		) -> Self {
			let thruster_ids: Vec<BlockId> = thruster_children
				.iter()
				.map(|blueprint| blueprint.specific_marker.get_id())
				.collect();

			// trace!("Spawned player has {} thrusters registered", thruster_ids.len());

			Self {
				spatial: SpatialBundle {
					transform: *transform,
					..default()
				},
				name: Name::new(format!("Player {}", network_id)),
				controllable_player: ControllablePlayer {
					network_id: *network_id,
					movement_input: Default::default(),
				}
				.with_thruster_mapping(thruster_ids),
				// collider: Collider::ball(0.1),
				mass: MassPropertiesBundle::new_computed(&Collider::ball(PIXEL_SIZE), 10.),
				external_force: ExternalForce::ZERO.with_persistence(false),
				body: RigidBody::Dynamic,
				replication: Replication,
			}
		}
	}

	impl PlayerBundle {
		fn with_thruster_mapping(mut self, thruster_ids: Vec<BlockId>) -> Self {
			self.controllable_player = self.controllable_player.with_thruster_mapping(thruster_ids);
			self
		}
	}
}
