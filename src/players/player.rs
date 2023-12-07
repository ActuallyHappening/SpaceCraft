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
		app.depends_on::<RepliconCorePlugin, _>(ReplicationPlugins);
		app.depends_on::<crate::cameras::CameraPlugin, _>(crate::cameras::CameraPlugin);

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
			.add_systems(
				WorldCreation,
				Self::creation_spawn_initial.in_set(WorldCreationSet::InitialPlayer),
			)
			.register_type::<ControllablePlayer>();
	}
}

mod systems {
	use crate::{
		cameras::{CameraBlockBundle, ChangeCameraConfig},
		players::{
			player::player_blueprint::PlayerBundle,
			spawn_points::AvailableSpawnPoints,
			thruster_block::{Thruster, ThrusterBlockBundle},
		},
		prelude::*,
	};

	use super::{ControllablePlayer, PlayerBlueprint, PlayerPlugin};

	impl PlayerPlugin {
		pub(super) fn sync_thruster_data(
			players: Query<(&ControllablePlayer, &Children)>,
			mut thrusters: Query<&mut Thruster>,
		) {
			for (controllable_player, children) in players.iter() {
				let movement_input = &controllable_player.get_movement_inputs();
				for child in children.iter() {
					if let Ok(mut thruster) = thrusters.get_mut(*child) {
						if let Some(input) = movement_input.get(&thruster.get_id()) {
							thruster.set_status(*input);
						}
					}
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
					.insert(player_blueprint.stamp(&mut ()))
					.with_children(|parent| {
						for blueprint in &player_blueprint.structure_children {
							parent.spawn(blueprint.stamp(&mut mma));
						}

						for blueprint in &player_blueprint.thruster_children {
							parent.spawn(blueprint.stamp(&mut mma));
						}

						let camera_entity = parent
							.spawn(player_blueprint.primary_camera.stamp(&mut mma))
							.id();
						set_primary_camera.send(ChangeCameraConfig::SetPrimaryCamera {
							follow_camera_block: camera_entity,
						});
						debug!("Using player's primary camera block as the primary camera");
					});
			}
		}

		pub(super) fn creation_spawn_initial(
			mut commands: Commands,
			spawn_point: AvailableSpawnPoints,
		) {
			// commands.spawn(PlayerBlueprint::new(SERVER_ID, Transform::IDENTITY));
			let transform = spawn_point
				.try_get_spawn_location(SERVER_ID)
				.expect("No more spawn points left!");

			commands.spawn(PlayerBlueprint::new(SERVER_ID, transform));
		}
	}

	#[test]
	fn player_blueprint_expands() {
		let mut app = test_app();

		app.add_plugins(PlayerPlugin);

		const ID: ClientId = ClientId::from_raw(69);
		let transform = Transform::from_xyz(random(), random(), random());
		app.world.spawn(PlayerBlueprint::new(ID, transform));

		app.world.run_schedule(FixedUpdate);

		app
			.world
			.run_system_once(|players: Query<(EntityRef, &ControllablePlayer)>| {
				let (player, control) = players
					.get_single()
					.expect("Only one controllable player expanded");
				assert_eq!(control.get_id(), ID);
				assert!(player.get::<PlayerBlueprint>().is_some());
			});
	}
}

pub use components::ControllablePlayer;
mod components {
	use crate::prelude::*;

	/// The marker component for player entities.
	#[derive(Component, Reflect)]
	#[reflect(from_reflect = false)]
	pub struct ControllablePlayer {
		#[reflect(ignore)]
		network_id: ClientId,

		movement_input: HashMap<BlockId, f32>,
	}

	impl ControllablePlayer {
		pub fn get_id(&self) -> ClientId {
			self.network_id
		}

		pub(super) fn get_movement_inputs(&self) -> &HashMap<BlockId, f32> {
			&self.movement_input
		}

		pub(super) fn new_with_thruster_mapping(
			network_id: ClientId,
			thruster_ids: Vec<BlockId>,
		) -> Self {
			Self {
				network_id,
				movement_input: thruster_ids.into_iter().map(|id| (id, 0.)).collect(),
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
		pub fn new(network_id: ClientId, transform: Transform) -> Self {
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
	pub struct PlayerBundle {
		spatial: SpatialBundle,
		replication: Replication,
		mass: MassPropertiesBundle,
		body: RigidBody,
		name: Name,
		controllable_player: ControllablePlayer,
		external_force: ExternalForce,
	}

	impl Blueprint for PlayerBlueprint {
		type Bundle = PlayerBundle;
		type StampSystemParam<'w, 's> = ();

		fn stamp<'w, 's>(
			&self,
			system_param: &mut Self::StampSystemParam<'w, 's>,
		) -> Self::Bundle {
			let PlayerBlueprint { network_id, transform, structure_children, thruster_children, primary_camera } = self;
			let thruster_ids: Vec<BlockId> = thruster_children
				.iter()
				.map(|blueprint| blueprint.specific_marker.get_id())
				.collect();

			// trace!("Spawned player has {} thrusters registered", thruster_ids.len());

			Self::Bundle {
				spatial: SpatialBundle {
					transform: *transform,
					..default()
				},
				name: Name::new(format!("Player {}", network_id)),
				controllable_player: ControllablePlayer::new_with_thruster_mapping(
					*network_id,
					thruster_ids,
				),
				mass: MassPropertiesBundle::new_computed(&Collider::ball(1.0), 1.0),
				external_force: ExternalForce::ZERO.with_persistence(false),
				body: RigidBody::Dynamic,
				replication: Replication,
			}
		}
	}
}
