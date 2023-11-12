use bevy::asset::AssetPath;
use meshtext::MeshGenerator;

#[derive(Debug, Clone)]
pub enum GlobalFont {
	Default,
}

impl From<GlobalFont> for AssetPath<'_> {
	fn from(value: GlobalFont) -> Self {
		match value {
			GlobalFont::Default => AssetPath::from("fonts/FiraMono-Medium.ttf"),
		}
	}
}

impl From<GlobalFont> for MeshGenerator<meshtext::Face<'_>> {
	fn from(value: GlobalFont) -> Self {
		match value {
			GlobalFont::Default => MeshGenerator::new(include_bytes!("../../assets/fonts/FiraMono-Medium.ttf")),
		}
	}
}
