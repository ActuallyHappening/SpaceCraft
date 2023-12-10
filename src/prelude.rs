//! Glob exports of dependencies and utils in a convenient path
//!

// std
pub use bevy::utils::{HashMap, HashSet};
pub use std::borrow::Cow;
pub use std::f32::consts::{PI, TAU};
pub use std::net::*;
pub use std::num::*;
pub use std::ops::{Add as _, Div as _, Mul as _};
pub use std::time::*; // is this better than std library?

// bevy
pub use bevy::app::*;
pub use bevy::core_pipeline::clear_color::ClearColorConfig;
pub use bevy::core_pipeline::tonemapping::Tonemapping;
pub use bevy::ecs::system::*;
pub use bevy::prelude::AlphaMode;
pub use bevy::prelude::*;
pub use bevy::render::view::RenderLayers;

// bevy_* deps
pub use bevy_hanabi::prelude::*;
pub use bevy_inspector_egui::prelude::*;
pub use bevy_mod_picking::prelude::*;
pub use bevy_replicon::{prelude::*, renet::ClientId};
pub use bevy_timewarp::prelude::*;
pub use bevy_xpbd_3d::prelude::*;
// conflicts names like [Transform]
// pub use bevy_dolly::prelude::*;
pub use leafwing_input_manager::prelude::Actionlike as ActionLike;
pub use leafwing_input_manager::prelude::*;

// helper deps
pub use structstruck::strike as nested_structs;
pub use strum::EnumIter;
pub use strum::IntoEnumIterator as _;
// pub use uuid::Uuid;
pub use clap::Parser;
pub use rand::random;
pub use rand::rngs::ThreadRng;
pub use rand::Rng;
pub use serde::{Deserialize, Serialize};
pub use strum::IntoStaticStr;

// internal re-exports

pub use crate::blocks::*;
pub(crate) use crate::global;
pub use crate::global::assets::*;
pub use crate::global::blueprints::*;
pub use crate::global::*;

pub use crate::netcode::*;

pub use crate::states::*;

pub use crate::utils::*;
