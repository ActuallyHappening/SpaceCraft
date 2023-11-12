mod global;
mod player;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

#[allow(unused_imports)]
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};
#[allow(unused_imports)]
use bevy_mod_picking::{
	prelude::{DebugPickingPlugin, DefaultHighlightingPlugin},
	DefaultPickingPlugins,
};
use player::PlayerPlugins;
use ui::UiPlugins;

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

		// spawn initial Main Camera
		app.add_systems(Startup, |mut commands: Commands| {
			commands
				.spawn((
					Camera3dBundle {
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
					},
					// BloomSettings {
					// 	intensity: 1.0,
					// 	low_frequency_boost: 0.5,
					// 	low_frequency_boost_curvature: 0.5,
					// 	high_pass_frequency: 0.5,
					// 	prefilter_settings: BloomPrefilterSettings {
					// 		threshold: 3.0,
					// 		threshold_softness: 0.6,
					// 	},
					// 	composite_mode: BloomCompositeMode::Additive,
					// },
				))
				.insert(VisibilityBundle::default())
				.named("Main Camera")
				.render_layer(GlobalRenderLayers::InGame);
		});

		// states
		app.add_state::<GlobalGameStates>();

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
