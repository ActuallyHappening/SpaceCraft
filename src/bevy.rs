use bevy::prelude::*;

use crate::core::CorePlugin;

use self::{camera::CameraPlugin, player::PlayerPlugin, setup::SetupPlugin, ui::UiPlugins};

mod camera;
mod netcode;
mod player;
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

		debug!(
			"At the end of MainPlugin: client configs {:#?}",
			app
				.world
				.resource::<bevy_replicon::replicon_core::NetworkChannels>()
				.get_client_configs()
		);
	}
}
