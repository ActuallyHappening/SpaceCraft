use std::borrow::Cow;

use crate::prelude::*;

use extension_traits::extension;
use serde::de::DeserializeOwned;

// mod text;

pub mod scenes;
#[cfg(test)]
mod testing;
#[cfg(test)]
pub use testing::*;

/// Represents a type [Blueprint] that can be [Blueprint::stamp]ed into
/// a bundle that can be spawned, i.e., a [Bundle] that is specifically
/// [Blueprint::For]
pub trait Blueprint: Clone {
	/// The bundle type that this blueprint can be stamped into.
	type Bundle: Bundle;
	/// A way to access the world when stamping, typically [MMA],
	/// for things like [AssetServer] or [ResMut<Assets<Mesh>>].
	type StampSystemParam<'w, 's>: SystemParam;

	/// Stamps this blueprint into a bundle that can be spawned.
	fn stamp(&self, system_param: &mut Self::StampSystemParam<'_, '_>) -> Self::Bundle;
}

/// A blueprint that is synced over the network.
/// Hence, it must be serializable and deserializable,
/// and a component so that [bevy_replicon] can sync it.
pub trait ExpandableBlueprint: Blueprint + Component + Serialize + DeserializeOwned  {
	/// What access is needed when expanding this blueprint.
	type SpawnSystemParam: SystemParam;

	/// The system that expands this blueprint on both server and client side.
	fn expand_system(instances: Query<&Self, Changed<Self>>, system_param: &mut Self::SpawnSystemParam);
}

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

pub fn vec3_polar(horizontal_xz: f32, altitude_y: f32) -> Vec3 {
	Vec3 {
		x: altitude_y.cos() * horizontal_xz.cos(),
		y: altitude_y.sin(),
		z: altitude_y.cos() * horizontal_xz.sin(),
	}.normalize()
}

#[cfg(test)]
mod polar_tests {
	use std::f32::consts::TAU;

	use bevy::math::Vec3;
	use rand::random;
	use assert_float_eq::*;

	use crate::prelude::vec3_polar;

	macro_rules! assert_vec3_near {
		// delegates each component to assert_f32_near
		($a:expr, $b:expr) => {
			assert_float_absolute_eq!($a.x, $b.x, 0.01);
			assert_float_absolute_eq!($a.y, $b.y, 0.01);
			assert_float_absolute_eq!($a.z, $b.z, 0.01);
		};
	}

	#[test]
	fn edges() {
		assert_eq!(vec3_polar(0.0, 0.0), Vec3::X);
		assert_vec3_near!(vec3_polar(random(), TAU / 4.), Vec3::Y);
		assert_vec3_near!(vec3_polar(random(), -TAU / 4.), -Vec3::Y);
	}
}

pub fn vec3_polar_random(rng: &mut ThreadRng) -> Vec3 {
	let phi = rng.gen_range(0. ..TAU);
	let z: f32 = rng.gen_range(-1. ..1.);
	let theta = z.acos();

	let ret = vec3_polar(theta, phi);

	ret.normalize()
}

pub trait GetNetworkId {
	fn get_network_id(&self) -> ClientId;
}

pub trait GetBlockId {
	fn get_block_id(&self) -> BlockId;
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
