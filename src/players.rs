use crate::prelude::*;

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
	use super::blocks::{BlockBlueprint, BlockId, StructureBlock};
	use super::thruster::ThrusterBlock;
	use crate::prelude::*;

	pub struct PlayerPlugin;
	impl Plugin for PlayerPlugin {
		fn build(&self, app: &mut App) {
			app
				.add_server_event::<PlayerBlueprint>(EventType::Unordered)
				.add_systems(
					FixedUpdate,
					Self::handle_spawn_player_blueprints
						.in_set(GlobalSystemSet::BlueprintExpansion("player")),
				);
		}
	}

	/// Sent as an event to all clients, then expanded into a full player bundle
	#[derive(Event, Serialize, Deserialize)]
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
				structure_children: vec![],
				thruster_children: vec![],
			}
		}
	}

	/// Parent entity of a player
	#[derive(Bundle)]
	struct PlayerBundle {
		spatial: VisibilityBundle,
		replication: Replication,
		collider: AsyncCollider,
	}

	#[derive(Component)]
	struct ControllablePlayer {
		network_id: ClientId,
		movement_input: HashMap<BlockId, f32>,
	}

	impl PlayerPlugin {
		fn handle_spawn_player_blueprints(
			mut events: EventReader<PlayerBlueprint>,
			mut commands: Commands,
		) {
		}
	}
}

mod thruster {
	use crate::prelude::*;

	/// Will spawn a particle emitter as a child
	#[derive(Serialize, Deserialize)]
	pub struct ThrusterBlock;
}

mod blocks {
	use bevy::asset::AssetPath;

	use crate::prelude::*;

	/// The unique identifier for a persistent block in the world
	#[derive(Reflect, Debug, Clone, Copy, Component)]
	#[reflect(Component)]
	pub struct BlockId(u128);

	impl Default for BlockId {
		fn default() -> Self {
			warn!("Generating default BlockId!!");
			Self(0)
		}
	}

	/// Represents a 'block', which is useful for spawning standardized structures
	/// like thrusters.
	///
	/// Assumes all blocks have a position, material and mesh. These restrictions may be lifted
	/// if ever there was a need.
	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockBlueprint<T> {
		pub transform: Transform,
		pub mesh: OptimizableMesh,
		pub material: OptimizableMaterial,
		pub specific_marker: T,
	}

	#[derive(Serialize, Deserialize)]
	pub struct StructureBlock;

	/// Since raw [Mesh] cannot be serialized
	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub enum OptimizableMesh {
		StandardBlock,
		FromAsset(String),
	}

	impl OptimizableMesh {
		pub fn into_mesh(&self, ass: &mut AssetServer) -> Handle<Mesh> {
			match self {
				Self::FromAsset(name) => ass.load(name),
				Self::StandardBlock => ass.add(shape::Cube { size: PIXEL_SIZE }.into()),
			}
		}
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub enum OptimizableMaterial {
		OpaqueColour(Color),
	}

	impl OptimizableMaterial {
		pub fn into_material(&self, mat: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
			match self {
				Self::OpaqueColour(col) => mat.add((*col).into()),
			}
		}
	}
}
