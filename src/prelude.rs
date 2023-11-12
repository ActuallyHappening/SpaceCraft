//! Glob exports of dependencies and utils in a convenient path

// std
pub use std::ops::{Add as _, Div as _, Mul as _};
pub use std::f32::consts::TAU;
pub use std::borrow::Cow;
pub use std::num::*;

// bevy
pub use bevy::prelude::*;
pub use bevy::app::*;
pub use bevy::render::view::RenderLayers;
pub use bevy::ecs::system::*;
pub use bevy::core_pipeline::clear_color::ClearColorConfig;
pub use bevy::core_pipeline::tonemapping::Tonemapping;

// bevy_* deps
pub use bevy_hanabi::prelude::*;
pub use bevy_mod_picking::prelude::*;

// helper deps
pub use strum::EnumIter;
pub use strum::IntoEnumIterator as _;
pub use structstruck::strike as nested_structs;

pub use crate::utils::*;
pub use crate::global::*;
pub use crate::global::assets::*;
pub use crate::states::*;