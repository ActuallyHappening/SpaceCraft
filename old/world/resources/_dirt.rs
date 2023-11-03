use bevy::prelude::Color;

use super::_traits::impl_pixel_type;

pub struct RDirt;
impl_pixel_type!(
	RDirt { col = Color::YELLOW_GREEN, natural = { freq = 100, random = RDirt },}
);
