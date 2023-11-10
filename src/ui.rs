use crate::prelude::*;

/// Plugin Group
pub struct UiPlugins;

impl PluginGroup for UiPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(self::start_screen::StartScreen)
			.add(self::ui_cameras::UiCamerasPlugin)
			.build()
	}
}

mod ui_cameras {
	//! Manages spawning and responding to screen re-size events for
	//! all ui cameras

	use crate::prelude::*;

	/// Plugin
	pub struct UiCamerasPlugin;

	impl Plugin for UiCamerasPlugin {
		fn build(&self, app: &mut App) {
			app
				.add_systems(Startup, Self::spawn_ui_cameras)
				.add_systems(Update, update_cameras);
		}
	}

	#[derive(Bundle)]
	struct UiCameraBundle {
		inner_camera: Camera2dBundle,
		render_layer: RenderLayers,
		variant: UiCamera,

		vis: VisibilityBundle,
	}

	impl UiCamerasPlugin {
		fn spawn_ui_camera(cam: UiCamera, commands: &mut Commands) {
			commands.spawn(UiCameraBundle {
				inner_camera: Camera2dBundle {
					camera: Camera {
						order: GlobalCameraOrders::Ui(cam.variant).into(),
						..default()
					},
					..default()
				},
				render_layer: GlobalRenderLayers::Ui(cam.variant).into(),
				variant: cam,
				vis: Default::default(),
			});
		}

		fn spawn_ui_cameras(mut commands: Commands) {
			Self::spawn_ui_camera(
				UiCamera {
					variant: UiCameras::Center,
				},
				&mut commands,
			)
		}
	}

	/// Component of UiCameras that are 2D
	#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
	pub struct UiCamera {
		variant: UiCameras,
	}

	impl UiCamera {
		fn get_camera_transform(&self, half_width: f32, half_height: f32) -> UVec2 {
			match self.variant {
				UiCameras::TopLeft => UVec2::new(half_width as u32, half_height as u32),
				UiCameras::Center => UVec2::new(0, 0),
				_ => panic!(),
			}
		}
	}

	fn update_cameras(
		windows: Query<&Window>,
		mut resize_events: EventReader<bevy::window::WindowResized>,
		mut cam: Query<(&mut Transform, &UiCamera)>,
	) {
		for ev in resize_events.read() {
			let window = windows.get(ev.window).unwrap();
			let (mut cam, variant) = cam.single_mut();

			let width = window.resolution.width();
			let height = window.resolution.height();

			let cam_translation = variant.get_camera_transform(width / 2., height / 2.);
			cam.translation = Vec3::new(cam_translation.x as f32, cam_translation.y as f32, 0.);
		}
	}
}

mod start_screen {
	use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

	use crate::prelude::*;

	/// Plugin
	pub struct StartScreen;

	impl Plugin for StartScreen {
		fn build(&self, app: &mut App) {
			app
				.add_state::<StartScreenStates>()
				.add_systems(OnEnter(StartScreenStates::Initial), Self::spawn_initial);
		}
	}

	#[derive(States, Component, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
	enum StartScreenStates {
		#[default]
		Initial,

		ConfigureHost,

		ConfigureClient,

		ConfigureSolo,
	}

	impl StartScreen {
		fn spawn_initial(mut commands: Commands, mut mma: MM2) {
			commands.spawn(HostGameButtonBundle::new(&mut mma));
		}
	}

	#[derive(Bundle)]
	struct HostGameButtonBundle {
		mesh: Mesh2dHandle,
		material: Handle<ColorMaterial>,
		spatial: SpatialBundle,

		layer: RenderLayers,
	}

	impl HostGameButtonBundle {
		fn new(mma: &mut MM2) -> Self {
			Self {
				mesh: mma
					.meshs
					.add(shape::Quad::new(Vec2::new(100., 50.)).into())
					.into(),
				material: mma.mats.add(Color::WHITE.into()),
				spatial: Default::default(),
				layer: GlobalRenderLayers::Ui(UiCameras::Center).into(),
			}
		}
	}
}
