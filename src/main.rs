#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{log::LogPlugin, prelude::*, window::WindowMode};

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
						visible: false,
						..default()
					}),
					..default()
				})
				.set(LogPlugin {
					level: bevy::log::Level::WARN,
					filter:
						"space_craft=trace,bevy_ecs=info,bevy_replicon=debug,renet=debug,bevy_xpbd_3d=debug"
							// ""
							.into(),
				})
				.build(),
		)
		.add_plugins((space_craft::MainPlugin,));

	#[cfg(target_os = "windows")]
	app.add_systems(Startup, set_window_icon);

	app.run();
}

// copied from https://github.com/NiklasEi/bevy_game_template/blob/d786f979ea023d557373b373edadd3b027451b1a/src/main.rs#L35
// Sets the icon on windows and X11
#[cfg(target_os = "windows")]
fn set_window_icon(
	windows: NonSend<bevy::winit::WinitWindows>,
	primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
) {
	use std::io::Cursor;
	use winit::window::Icon;
	let primary_entity = primary_window.single();
	let primary = windows.get_window(primary_entity).unwrap();
	let icon_buf = Cursor::new(include_bytes!(
		"../build/macos.app/Contents/AppIcon.iconset/icon_256x256.png"
	));
	if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
		let image = image.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		let icon = Icon::from_rgba(rgba, width, height).unwrap();
		primary.set_window_icon(Some(icon));
	};
}
