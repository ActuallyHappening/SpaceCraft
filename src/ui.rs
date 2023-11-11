use crate::prelude::*;

/// Plugin Group
pub struct UiPlugins;

impl PluginGroup for UiPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(self::start_screen::StartScreen)
			.add(self::ui_cameras::UiCamerasPlugin)
			.build()
	}
}

mod ui_cameras;

mod manual_ui {
	use crate::prelude::*;

	/// Defining the rectangular dimensions / bounding box
	#[derive(Debug, Component)]
	pub struct BBox {
		pub half_width: f32,
		pub half_height: f32,
	}

	/// Minimum information needed to construct a bevy bundle
	pub struct ManualNode {
		/// Used to create mesh
		pub bbox: BBox,
		pub position: Vec2,
	}

	pub struct ManualColumn {
		pub const_x: f32,
		pub const_width: f32,

		pub item_height: f32,
		pub margin: f32,

		pub current_y: f32,
	}

	impl Iterator for ManualColumn {
		type Item = ManualNode;

		fn next(&mut self) -> Option<Self::Item> {
			let node = ManualNode {
				bbox: BBox {
					half_width: self.const_width / 2.,
					half_height: self.item_height / 2.,
				},
				position: Vec2::new(self.const_x, self.current_y),
			};

			self.current_y += self.item_height + self.margin;

			Some(node)
		}
	}

	impl ManualColumn {
		pub fn next(&mut self) -> ManualNode {
			Iterator::next(self).unwrap()
		}
	}
}

mod path_tracing {
	use super::manual_ui::BBox;
	use crate::prelude::*;

	#[derive(Component, Debug)]
	pub struct BevyPath {
		path: Vec<Segment>,
		total_length: f32,
	}

	#[derive(Debug)]
	enum Segment {
		Start(Vec2),
		LineTo(Vec2),
	}

	impl BevyPath {
		pub fn rectangle_from_bbox(bbox: BBox) -> Self {
			let half_width = bbox.half_width;
			let half_height = bbox.half_height;

			let mut path = Vec::new();

			path.push(Segment::Start(Vec2::new(-half_width, -half_height)));
			path.push(Segment::LineTo(Vec2::new(half_width, -half_height)));
			path.push(Segment::LineTo(Vec2::new(half_width, half_height)));
			path.push(Segment::LineTo(Vec2::new(-half_width, half_height)));
			path.push(Segment::LineTo(Vec2::new(-half_width, -half_height)));

			Self {
				path,
				total_length: 4. * half_width + 4. * half_height,
			}
		}

		pub fn get_pos_at_time(&self, time: f32) -> Vec2 {
			assert!((0. ..=1.).contains(&time));
			let current_len = self.total_length * time;
			let mut current_pos = Vec2::ZERO;
			let mut current_len_so_far = 0.;
			for seg in &self.path {
				match seg {
					Segment::Start(pos) => {
						current_pos = *pos;
					}
					Segment::LineTo(pos) => {
						let seg_len = (*pos - current_pos).length();
						if current_len_so_far + seg_len >= current_len {
							let seg_time = (current_len - current_len_so_far) / seg_len;
							return current_pos + (*pos - current_pos) * seg_time;
						} else {
							current_len_so_far += seg_len;
							current_pos = *pos;
						}
					}
				}
			}

			// edge case
			error!("Wrong length computed!");
			Vec2::ZERO
		}
	}

	#[test]
	fn start_pos() {
		let path = BevyPath::rectangle_from_bbox(BBox {
			half_width: 5.,
			half_height: 7.,
		});
		assert_eq!(path.get_pos_at_time(0.), Vec2::new(-5., -7.));
		assert_eq!(path.get_pos_at_time(1.), Vec2::new(-5., -7.));
	}
}

mod start_screen;
