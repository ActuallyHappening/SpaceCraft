pub use bevy::render::view::RenderLayers;

use crate::utils::*;

#[derive(Debug)]
pub enum RenderLayer {
	Stars,
	Designing,
}

impl From<RenderLayer> for RenderLayers {
	fn from(value: RenderLayer) -> Self {
		match value {
			RenderLayer::Stars => RenderLayers::none().with(1),
			RenderLayer::Designing => RenderLayers::none().with(2),
		}
	}
}

#[extension(pub trait BundleExt2)]
impl<T: Bundle> T {
	fn render_layer(self, layer: impl Into<RenderLayers>) -> (RenderLayers, Self) {
		self.insert(layer.into())
	}
}
