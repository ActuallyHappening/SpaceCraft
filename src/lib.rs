mod global;
mod player;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

use player::PlayerPlugins;
use ui::UiPlugins;
#[allow(unused_imports)]
use bevy_mod_picking::{DefaultPickingPlugins, prelude::{DefaultHighlightingPlugin, DebugPickingPlugin}};

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

		app.add_systems(Startup, |mut commands: Commands| {
			commands
				.spawn(Camera3dBundle {
					transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
					camera: Camera {
						hdr: true,
						..default()
					},
					camera_3d: Camera3d {
						clear_color: ClearColorConfig::Custom(Color::BLACK),
						..default()
					},
					tonemapping: Tonemapping::None,
					..default()
				})
				.insert(VisibilityBundle::default())
				.named("Main Camera")
				.render_layer(GlobalRenderLayers::InGame);
		});

		app.add_plugins((
			bevy_editor_pls::EditorPlugin::default(),
			ScreenDiagnosticsPlugin::default(),
			ScreenFrameDiagnosticsPlugin,
			DefaultPickingPlugins
				.build()
				// .disable::<DebugPickingPlugin>()
				.disable::<DefaultHighlightingPlugin>(),
			HanabiPlugin,
			PlayerPlugins,
			UiPlugins,
		));
	}
}
