macro_rules! pixel_type {
	($self:ident, name: $name:literal, description: $description:literal, colour: $colour:expr, collectable: $collectable:expr, naturally_spawning: $naturally_spawning:expr, ) => {
		(
			Pixel {
				name: $name.into(),
				description: $description.into(),
				colour: $colour,
				variant: $self,
			},
			PixelVariantInfo {
				collectable: $collectable,
				naturally_spawning: $naturally_spawning,
			},
		)
	};
}

pub(crate) use pixel_type;
