//! Glob exports of dependencies and utils in a convenient path
//! 

// std
pub use std::borrow::Cow;
pub use std::f32::consts::{TAU, PI};
pub use std::net::*;
pub use std::num::*;
pub use std::ops::{Add as _, Div as _, Mul as _};
pub use std::time::*;
pub use bevy::utils::HashMap; // is this better than std library?

// bevy
pub use bevy::app::*;
pub use bevy::core_pipeline::clear_color::ClearColorConfig;
pub use bevy::core_pipeline::tonemapping::Tonemapping;
pub use bevy::ecs::system::*;
pub use bevy::prelude::*;
pub use bevy::render::view::RenderLayers;

// bevy_* deps
pub use bevy_hanabi::prelude::*;
pub use bevy_mod_picking::prelude::*;
pub use bevy_replicon::{prelude::*, renet::ClientId};
pub use bevy_xpbd_3d::prelude::*;
pub use bevy_timewarp::prelude::*;

// helper deps
pub use structstruck::strike as nested_structs;
pub use strum::EnumIter;
pub use strum::IntoEnumIterator as _;
// pub use uuid::Uuid;
pub use rand::random;
pub use serde::{Serialize, Deserialize};
pub use clap::Parser;
pub use strum::IntoStaticStr;

// internal re-exports
pub use crate::global::assets::*;
pub use crate::global::*;
pub use crate::states::*;
pub use crate::utils::*;
pub use crate::blocks::*;
