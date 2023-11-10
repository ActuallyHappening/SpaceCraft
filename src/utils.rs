use crate::prelude::*;

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