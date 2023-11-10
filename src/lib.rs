mod player;
mod prelude;
mod states;
mod ui;
mod world;
mod global;
mod utils;

use player::PlayerPlugins;
use ui::UiPlugins;

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((PlayerPlugins, UiPlugins));
	}
}
