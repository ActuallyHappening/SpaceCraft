use crate::utils::*;

mod structures;
pub use structures::*;
mod macros;
use macros::*;
mod world_gen;
pub use world_gen::*;

/// Data about a class of pixels
/// Does not implement [PartialEq] because the identity of a pixel is only in its variant,
/// spawning default pixels does not imply that all default pixels are the same,
/// even though all of the information contained within this struct would imply that
/// [PartialEq] they are equal.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Pixel {
	pub name: Cow<'static, str>,
	pub description: Cow<'static, str>,
	pub colour: Color,
	pub variant: PixelVariant,
}

#[derive(Debug, Clone)]
pub struct Natural {
	/// Higher the number, greater chance of spawning
	pub frequency: u16,
}

#[derive(Debug, Clone)]
pub struct Collect {
	pub player_mineable: bool,
	pub amount_multiplier: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum PixelVariant {
	Dirt,
	Copper,
	Lead,

	/// Used to create player
	PlayerSteel,
	/// Used for player engine
	PlayerLargeEngineDecoration,
}

pub struct PixelVariantInfo {
	pub collectable: Option<Collect>,
	pub naturally_spawning: Option<Natural>,
}

impl PixelVariant {
	// todo: maybe put this info in a lazy_static and reference it instead of re-constructing it everywhere?
	fn default_hardcoded(self) -> (Pixel, PixelVariantInfo) {
		type PV = PixelVariant;
		match self {
			PV::Dirt => pixel_type! {self,
				name: "Dirt",
				description: "Some dirt with no life in it",
				colour: Color::rgb(0.3, 0.25, 0.),
				collectable: None,
				naturally_spawning: Some(Natural { frequency: 1000 }),
			},
			PV::Copper => pixel_type! {self,
				name: "Copper",
				description: "A block of copper",
				colour: Color::rgb(0.6, 0.25, 0.05),
				collectable: Some(Collect {
					amount_multiplier: 5,
					player_mineable: true,
				}),
				naturally_spawning: Some(Natural { frequency: 150 }),
			},
			PV::Lead => pixel_type! {self,
				name: "Lead",
				description: "A block of lead",
				colour: Color::SILVER,
				collectable: Some(Collect {
					amount_multiplier: 1,
					player_mineable: true,
				}),
				naturally_spawning: Some(Natural { frequency: 3 }),
			},
			PV::PlayerSteel => pixel_type! {self,
				name: "Player Steel",
				description: "Steel used in the construction of the MainPlayer",
				colour: Color::SILVER,
				collectable: None,
				naturally_spawning: None,
			},
			PV::PlayerLargeEngineDecoration => pixel_type!(self,
				name: "Player Engine",
				description: "A decoration block",
				colour: Color::RED,
				collectable: None,
				naturally_spawning: None,
			),
		}
	}

	/// From/Into also implemented, but prefer explicit methods
	pub fn get_default_pixel(self) -> Pixel {
		impl From<PixelVariant> for Pixel {
			fn from(variant: PixelVariant) -> Self {
				variant.get_default_pixel()
			}
		}

		self.default_hardcoded().0
	}

	/// From/Into also implemented, but prefer explicit methods
	pub fn get_variant_info(self) -> PixelVariantInfo {
		impl From<PixelVariant> for PixelVariantInfo {
			fn from(variant: PixelVariant) -> Self {
				variant.get_variant_info()
			}
		}

		self.default_hardcoded().1
	}

	pub fn natural_pool() -> Vec<(PixelVariant, Natural)> {
		let mut pool = Vec::new();
		for variant in Self::iter() {
			if let Some(natural) = variant.get_variant_info().naturally_spawning {
				pool.push((variant, natural));
			}
		}
		pool
	}
}

impl PixelVariant {
	pub fn iter() -> impl Iterator<Item = PixelVariant> {
		<PixelVariant as strum::IntoEnumIterator>::iter()
	}

	/// Returns all variants that are mineable by the player
	pub fn get_mineable_variants() -> impl Iterator<Item = PixelVariant> {
		PixelVariant::iter().filter(|v| v.get_variant_info().is_player_mineable())
	}
}

impl PixelVariantInfo {
	pub fn is_player_mineable(&self) -> bool {
		self
			.collectable
			.as_ref()
			.is_some_and(|collect| collect.player_mineable)
	}
}
