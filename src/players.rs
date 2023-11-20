use crate::prelude::*;

pub use player::ControllablePlayer;

mod thruster_block;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(player::PlayerPlugin)
			.add(thruster_block::ThrusterPlugin)
			.build()
	}
}

pub use player::PlayerBlueprint;

mod player {
	use super::thruster_block::ThrusterBlock;
	use crate::blocks::manual_builder::Facing;
	use crate::blocks::StructureBlockBundle;
	use crate::blocks::{BlockBlueprint, BlockId, StructureBlock};
	use crate::players::thruster_block::ThrusterBlockBundle;
	use crate::prelude::*;

	pub struct PlayerPlugin;
	impl Plugin for PlayerPlugin {
		fn build(&self, app: &mut App) {
			app.replicate::<PlayerBlueprint>().add_systems(
				FixedUpdate,
				Self::handle_spawn_player_blueprints.in_set(BlueprintExpansion::Player),
			);
		}
	}

	/// Sent as an event to all clients, then expanded into a full player bundle
	#[derive(Component, Serialize, Deserialize, Clone, Debug)]
	pub struct PlayerBlueprint {
		network_id: ClientId,
		transform: Transform,
		structure_children: Vec<BlockBlueprint<StructureBlock>>,
		thruster_children: Vec<BlockBlueprint<ThrusterBlock>>,
	}

	impl PlayerBlueprint {
		pub fn default_at(network_id: ClientId, transform: Transform) -> Self {
			PlayerBlueprint {
				network_id,
				transform,
				structure_children: vec![BlockBlueprint::new_structure(
					StructureBlock::Aluminum,
					IVec3::ZERO,
				)],
				thruster_children: vec![BlockBlueprint::new_thruster(
					IVec3::new(-1, 0, 0),
					Facing::Right,
				)],
			}
		}
	}

	/// Parent entity of a player.
	/// Doesn't actually have its own mesh
	#[derive(Bundle)]
	struct PlayerBundle {
		spatial: SpatialBundle,
		replication: Replication,
		mass: MassPropertiesBundle,
		body: RigidBody,
		name: Name,
		controllable_player: ControllablePlayer,
		external_force: ExternalForce,
	}

	/// The marker component for player entities
	#[derive(Component)]
	pub struct ControllablePlayer {
		network_id: ClientId,
		movement_input: HashMap<BlockId, f32>,
	}

	impl PlayerPlugin {
		fn handle_spawn_player_blueprints(
			player_blueprints: Query<(Entity, &PlayerBlueprint), Added<PlayerBlueprint>>,
			mut commands: Commands,
			mut mma: MMA,
		) {
			for (player, player_blueprint) in player_blueprints.iter() {
				debug!(
					"Expanding player blueprint for {:?}",
					player_blueprint.network_id
				);
				commands
					.entity(player)
					.insert(PlayerBundle::stamp_from_blueprint(
						player_blueprint,
						&mut mma,
					)).insert(ExternalForce::ZERO)
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
					});
			}
		}
	}

	impl FromBlueprint for PlayerBundle {
		type Blueprint = PlayerBlueprint;

		fn stamp_from_blueprint(
			PlayerBlueprint {
				network_id,
				transform,
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
