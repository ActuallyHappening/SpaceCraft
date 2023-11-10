use crate::prelude::*;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>().add(PlayerPlugin).build()
	}
}

struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, _app: &mut App) {}
}
