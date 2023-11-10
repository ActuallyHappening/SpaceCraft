use crate::prelude::*;

/// Plugin Group
pub struct UiPlugins;

impl PluginGroup for UiPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(self::startscreen::StartScreen)
			.build()
	}
}

mod ui_cameras {
	use crate::{global::GlobalCameraOrders, prelude::*};

	/// Component of UiCameras that are 2D
	#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
	pub struct UiCamera {
		order: GlobalCameraOrders,
	}

	// pub trait CamType {
	// 	fn get_camera_transform(half_width: f32, half_height: f32) -> UVec2;
	// }

	impl GlobalCameraOrders {
		fn get_camera_transform(self, half_width: f32, half_height: f32) -> UVec2 {
			match self {
				Self::TopLeft => UVec2::new(half_width as u32, half_height as u32),
				_ => panic!(),
			}
		}
	}
}

mod startscreen {
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
			}
		}
	}
}
