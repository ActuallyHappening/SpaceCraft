use bevy::prelude::*;

use crate::core::CorePlugin;

use self::{camera::CameraPlugin, player::PlayerPlugin, setup::SetupPlugin, ui::UiPlugins};

mod camera;
mod player;
pub mod renet;
mod setup;
mod ui;

pub use player::types;
pub use player::WeaponFlags;

pub use renet::ClientID;

pub struct MainPlugin;
impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			self::renet::RenetPlugin,
			CorePlugin,
			SetupPlugin,
			PlayerPlugin,
			CameraPlugin,
			UiPlugins.build(),
		));
	}
}
