use bevy::{winit::WinitPlugin, DefaultPlugins, app::{App, PluginGroup}};

/// Returns an app that can run basic schedules
pub fn test_app() -> App {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());

	app
}
