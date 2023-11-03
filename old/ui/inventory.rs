use crate::utils::*;

use super::macros::*;

#[derive(Component, Constructor)]
pub struct PlayerInventoryText {
	variant: PixelVariant,
}

pub fn update_inventory_ui(
	mut invent_texts: Query<(&mut Text, &PlayerInventoryText)>,
	inventory: Res<PlayerInventory>,
) {
	for (mut text, PlayerInventoryText { variant }) in invent_texts.iter_mut() {
		if variant.get_variant_info().collectable.is_some() {
			text.sections[1].value = format!("{}", inventory[*variant]);
		}
	}
}

impl PlayerInventory {
	pub fn ui(parent: &mut ChildBuilder, ass: &Res<AssetServer>) {
		let text_style = TextStyle {
			font: ass.load(Font::Medium.into_path()),
			font_size: 30.,
			color: Color::BLUE,
		};

		for pixel in PixelVariant::get_mineable_variants().map(|pv| pv.get_default_pixel()) {
			parent.spawn(
				(
					TextBundle::from_sections([
						TextSection::new(format!("{}: ", pixel.name), text_style.clone()),
						TextSection::new("0", text_style.clone()),
					])
					.with_style(style! {Style
						margin: 5 px,
						width: 100%,
						height: 100%,
						flex_grow: 1,
					}),
					PlayerInventoryText::new(pixel.variant),
				)
					.named(format!("{} count", pixel.name))
					.not_pickable(),
			);
		}
	}
}
