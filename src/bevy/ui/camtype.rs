use crate::utils::*;

#[derive(Component, Debug, Default)]
pub struct UiCamera<S: CamType>(PhantomData<S>);

#[allow(private_bounds)]
pub trait CamType: Component + std::fmt::Debug + Send + Sync + Default + Sealed + Into<RenderLayers> {
	/// Implement this without the offset
	fn get_cam_transform(half_width: f32, half_height: f32) -> UVec2;

	fn get_camera_order() -> isize;
}
pub trait Sealed {}

macro_rules! impl_cam_sticky {
		($(pub struct $name:ident; half_width = $w:literal%, half_height = $h:literal %, order = $order:literal, render_layer = $layer:literal,)*) => {
		// ($(pub struct $name:ident;)*) => {
			$(
				#[derive(Component, Debug, Clone, Copy, Default, Hash, PartialEq, Eq, SystemSet)]
				pub struct $name;
				impl Sealed for $name {}
				impl CamType for $name {
					fn get_cam_transform(half_width: f32, half_height: f32) -> UVec2 {
						UVec2::new(
							(half_width as f32 * $w as f32 / 100.) as u32,
							(half_height as f32 * $h as f32 / 100.) as u32,
						)
					}

					fn get_camera_order() -> isize {
						$order
					}
				}
				impl From<$name> for RenderLayers {
					fn from(_: $name) -> Self {
						RenderLayers::none().with($layer)
					}
				}
			)*
		};
}

impl_cam_sticky!(
	pub struct BottomLeft; half_width = 100%, half_height = 100%, order = 1, render_layer = 31,
	pub struct TopLeft; half_width = 100%, half_height = -100%, order = 2, render_layer = 30,
	pub struct BottomRight; half_width = -100%, half_height = 100%, order = 3, render_layer = 29,
	pub struct TopRight; half_width = -100%, half_height = -100%, order = 4, render_layer = 28,
);

impl<T: CamType> UiCamera<T> {
	pub fn get_camera_bundle() -> impl Bundle {
		Camera2dBundle {
			camera: Camera {
				order: T::get_camera_order(),
				..default()
			},
			camera_2d: Camera2d {
				clear_color: ClearColorConfig::None,
			},
			..default()
		}
		.insert(Self::default())
		.render_layer(T::default())
		.not_pickable()
	}
}
