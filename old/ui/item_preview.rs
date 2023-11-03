use bevy::{
	core_pipeline::clear_color::ClearColorConfig, render::camera::Viewport, window::WindowResized,
};

use crate::bevy::camera::MainCamera;

use super::*;

/// A square [ItemPreview::WIDTH_PERCENT] of the screen
pub struct ItemPreview;
impl Plugin for ItemPreview {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Startup, |mut commands: Commands| {
				commands.spawn(
					Camera3dBundle {
						transform: Transform::from_xyz(0., CAMERA_HEIGHT, 0.).with_rotation(*INITIAL_ROT),
						camera: Camera {
							order: 1,
							..default()
						},
						camera_3d: Camera3d {
							clear_color: ClearColorConfig::None,
							..default()
						},
						..default()
					}
					.insert(PreviewCamera)
					.not_pickable(),
				);
			})
			.add_systems(Update, update_preview_cam);
	}
}

#[derive(Component)]
struct PreviewCamera;

lazy_static::lazy_static! {
	static ref INITIAL_ROT: Quat = Quat::from_rotation_x(-90f32.to_radians());
}

impl ItemPreview {
	/// Width of viewport taken by item preview
	const WIDTH_PERCENT: u32 = 20;
	/// Around on all sides of preview, used to make preview wit within the *border*
	/// CSS property in style!{} below in UI code
	const MARGIN: u32 = 10;

	// TODO Implement
	const EDGE_OFFSET: u32 = 0;

	/// Flex location of preview, assuming placed in top right of screen
	/// with no margin between this and viewport border
	pub fn ui(parent: &mut ChildBuilder) {
		// ui for preview with border
		parent.spawn(
			NodeBundle {
				style: style! {Style
					// aspect_ratio: 1,
					border: 5 px,
					margin: 0 px,
				}
				.with_width_vw(Self::WIDTH_PERCENT as f32)
				.with_height_vw(Self::WIDTH_PERCENT as f32),
				border_color: Color::BLACK.into(),
				// background_color: Color::PURPLE.into(),
				..default()
			}
			.not_pickable()
			.named("Item Preview UI"),
		);
		// todo: maybe use right: EDGE_OFFSET and top EDGE_OFFSET plus display: absolute to get an edge offset for preview ui
		// .with_children(|parent| {
		// 	parent.spawn(NodeBundle {
		// 		style: style!{Style
		// 			width: 100%,
		// 			height: 100%,

		// 			margin: 5 px,
		// 			border: 5 px,
		// 		},
		// 		border_color: Color::BLACK.into(),
		// 		..default()
		// 	}.named("Item preview"));
		// });
	}
}

fn update_preview_cam(
	windows: Query<&Window>,
	mut resize_events: EventReader<WindowResized>,
	mut cam: Query<&mut Camera, (With<PreviewCamera>, Without<MainCamera>)>,
) {
	// We need to dynamically resize the camera's viewports whenever the window size changes
	// A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
	for resize_event in resize_events.iter() {
		let window = windows.get(resize_event.window).unwrap();
		let mut preview_cam = cam.single_mut();

		let width = window.resolution.physical_width();
		let _height = window.resolution.physical_height();
		let preview_width = ((ItemPreview::WIDTH_PERCENT as f32 / 100.) * width as f32).round() as u32
			- ItemPreview::MARGIN * 2;
		let top_left_x = width - preview_width - ItemPreview::MARGIN - ItemPreview::EDGE_OFFSET;
		let top_left_y = ItemPreview::MARGIN + ItemPreview::EDGE_OFFSET;

		preview_cam.viewport = Some(Viewport {
			physical_position: UVec2::new(top_left_x, top_left_y),
			physical_size: UVec2::new(preview_width, preview_width),
			..default()
		});

		// let mut right_camera = right_camera.single_mut();
		// right_camera.viewport = Some(Viewport {
		//     physical_position: UVec2::new(window.resolution.physical_width() / 2, 0),
		//     physical_size: UVec2::new(
		//         window.resolution.physical_width() / 2,
		//         window.resolution.physical_height(),
		//     ),
		//     ..default()
		// });
	}
}
