use ::bevy::{log::LogPlugin, prelude::*, window::WindowMode};
use bevy_replicon::prelude::*;
use bevy_replicon::ReplicationPlugins;
use creativity_game::*;
use serde::Serialize;
use std::net::*;
use std::time::*;

use creativity_game::utils::*;
fn main() {
	let mut app = App::new();

	app
		.add_plugins(
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						fit_canvas_to_parent: true,
						prevent_default_event_handling: false,
						canvas: Some("#canvas".to_string()),
						title: "Creativity Game".to_string(),
						mode: WindowMode::BorderlessFullscreen,
						..default()
					}),
					..default()
				})
				.set(LogPlugin {
					level: bevy::log::Level::WARN,
					filter: "creativity_game=trace,bevy_ecs=info,bevy_replicon=debug".into(),
				})
				.build(),
		)
		.add_plugins((
			creativity_game::MainPlugin,
		))
		.run();
}
