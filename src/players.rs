use crate::prelude::*;

mod blocks;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(player::PlayerPlugin)
			.build()
	}
}

pub use player::PlayerBlueprint;

mod player {
	use super::blocks::manual_builder::Facing;
	use super::blocks::{BlockBlueprint, BlockId, StructureBlock};
	use super::thruster_block::ThrusterBlock;
	use crate::players::blocks::StructureBlockBundle;
	use crate::players::thruster_block::ThrusterBlockBundle;
use crate::prelude::*;

	pub struct PlayerPlugin;
	impl Plugin for PlayerPlugin {
		fn build(&self, app: &mut App) {
			app.replicate::<PlayerBlueprint>().add_systems(
				FixedUpdate,
				Self::handle_spawn_player_blueprints.in_set(GlobalSystemSet::BlueprintExpansion("player")),
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
		collider: AsyncCollider,
		name: Name,
		controllable_player: ControllablePlayer,
	}

	#[derive(Component)]
	struct ControllablePlayer {
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
				collider: AsyncCollider(ComputedCollider::ConvexHull),
				replication: Replication,
			}
		}
	}
}

mod thruster_block {
	use crate::prelude::*;

	use super::blocks::{manual_builder, BlockBlueprint};

	/// Will spawn a particle emitter as a child
	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub struct ThrusterBlock;

	#[derive(Bundle)]
	pub struct ThrusterBlockBundle {
		pbr: PbrBundle,
		collider: AsyncCollider,
		name: Name,
	}

	impl FromBlueprint for ThrusterBlockBundle {
		type Blueprint = BlockBlueprint<ThrusterBlock>;

		fn stamp_from_blueprint(
			BlockBlueprint {
				transform,
				mesh,
				material,
				specific_marker,
			}: &Self::Blueprint,
			mma: &mut MMA,
		) -> Self {
			Self {
				pbr: PbrBundle {
					transform: *transform,
					mesh: mesh.get_mesh(mma),
					material: material.get_material(&mut mma.mats),
					..default()
				},
				collider: AsyncCollider(ComputedCollider::ConvexHull),
				name: Name::new("ThrusterBlock"),
			}
		}
	}

	impl BlockBlueprint<ThrusterBlock> {
		pub fn new_thruster(location: manual_builder::RelativePixel, facing: impl Into<Quat>) -> Self {
			let rotation = facing.into();
			BlockBlueprint {
				transform: Transform {
					translation: location.as_vec3() * PIXEL_SIZE
						+ Transform::from_rotation(rotation).forward() * PIXEL_SIZE / 2.,
					rotation,
					..default()
				},
				mesh: super::blocks::OptimizableMesh::CustomRectangularPrism {
					size: Vec3::splat(PIXEL_SIZE / 2.),
				},
				material: super::blocks::OptimizableMaterial::OpaqueColour(Color::RED),
				specific_marker: ThrusterBlock,
			}
		}
	}
}
