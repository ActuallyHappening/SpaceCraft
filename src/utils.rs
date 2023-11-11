use std::borrow::Cow;

use crate::prelude::*;

use extension_traits::extension;

#[allow(clippy::upper_case_acronyms)]
#[derive(SystemParam)]
pub struct MMA<'w> {
	pub meshs: ResMut<'w, Assets<Mesh>>,
	pub mats: ResMut<'w, Assets<StandardMaterial>>,
	pub ass: Res<'w, AssetServer>,
}
#[allow(clippy::upper_case_acronyms)]
#[derive(SystemParam)]
pub struct MM<'w> {
	pub meshs: ResMut<'w, Assets<Mesh>>,
	pub mats: ResMut<'w, Assets<StandardMaterial>>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(SystemParam)]
pub struct MMA2<'w> {
	pub meshs: ResMut<'w, Assets<Mesh>>,
	pub mats: ResMut<'w, Assets<ColorMaterial>>,
	pub ass: Res<'w, AssetServer>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(SystemParam)]
pub struct MM2<'w> {
	pub meshs: ResMut<'w, Assets<Mesh>>,
	pub mats: ResMut<'w, Assets<ColorMaterial>>,
}

#[extension(pub trait BundleExt)]
impl &mut EntityCommands<'_, '_, '_> {
	fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
		self.insert(Name::new(name))
	}

	fn render_layer(self, layer: impl Into<RenderLayers>) -> Self {
		self.insert(layer.into())
	}

	fn not_pickable(self) -> Self {
		self.insert(bevy_mod_picking::prelude::Pickable::IGNORE)
	}

	fn pickable(self) -> Self {
		self.insert(bevy_mod_picking::prelude::PickableBundle::default())
	}
}