use std::net::Ipv4Addr;

use crate::utils::*;

/// For UI purposes
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
pub enum ScreenState {
	#[default]
	StartScreen,

	InGame,
}

// #[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
// pub enum GameStates {
// 	#[default]
// 	PlayField,
	
// 	// #[default]
// 	Designing,
// }

// /// What context to execute logic for each frame
// #[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
// pub enum Controlling {
// 	/// Controlling a local player
// 	#[default]
// 	Local,

// 	/// Not controlling any character
// 	Global
// }

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
pub enum ServerConnections {
	/// Hosting to outside world
	// #[default]
	Hosting,

	/// Connecting to a server and displaying server state
	Client,

	/// Not interacting with any servers,
	/// or hosting (yet)
	Local,

	#[default]
	NotPlaying,
}

impl ServerConnections {
	pub fn should_simulate(&self) -> bool {
		matches!(self, ServerConnections::Hosting | ServerConnections::Client)
	}

	
}

#[derive(Debug, PartialEq, Eq, Hash, Resource)]
pub struct SavedHostingInfo {
	pub join_ip: IpAddr,
	pub join_port: u16,

	pub host_ip: IpAddr,
	pub host_port: u16,
}

impl Default for SavedHostingInfo {
	fn default() -> Self {
		const PORT: u16 = 5069;
		SavedHostingInfo {
			join_ip: Ipv4Addr::LOCALHOST.into(),
			join_port: PORT,

			host_ip: Ipv4Addr::LOCALHOST.into(),
			host_port: PORT,
		}
	}
}