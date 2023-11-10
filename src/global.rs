//! Types and functionality that affect the project globally
//! Like constants

use crate::prelude::*;

/// Naming of all render layers used within the project
pub enum GlobalRenderLayers {
	/// Only showing entities relevant to UI, based on the camera intended to render them
	Ui(GlobalCameraOrders),
}

impl From<GlobalRenderLayers> for RenderLayers {
	fn from(value: GlobalRenderLayers) -> Self {
		match value {
			GlobalRenderLayers::Ui(cam_order) => RenderLayers::none().with(match cam_order {
				GlobalCameraOrders::TopLeft => 1,
				GlobalCameraOrders::TopRight => 2,
				GlobalCameraOrders::BottomLeft => 3,
				GlobalCameraOrders::BottomRight => 4,
				GlobalCameraOrders::Center => 5,
			}),
		}
	}
}

/// Handles distribution of the camera orders.
/// This currently only serves the crate::ui module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalCameraOrders {
	TopLeft,
	TopRight,
	BottomLeft,
	BottomRight,

	Center,
}

impl From<GlobalCameraOrders> for isize {
	fn from(value: GlobalCameraOrders) -> Self {
		match value {
			GlobalCameraOrders::TopLeft => 1,
			GlobalCameraOrders::TopRight => 2,
			GlobalCameraOrders::BottomLeft => 3,
			GlobalCameraOrders::BottomRight => 4,
			GlobalCameraOrders::Center => 5,
		}
	}
}
