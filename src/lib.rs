// #![feature(const_trait_impl)]

pub mod utils;
// pub use utils::init_debug_tools;

mod bevy;
pub use bevy::renet::RenetPlugin;
use std::sync::{Arc, Mutex};

pub use bevy::MainPlugin;
mod core;

use crate::utils::*;

// lazy_static::lazy_static!(
// 	pub static ref ADD_SERVER: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
// );

pub fn add_server(mut commands: Commands, network_channels: Res<NetworkChannels>) {
	// if *ADD_SERVER.lock().unwrap() {
	// 	*ADD_SERVER.lock().unwrap() = false;

		
}
