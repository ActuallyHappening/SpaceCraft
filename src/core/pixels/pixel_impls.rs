use super::Pixel;
use crate::utils::*;

impl Pixel {
	pub fn from_raw(raw: RawPixel) -> Pixel {
		Pixel::Special(raw)
	}

	/// Creates a [Pixel] from a variant, assuming it is a default pixel with no special modifications / properties
	pub fn default_from_variant(variant: PixelVariant) -> Pixel {
		impl From<PixelVariant> for Pixel {
			fn from(value: PixelVariant) -> Self {
				Pixel::default_from_variant(value)
			}
		}

		Pixel::Default { variant }
	}

	/// todo: get this to return reference to [RawPixel] so that we don't have to clone it
	pub fn get_raw(&self) -> RawPixel {
		match self {
			Pixel::Default { variant } => variant.get_default_pixel(),
			Pixel::Special(raw) => raw.clone(),
		}
	}
}

impl Pixel {
	pub fn get_name(&self) -> Cow<'static, str> {
		self.get_raw().name
	}

	pub fn get_description(&self) -> Cow<'static, str> {
		self.get_raw().description
	}

	pub fn get_colour(&self) -> Color {
		self.get_raw().colour
	}

	pub fn get_variant(&self) -> PixelVariant {
		match self {
			Pixel::Default { variant } => *variant,
			Pixel::Special(raw) => raw.variant,
		}
	}

	pub fn get_variant_info(&self) -> PixelVariantInfo {
		self.get_variant().get_variant_info()
	}

	pub fn get_natural_frequency(&self) -> Option<u16> {
		self.get_variant_info().naturally_spawning.map(|n| n.frequency)
	}
}
