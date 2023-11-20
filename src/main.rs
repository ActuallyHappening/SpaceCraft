use bevy::{prelude::*, window::WindowMode, log::LogPlugin};

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
					filter: "creativity_game=trace,bevy_ecs=info,bevy_replicon=debug,renet=debug,bevy_xpbd_3d=debug".into(),
				})
				.build(),
		)
		.add_plugins((creativity_game::MainPlugin,))
		.run();
}
