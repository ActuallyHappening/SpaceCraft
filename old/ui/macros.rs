#![allow(unused_imports)]
use crate::utils::*;
use bevy::{prelude::Style, ui::UiRect};

// concat_idents!()

macro_rules! to_literal_f32 {
	($val:literal) => {
		stringify!($val.0)
			.parse()
			.unwrap_or(stringify!($val).parse().unwrap())
	};
}

pub(crate) use to_literal_f32;

// macro_rules! access {
// 	($type:expr ;.; $prop:ident = $val:expr) => {
// 		$type.$prop = $val;
// 	}
// }
// pub(crate) use access;

macro_rules! style {
		( Style $($rest:tt)+ ) => {
			style!{
				prev
					Style {
						..default()
					}
				;next; $($rest)*
			}
		};

		// aspect_ratio
		(prev $prev:expr ;next; aspect_ratio: $val:literal, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.aspect_ratio = Some($crate::bevy::ui::macros::to_literal_f32!($val));
					prev
				} ;next; $($rest)*
			}
		};

		// margin
		(prev $prev:expr ;next; margin: $val:literal px, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.margin = UiRect::all(Val::Px($crate::bevy::ui::macros::to_literal_f32!($val)));
					prev
				} ;next; $($rest)*
			}
		};
		// border
		(prev $prev:expr ;next; border: $val:literal px, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.border = UiRect::all(Val::Px($crate::bevy::ui::macros::to_literal_f32!($val)));
					prev
				} ;next; $($rest)*
			}
		};

		// flex direction
		(prev $prev:expr ;next; flex_direction: row, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.flex_direction = FlexDirection::Row;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; flex_direction: column, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.flex_direction = FlexDirection::Column;
					prev
				} ;next; $($rest)*
			}
		};

		// generic % percentages
		(prev $prev:expr ;next; $prop:ident: $val:literal %, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.$prop = Val::Percent($crate::bevy::ui::macros::to_literal_f32!($val));
					prev
				} ;next; $($rest)*
			}
		};
		// generic px pixels
		(prev $prev:expr ;next; $prop:ident: $val:literal px, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.$prop = Val::Px($crate::bevy::ui::macros::to_literal_f32!($val));
					prev
				} ;next; $($rest)*
			}
		};
		// generic vw viewport width
		(prev $prev:expr ;next; $prop:ident: $val:literal vw, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.$prop = Val::Vw($crate::bevy::ui::macros::to_literal_f32!($val));
					prev
				} ;next; $($rest)*
			}
		};
		// // generic vw expr
		// (prev $prev:expr ;next; $prop:ident: $val:expr vw, $($rest:tt)*) => {
		// 	style!{
		// 		prev {
		// 			let mut prev = $prev;
		// 			prev.$prop = Val::Vw($val);
		// 			prev
		// 		} ;next; $($rest)*
		// 	}
		// };
		// generic f32 literals
		(prev $prev:expr ;next; $prop:ident: $val:literal, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.$prop = $crate::bevy::ui::macros::to_literal_f32!($val);
					prev
				} ;next; $($rest)*
			}
		};

		// justify content
		(prev $prev:expr ;next; justify_content: center, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.justify_content = JustifyContent::Center;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; justify_content: end, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.justify_content = JustifyContent::End;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; justify_content: start, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.justify_content = JustifyContent::Start;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; justify_content: space_evenly, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.justify_content = JustifyContent::SpaceEvenly;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; justify_content: space_around, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.justify_content = JustifyContent::SpaceAround;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; justify_content: space_between, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.justify_content = JustifyContent::SpaceBetween;
					prev
				} ;next; $($rest)*
			}
		};

		// align items
		(prev $prev:expr ;next; align_items: center, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.align_items = AlignItems::Center;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; align_items: end, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.align_items = AlignItems::End;
					prev
				} ;next; $($rest)*
			}
		};
		(prev $prev:expr ;next; align_items: start, $($rest:tt)*) => {
			style!{
				prev {
					let mut prev = $prev;
					prev.align_items = AlignItems::Start;
					prev
				} ;next; $($rest)*
			}
		};

		// base case
		(prev $prev:expr ;next; ) => {
			$prev
		};
}

#[test]
fn test_style_macro() {
	assert_eq!(
		style! {Style width: 100%, },
		Style {
			width: Val::Percent(100.),
			..default()
		}
	);
	assert_eq!(
		style! {Style height: 69%, },
		Style {
			height: Val::Percent(69.),
			..default()
		}
	);
	assert_eq!(
		style! {Style
			justify_content: center,
			justify_content: end,
			justify_content: start,
		},
		Style {
			justify_content: JustifyContent::Start,
			..default()
		}
	);

	assert_eq!(
		style! {Style
			align_items: center,
			align_items: end,
			align_items: start,
		},
		Style {
			align_items: AlignItems::Start,
			..default()
		}
	);

	assert_eq!(
		style! {Style margin: 69 px, border: 420 px, },
		Style {
			margin: UiRect::all(Val::Px(69.)),
			border: UiRect::all(Val::Px(420.)),
			..default()
		}
	);

	assert_eq!(
		style! {Style max_width: 69 px,},
		Style {
			max_width: Val::Px(69.),
			..default()
		}
	);

	assert_eq!(
		style! {Style
			flex_direction: row,
			flex_direction: column,
		},
		Style {
			flex_direction: FlexDirection::Column,
			..default()
		}
	);
}

pub(crate) use style;
