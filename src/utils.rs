use std::borrow::Cow;

use crate::prelude::*;

use extension_traits::extension;

// mod text;

pub mod scenes;
#[cfg(test)]
mod testing;
#[cfg(test)]
pub use testing::*;

#[extension(pub trait AppExt)]
impl &mut App {
	#[allow(unused_variables)]
	fn depends_on<T: Plugin, M>(self, plugin_group: impl Plugins<M>) -> Self {
		if self.is_plugin_added::<T>() {
			self
		} else {
			#[cfg(not(test))]
			panic!("Plugin {:?} is already added", std::any::type_name::<T>());

			#[cfg(test)]
			self.add_plugins(plugin_group);
			// #[cfg(test)]
			// debug!("Adding plugin group {:?} because .depends_on was called", std::any::type_name::<U>());
			#[cfg(test)]
			self
		}
	}
}

pub fn vec3_polar(theta: f32, phi: f32) -> Vec3 {
	Vec3 {
		x: theta.sin() * phi.cos(),
		y: theta.sin() * phi.sin(),
		z: theta.cos(),
	}
}

pub fn vec3_polar_random(rng: &mut ThreadRng) -> Vec3 {
	let phi = rng.gen_range(0. ..TAU);
	let z: f32 = rng.gen_range(-1. ..1.);
	let theta = z.acos();

	let ret = vec3_polar(theta, phi);

	ret.normalize()
}

pub trait FromBlueprint {
	type Blueprint;

	fn stamp_from_blueprint(blueprint: &Self::Blueprint, mma: &mut MMA) -> Self;
}

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

#[extension(pub trait TransformExt)]
impl Transform {
	fn translate(mut self, delta: Vec3) -> Self {
		self.translation += delta;
		self
	}
}
