use crate::utils::*;

#[derive(Event, Debug, Deref)]
pub struct PlayerMinedPixel(Pixel);

impl PlayerMinedPixel {
	pub fn new(pixel: Pixel) -> Option<Self> {
		if pixel.variant.get_variant_info().is_player_mineable() {
			Some(Self(pixel))
		} else {
			None
		}
	}
}
