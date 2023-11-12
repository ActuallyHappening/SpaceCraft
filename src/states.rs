//! Global state management

use std::net::{IpAddr, Ipv4Addr};

use crate::prelude::*;

nested_structs!(
	/// The states that are truly ir-reconcilable, that fully clear all affected entities and are a major `.run_if()` condition
	#[derive(States)]
	#[strikethrough[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]]
	pub enum GlobalGameStates {
		StartMenu(pub enum StartScreenStates {
				Initial,

				ConfigureHosting,

				ConfigureClient,
				// ConfigureSolo
		}),

		InGame,
	}
);

impl Default for GlobalGameStates {
	fn default() -> Self {
		GlobalGameStates::StartMenu(StartScreenStates::Initial)
	}
}

pub fn in_state_start_menu(state: Res<State<GlobalGameStates>>) -> bool {
	matches!(state.get(), GlobalGameStates::StartMenu(_))
}

pub fn in_state_game(state: Res<State<GlobalGameStates>>) -> bool {
	matches!(state.get(), GlobalGameStates::InGame)
}

#[derive(Resource, Debug)]
pub enum NetcodeConfig {
	Hosting { addr: IpAddr, port: u16 },
	Client { addr: IpAddr, port: u16 },
}

impl NetcodeConfig {
	pub const fn new_hosting_public() -> Self {
		// TODO: Verify this actually hosts, don't we need 0.0.0.0?
		NetcodeConfig::Hosting {
			addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
			port: DEFAULT_PORT,
		}
	}

	pub const fn new_hosting_machine_local() -> Self {
		NetcodeConfig::Hosting {
			addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
			port: DEFAULT_PORT,
		}
	}

	pub const fn new_client_machine_local() -> Self {
		NetcodeConfig::Client {
			addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
			port: DEFAULT_PORT,
		}
	}
}
