use crate::prelude::*;

use self::blocks::StructureBlock;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(PlayerPlugin)
			.build()
	}
}

struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, _app: &mut App) {}
}

/// Sent as an event to all clients, then expanded into a full player bundle
#[derive(Default, Event)]
struct PlayerBlueprint {
	transform: Transform,
	structure_childern: Vec<BlockBlueprint<StructureBlock>>,
	thruster_children: Vec<BlockBlueprint<ThrusterBlock>>,
}

use thruster::*;
mod thruster {
	use crate::prelude::*;

	/// Will spawn a particle emitter as a child
	pub struct ThrusterBlock;
}

use blocks::*;
mod blocks {
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
	/// like thrusters
	pub struct BlockBlueprint<T> {
		pub transform: Transform,
		pub mesh: OptimizableMesh,
		pub specific_marker: T,
	}

	pub struct StructureBlock;

	/// For efficiency,
	pub enum OptimizableMesh {
		Custom(Mesh),
		StandardBlock,
	}
}
