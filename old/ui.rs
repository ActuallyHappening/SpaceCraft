use bevy::prelude::*;

use crate::utils::*;

mod macros;
use macros::*;
mod inventory;
mod item_preview;
use inventory::*;

use self::item_preview::ItemPreview;

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(ItemPreview)
			// .add_systems(Startup, ui)
			// .add_systems(Update, update_inventory_ui);
			;
	}
}

fn ui(mut commands: Commands, mma: MMA) {
	commands
		.spawn(
			NodeBundle {
				// style: Style {
				// 	width: Val::Percent(100.0),
				// 	height: Val::Percent(100.0),
				// 	justify_content: JustifyContent::FlexEnd,
				// 	align_items: AlignItems::Center,
				// 	..default()
				// },
				style: style! { Style
					width: 100%,
					height: 100%,
					justify_content: center,
					align_items: end,
					// margin: 10 px,
					flex_direction: column,
					// flex_grow: 1,
				},
				// background_color: Color::GREEN.into(),
				..default()
			}
			.named("UI Root")
			.not_pickable(),
		)
		.with_children(|parent| {
			ItemPreview::ui(parent);

			parent
				.spawn(
					NodeBundle {
						style: style! {Style
							flex_grow: 1,
							flex_direction: column,
							justify_content: space_evenly,
							align_items: center,

							width: 100%,
							max_width: 20 vw,
						},
						// background_color: Color::DARK_GREEN.into(),
						..default()
					}
					.named("Inventory Root"),
				)
				.with_children(|parent| {
					PlayerInventory::ui(parent, &mma.ass);
				});
		});
}
