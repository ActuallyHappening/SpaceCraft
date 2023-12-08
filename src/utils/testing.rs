use bevy::{
	app::{App, PluginGroup},
	winit::WinitPlugin,
	DefaultPlugins,
};

/// Returns an app that can run basic schedules
pub fn test_app() -> App {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());

	app
}
