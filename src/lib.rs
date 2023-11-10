mod player;
mod prelude;
mod world;
mod ui;
mod states;

use player::PlayerPlugins;

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(PlayerPlugins);
	}
}
