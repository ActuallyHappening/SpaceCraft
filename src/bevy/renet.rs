use crate::utils::*;
use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::{shape::Icosphere, *}, ecs::system::SystemParam};
use bevy_rapier3d::prelude::*;
// use bevy_renet::renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use bevy_replicon::ReplicationPlugins;
use renet::transport::NetcodeClientTransport;
use serde::{Deserialize, Serialize};

mod client;
pub mod server;

/// Steps physics, done by server
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuthoritativeUpdate;

/// Handles effects, renders graphics, done by client
/// Semantic only, might change if 'headless' servers are needed
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientUpdate;

pub struct RenetPlugin;
impl Plugin for RenetPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(ReplicationPlugins)
			.add_plugins((client::ClientPlugin, server::ServerPlugin))
			.configure_set(Update, AuthoritativeUpdate.run_if(has_authority()))
			.configure_set(Update, ClientUpdate.run_if(in_state(ScreenState::InGame)));
	}
}

// #[cfg(feature = "transport")]
// pub const PRIVATE_KEY: &[u8; bevy_renet::renet::transport::NETCODE_KEY_BYTES] =
// b"un example sehr tres secret key."; // 32-bytes
// #[cfg(feature = "transport")]
pub const PROTOCOL_ID: u64 = 0;

#[derive(SystemParam)]
pub struct ClientID<'w> {
	res: Option<Res<'w, NetcodeClientTransport>>,
}

impl ClientID<'_> {
	pub fn id(&self) -> u64 {
		self
			.res
			.as_ref()
			.map(|client| client.client_id())
			.unwrap_or(SERVER_ID)
	}
}

// #[derive(Debug, Component)]
// pub struct Player {
// 	pub id: ClientId,
// }

// #[derive(Debug, Default, Clone, Serialize, Deserialize, Component, Resource)]
// pub struct PlayerInput {
// 	forward: bool,
// 	// pub up: bool,
// 	// pub down: bool,
// 	// pub left: bool,
// 	// pub right: bool,
// }

// fn cli_system(
//         mut commands: Commands,
//         cli: Res<Cli>,
//         network_channels: Res<NetworkChannels>,
//     ) -> Result<(), Box<dyn Error>> {
//         match *cli {
//             Cli::SinglePlayer => {
//                 commands.spawn(PlayerBundle::new(SERVER_ID, Vec2::ZERO, Color::GREEN));
//             }
//             Cli::Server { port } => {
//                 let server_channels_config = network_channels.server_channels();
//                 let client_channels_config = network_channels.client_channels();

//                 let server = RenetServer::new(ConnectionConfig {
//                     server_channels_config,
//                     client_channels_config,
//                     ..Default::default()
//                 });

//                 let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
//                 let public_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
//                 let socket = UdpSocket::bind(public_addr)?;
//                 let server_config = ServerConfig {
//                     max_clients: 10,
//                     protocol_id: PROTOCOL_ID,
//                     public_addr,
//                     authentication: ServerAuthentication::Unsecure,
//                 };
//                 let transport = NetcodeServerTransport::new(current_time, server_config, socket)?;

//                 commands.insert_resource(server);
//                 commands.insert_resource(transport);

//                 commands.spawn(TextBundle::from_section(
//                     "Server",
//                     TextStyle {
//                         font_size: 30.0,
//                         color: Color::WHITE,
//                         ..default()
//                     },
//                 ));
//                 commands.spawn(PlayerBundle::new(SERVER_ID, Vec2::ZERO, Color::GREEN));
//             }
//             Cli::Client { port, ip } => {
//                 let server_channels_config = network_channels.server_channels();
//                 let client_channels_config = network_channels.client_channels();

//                 let client = RenetClient::new(ConnectionConfig {
//                     server_channels_config,
//                     client_channels_config,
//                     ..Default::default()
//                 });

//                 let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
//                 let client_id = current_time.as_millis() as u64;
//                 let server_addr = SocketAddr::new(ip, port);
//                 let socket = UdpSocket::bind((ip, 0))?;
//                 let authentication = ClientAuthentication::Unsecure {
//                     client_id,
//                     protocol_id: PROTOCOL_ID,
//                     server_addr,
//                     user_data: None,
//                 };
//                 let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

//                 commands.insert_resource(client);
//                 commands.insert_resource(transport);

//                 commands.spawn(TextBundle::from_section(
//                     format!("Client: {client_id:?}"),
//                     TextStyle {
//                         font_size: 30.0,
//                         color: Color::WHITE,
//                         ..default()
//                     },
//                 ));
//             }
//         }

//         Ok(())
//     }
