use crate::prelude::*;

pub use player_blueprint::PlayerBlueprint;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.replicate::<PlayerBlueprint>().add_systems(
			FixedUpdate,
			Self::handle_spawn_player_blueprints.in_set(BlueprintExpansion::Player),
		);
	}
}

/// The marker component for player entities.
#[derive(Component)]
pub struct ControllablePlayer {
	network_id: ClientId,
	movement_input: HashMap<BlockId, f32>,
}

impl ControllablePlayer {
	pub fn get_id(&self) -> ClientId {
		self.network_id
	}
}

mod systems {
	use crate::{
		cameras::{CameraBlockBundle, ChangeCameraConfig},
		players::{player::player_blueprint::PlayerBundle, thruster_block::ThrusterBlockBundle},
		prelude::*,
	};

	use super::{PlayerBlueprint, PlayerPlugin};

	impl PlayerPlugin {
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

						let primary_camera_id = player_blueprint.primary_camera.specific_marker.id;
						parent.spawn(CameraBlockBundle::stamp_from_blueprint(
							&player_blueprint.primary_camera,
							&mut mma,
						));
						set_primary_camera.send(ChangeCameraConfig::SetPrimaryCamera {
							follow_block: primary_camera_id,
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
				// ignores children
				..
			}: &PlayerBlueprint,
			_mma: &mut MMA,
		) -> Self {
			Self {
				spatial: SpatialBundle {
					transform: *transform,
					..default()
				},
				name: Name::new(format!("Player {}", network_id)),
				controllable_player: ControllablePlayer {
					network_id: *network_id,
					movement_input: Default::default(),
				},
				// collider: Collider::ball(0.1),
				mass: MassPropertiesBundle::new_computed(&Collider::ball(PIXEL_SIZE), 10.),
				external_force: ExternalForce::ZERO.with_persistence(false),
				body: RigidBody::Dynamic,
				replication: Replication,
			}
		}
	}
}
