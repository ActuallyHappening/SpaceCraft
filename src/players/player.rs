use crate::prelude::*;

pub use api::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.depends_on::<RepliconCorePlugin, _>(ReplicationPlugins);
		app.depends_on::<crate::cameras::CameraPlugin, _>(crate::cameras::CameraPlugin);

		replicate_marked!(app, player_blueprint::PlayerBlueprintComponent);

		app
			.register_type::<player_blueprint::PlayerBlueprintComponent>()
			.register_type::<components::ControllablePlayer>()
			.add_systems(
				FixedUpdate,
				(Self::handle_spawn_player_blueprints.in_set(BlueprintExpansion::Player),),
			)
			.add_systems(
				WorldCreation,
				Self::creation_spawn_initial.in_set(WorldCreationSet::InitialPlayer),
			);
	}
}

mod api {
	pub use super::components::ControllablePlayer;
	pub use super::player_blueprint::{PlayerBlueprintBundle, PlayerBlueprintComponent};
}

mod systems {
	use crate::{
		cameras::ChangeCameraConfig, players::spawn_points::AvailableSpawnPoints, prelude::*,
	};

	use super::{player_blueprint::PlayerBlueprintComponent, PlayerBlueprintBundle, PlayerPlugin};

	impl PlayerPlugin {
		pub(super) fn handle_spawn_player_blueprints(
			player_blueprints: Query<
				(Entity, &PlayerBlueprintComponent),
				Added<PlayerBlueprintComponent>,
			>,
			mut commands: Commands,
			mut mma: MMA,
			mut set_primary_camera: EventWriter<ChangeCameraConfig>,
			local_id: ClientID,
		) {
			for (player, player_blueprint) in player_blueprints.iter() {
				debug!(
					"Expanding player blueprint for {:?}",
					player_blueprint.get_network_id()
				);
				commands
					.entity(player)
					.insert(player_blueprint.stamp())
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
						if local_id.get() == Some(player_blueprint.get_network_id()) {
							set_primary_camera.send(ChangeCameraConfig::SetPrimaryCamera {
								follow_camera_block: camera_entity,
							});
							debug!(
								"Using player's {} primary camera block as the primary camera",
								local_id.assert_client_id()
							);
						}
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

			commands.spawn(PlayerBlueprintBundle::new(SERVER_ID, transform));
		}
	}
}

mod components {
	use crate::prelude::*;

	/// The marker component for player entities.
	///
	///
	// / The [Eq] impl compares the [ClientId]s of the players.
	// / The [Serialize] and [Deserialize] impls serialize the [ClientId]s of the players,
	// / and are **NOT** synced using [bevy_replicon]
	#[derive(Component, Reflect, Debug)]
	pub struct ControllablePlayer {
		network_id: u64,
	}

	impl GetNetworkId for ControllablePlayer {
		fn get_network_id(&self) -> ClientId {
			ClientId::from_raw(self.network_id)
		}
	}

	impl ControllablePlayer {
		pub(super) fn new(network_id: ClientId) -> Self {
			Self {
				network_id: network_id.raw(),
			}
		}
	}
}

mod player_blueprint {
	use crate::{
		blocks::manual_builder::Facing, cameras::CameraBlockBlueprint,
		players::thruster_block::ThrusterBlockBlueprint, prelude::*,
	};

	/// What is used to construct a [PlayerBundle]
	#[derive(Component, Reflect, Serialize, Deserialize, Debug)]
	pub struct PlayerBlueprintComponent {
		pub(super) network_id: u64,
		pub(super) structure_children: Vec<BlockBlueprint<StructureBlockBlueprint>>,
		pub(super) thruster_children: Vec<BlockBlueprint<ThrusterBlockBlueprint>>,
		pub(super) primary_camera: BlockBlueprint<CameraBlockBlueprint>,
	}

	#[derive(Bundle, Debug, Serialize, Deserialize, Deref)]
	pub struct PlayerBlueprintBundle {
		/// Synced
		pub(super) transform: Transform,
		/// Synced
		#[deref]
		pub(super) blueprint: PlayerBlueprintComponent,
	}

	impl PlayerBlueprintBundle {
		pub fn new(network_id: ClientId, transform: Transform) -> Self {
			PlayerBlueprintBundle {
				transform,
				blueprint: PlayerBlueprintComponent {
					network_id: network_id.raw(),
					structure_children: vec![
						BlockBlueprint::new_structure(StructureBlockBlueprint::Aluminum, IVec3::ZERO),
						BlockBlueprint::new_structure(StructureBlockBlueprint::Aluminum, IVec3::new(0, 0, -1)),
					],
					thruster_children: vec![
						BlockBlueprint::new_thruster(IVec3::new(-1, 0, 0), Facing::Left),
						BlockBlueprint::new_thruster(IVec3::new(1, 0, 0), Facing::Right),
					],
					primary_camera: BlockBlueprint::new_camera(IVec3::new(0, 1, 0), Facing::Forwards),
				},
			}
		}
	}
	impl PlayerBlueprintComponent {
		pub fn derive_thruster_ids(&self) -> impl Iterator<Item = BlockId> + '_ {
			self.thruster_children.iter().map(|b| b.get_block_id())
		}
	}

	impl GetNetworkId for PlayerBlueprintComponent {
		fn get_network_id(&self) -> ClientId {
			ClientId::from_raw(self.network_id)
		}
	}

	impl PlayerBlueprintComponent {
		pub fn stamp(&self) -> <PlayerBlueprintComponent as Blueprint>::Bundle {
			Blueprint::stamp(self, &mut ())
		}
	}
}
mod player_bundle {
	use bevy::render::view::NoFrustumCulling;

	use crate::{players::player_movement::PlayerBundleMovementExt, prelude::*};

	use super::{ControllablePlayer, PlayerBlueprintComponent, PlayerBlueprintBundle};

	/// Parent entity of a player.
	/// Doesn't actually have its own [Mesh] / [Collider],
	/// because its children provide that for it.
	/// 
	/// Also, doesn't have a transform because [PlayerBlueprintBundle] provides
	/// that for it through [bevy_replicon].
	#[derive(Bundle)]
	pub struct PlayerBundle {
		spatial: SpatialBundleNoTransform,
		replication: Replication,
		mass: MassPropertiesBundle,
		body: RigidBody,
		name: Name,
		controllable_player: ControllablePlayer,
		external_force: ExternalForce,

		inputs_ext: PlayerBundleMovementExt,

		/// Stops the player from disappearing when inside a spawn point
		no_frustum: NoFrustumCulling,
	}

	impl Blueprint for PlayerBlueprintComponent {
		type Bundle = PlayerBundle;
		type StampSystemParam<'w, 's> = ();

		fn stamp(&self, _system_param: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle {
			let PlayerBlueprintComponent {
				network_id,
				structure_children: _,
				thruster_children: _,
				primary_camera: _,
			} = self;
			Self::Bundle {
				spatial: Default::default(),
				name: Name::new(format!("Player {}", network_id)),
				controllable_player: ControllablePlayer::new(ClientId::from_raw(*network_id)),
				mass: MassPropertiesBundle::new_computed(&Collider::ball(1.0), 1.0),
				external_force: ExternalForce::ZERO.with_persistence(false),
				body: RigidBody::Dynamic,
				replication: Replication,
				inputs_ext: PlayerBundleMovementExt::new(),
				no_frustum: NoFrustumCulling,
			}
		}
	}

	impl NetworkedBlueprintBundle for PlayerBlueprintBundle {
		type NetworkedBlueprintComponent = PlayerBlueprintComponent;
	}
}
