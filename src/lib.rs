mod global;
mod player;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

use player::PlayerPlugins;
use ui::UiPlugins;

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			bevy_editor_pls::EditorPlugin::default(),
			PlayerPlugins,
			UiPlugins,
		));
	}
}
