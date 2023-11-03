use bevy::prelude::*;

use crate::core::CorePlugin;

use self::{camera::CameraPlugin, player::PlayerPlugin, setup::SetupPlugin, ui::UiPlugins};

mod camera;
mod player;
mod netcode;
mod setup;
mod ui;

pub use player::types;
pub use player::WeaponFlags;

pub use netcode::ClientID;

pub struct MainPlugin;
impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			self::netcode::RenetPlugin,
			CorePlugin,
			SetupPlugin,
			PlayerPlugin,
			CameraPlugin,
			UiPlugins.build(),
		));
	}
}
