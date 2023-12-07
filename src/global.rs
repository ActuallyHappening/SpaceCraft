//! Types and functionality that affect the project globally
//! Like constants

use strum::IntoEnumIterator;

use crate::prelude::*;

pub mod assets;

pub const DEFAULT_PORT: u16 = 5069;
pub const PROTOCOL_ID: u64 = 0;
pub const PIXEL_SIZE: f32 = 1.; // how many pixels per block

/// Configured for [FixedUpdate] ONLY!
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GlobalSystemSet {
	BlueprintExpansion,

	/// Player's movement, set in [crate::players::player_movement]
	PlayerMovement,

	/// Player's thrusters syncing, set in
	/// [crate::players::player]
	PlayerThrusters,

	/// Runs physics simulation
	/// Note: Thrusters sync their data with external force just before this
	RawPhysics,

	/// Runs after physics simulation
	GameLogic,
}

/// Makes sure that the blueprints that the player creates are
/// expanded before, so that thruster visuals can be spawned on time
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlueprintExpansion {
	Player,
	Thruster,
}

/// Naming of all render layers used within the project
pub enum GlobalRenderLayers {
	/// Default render layers, used for in game since most entity logic assumes it is in game anyway
	InGame,

	/// Only showing entities relevant to UI, based on the camera intended to render them
	Ui(UiCameras),
}

impl From<GlobalRenderLayers> for RenderLayers {
	fn from(value: GlobalRenderLayers) -> Self {
		match value {
			GlobalRenderLayers::InGame => RenderLayers::default(),
			GlobalRenderLayers::Ui(cam_order) => RenderLayers::none().with(match cam_order {
				UiCameras::TopLeft => 1,
				UiCameras::TopMiddle => 2,
				UiCameras::TopRight => 3,
				UiCameras::MiddleLeft => 4,
				UiCameras::Center => 5,
				UiCameras::MiddleRight => 6,
				UiCameras::BottomLeft => 7,
				UiCameras::BottomMiddle => 8,
				UiCameras::BottomRight => 9,
			}),
		}
	}
}

/// Handles distribution of the camera orders.
/// This currently only serves the [`crate::ui`] module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalCameraOrders {
	/// Default camera order, used for in game
	InGame,

	/// Ui cameras, used in [crate::ui]
	Ui(UiCameras),
}

/// Enum of all ui cameras
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Reflect)]
pub enum UiCameras {
	TopLeft,
	TopMiddle,
	TopRight,
	MiddleLeft,
	Center,
	MiddleRight,
	BottomLeft,
	BottomMiddle,
	BottomRight,
}

impl From<GlobalCameraOrders> for isize {
	fn from(value: GlobalCameraOrders) -> Self {
		match value {
			// default / lowest
			GlobalCameraOrders::InGame => 0,
			GlobalCameraOrders::Ui(ui_cam) => match ui_cam {
				UiCameras::TopLeft => 1,
				UiCameras::TopMiddle => 2,
				UiCameras::TopRight => 3,
				UiCameras::MiddleLeft => 4,
				UiCameras::Center => 5,
				UiCameras::MiddleRight => 6,
				UiCameras::BottomLeft => 7,
				UiCameras::BottomMiddle => 8,
				UiCameras::BottomRight => 9,
			},
		}
	}
}

impl UiCameras {
	pub fn iter() -> impl Iterator<Item = Self> {
		<UiCameras as IntoEnumIterator>::iter()
	}
}
