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
				Blueprints,
				Self::handle_spawn_player_blueprints.in_set(BlueprintExpansion::Player),
			)
			.add_systems(
				FixedUpdate,
				(
					Self::manage_primary_camera.run_if(NetcodeConfig::not_headless()),
					Self::name_player,
				),
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
		cameras::{BlockEntity, CameraBlockMarker, ChangeCameraConfig},
		players::spawn_points::AvailableSpawnPoints,
		prelude::*,
	};

	use super::{
		player_blueprint::PlayerBlueprintComponent, ControllablePlayer, PlayerBlueprintBundle,
		PlayerPlugin,
	};

	impl PlayerPlugin {
		pub(super) fn handle_spawn_player_blueprints(
			player_blueprints: Query<
				(Entity, &PlayerBlueprintComponent, &NetworkId),
				Changed<PlayerBlueprintComponent>,
			>,
			mut commands: Commands,
			mut mma: MMA,
		) {
			for (player, player_blueprint, id) in player_blueprints.iter() {
				debug!("Expanding player blueprint for {:?}", id);
				commands
					.entity(player)
					.despawn_descendants()
					.insert(BlueprintUpdated)
					.insert(player_blueprint.stamp())
					.with_children(|parent| {
						for blueprint in &player_blueprint.structure_children {
							parent
								.spawn(blueprint.stamp(&mut mma))
								.insert(BlueprintUpdated);
						}

						for blueprint in &player_blueprint.thruster_children {
							parent
								.spawn(blueprint.stamp(&mut mma))
								.insert(BlueprintUpdated);
						}

						parent
							.spawn(player_blueprint.primary_camera.stamp(&mut mma))
							.insert(BlueprintUpdated);
					});
			}
		}

		/// When new [CameraBlockMarker]s are spawned,
		/// check if they are the child of the local player.
		/// If so, set the primary camera to it.
		pub(super) fn manage_primary_camera(
			// player spawned
			players: Query<&NetworkId, (With<ControllablePlayer>, Added<BlueprintUpdated>)>,
			// camera spawned as well
			camera_blocks: Query<(Entity, &Parent), (With<CameraBlockMarker>, Added<BlueprintUpdated>)>,
			mut set_primary_camera: EventWriter<ChangeCameraConfig>,
			local_id: ClientID,
		) {
			for (e, parent) in camera_blocks.iter() {
				if let Ok(player) = players.get(parent.get()) {
					if local_id.get() == Some(player.get_network_id()) {
						set_primary_camera.send(ChangeCameraConfig::SetPrimaryCamera {
							follow_camera_block: BlockEntity(e),
						});
					}
				}
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

		pub(super) fn name_player(
			mut players: Query<
				(&mut Name, &NetworkId),
				(With<ControllablePlayer>, Added<BlueprintUpdated>),
			>,
		) {
			for (mut name, id) in players.iter_mut() {
				*name = Name::new(format!("Player {}", id.get_network_id()));
			}
		}
	}
}

mod components {
	use crate::prelude::*;

	/// The marker component for player entities.
	#[derive(Component, Reflect, Debug)]
	pub struct ControllablePlayer;
}

mod player_blueprint {
	use crate::{
		blocks::manual_builder::Facing, cameras::CameraBlockBlueprint,
		players::thruster_block::ThrusterBlockBlueprint, prelude::*,
	};

	/// What is used to construct a [PlayerBundle]
	#[derive(Component, Reflect, Serialize, Deserialize, Debug)]
	pub struct PlayerBlueprintComponent {
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

		/// Synced
		pub(super) network_id: NetworkId,
	}

	impl PlayerBlueprintBundle {
		pub fn new(network_id: ClientId, transform: Transform) -> Self {
			PlayerBlueprintBundle {
				transform,
				network_id: NetworkId::from_raw(network_id.raw()),
				blueprint: PlayerBlueprintComponent {
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

	impl PlayerBlueprintComponent {
		// todo: impl spawn point semantics
		pub fn stamp(&self) -> <PlayerBlueprintComponent as Blueprint>::Bundle {
			Blueprint::stamp(self, &mut ())
		}
	}
}
mod player_bundle {
	use bevy::render::view::NoFrustumCulling;

	use crate::{players::player_movement::PlayerBundleMovementExt, prelude::*};

	use super::{ControllablePlayer, PlayerBlueprintBundle, PlayerBlueprintComponent};

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
				structure_children: _,
				thruster_children: _,
				primary_camera: _,
			} = self;
			Self::Bundle {
				spatial: Default::default(),
				name: Name::new("Player"),
				controllable_player: ControllablePlayer,
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

		type SpawnSystemParam = ();
	}
}
