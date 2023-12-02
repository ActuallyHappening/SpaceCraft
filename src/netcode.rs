use crate::{players::PlayerBlueprint, prelude::*};

pub use bevy_replicon::{
	prelude::*,
	renet::{
		transport::{
			ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
			ServerConfig,
		},
		ConnectionConfig, ServerEvent,
	},
};

pub struct NetcodePlugin;

/// Contains only systems that are relevant to controlling a player.
/// 
/// Configured for [Update] and [FixedUpdate] schedules.
#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub struct Client;

/// Contains only systems that are relevant to the server.
/// 
/// Configured ONLY for [FixedUpdate]
#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub struct Server;

impl Plugin for NetcodePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnEnter(GlobalGameStates::InGame), Self::add_netcode)
			.add_systems(OnExit(GlobalGameStates::InGame), Self::disconnect_netcode)
			.add_systems(Update, Self::server_event_system.in_set(Server))
			.add_systems(FixedUpdate, Self::frame_inc_and_replicon_tick_sync)
			.configure_sets(
				FixedUpdate,
				Client.run_if(NetcodeConfig::not_headless()),
			)
			.configure_sets(
				Update,
				Client.run_if(NetcodeConfig::not_headless()),
			)
			.configure_sets(FixedUpdate, Server.run_if(has_authority()));
	}
}

impl NetcodePlugin {
	fn frame_inc_and_replicon_tick_sync(
		mut game_clock: ResMut<GameClock>, // your own tick counter
		mut replicon_tick: ResMut<RepliconTick>,
	) {
		// advance your tick and replicon's tick in lockstep
		game_clock.advance(1);
		let delta = game_clock.frame().saturating_sub(replicon_tick.get());
		replicon_tick.increment_by(delta);
	}
}

/// Useful for finding the local player's renet id.
#[derive(SystemParam)]
pub struct ClientID<'w> {
	res: Option<Res<'w, NetcodeClientTransport>>,
	is_headless: Res<'w, NetcodeConfig>,
}

impl ClientID<'_> {
	/// Returns the client_id of the local player, or
	/// [None] if [NetcodeConfig] is configured to be
	/// [NetcodeConfig::is_headless]
	pub fn client_id(&self) -> Option<ClientId> {
		let local_id = self
			.res
			.as_ref()
			.map(|client| client.client_id())
			.unwrap_or(SERVER_ID);
		if self.is_headless.get_headless() {
			None
		} else {
			Some(local_id)
		}
	}

	/// See [Self::client_id], [Option::unwrap]s for you.
	/// Only use in systems that are already gated by
	/// `.run_if`
	pub fn assert_client_id(&self) -> ClientId {
		self
			.client_id()
			.expect("ClientID is None, place the system that uses this parameter in a .run_if")
	}
}

/// Holds information about what ip and port to connect to, or host on.
#[derive(Resource, Debug, clap::Parser)]
pub enum NetcodeConfig {
	Server {
		#[arg(long, default_value_t = Ipv4Addr::LOCALHOST.into())]
		ip: IpAddr,

		#[arg(short, long, default_value_t = DEFAULT_PORT)]
		port: u16,

		/// Whether or not to run the server in headless mode.
		#[arg(long, default_value_t = false)]
		headless: bool,
	},
	Client {
		#[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
		ip: IpAddr,

		#[arg(short, long, default_value_t = DEFAULT_PORT)]
		port: u16,
	},
}

impl NetcodeConfig {
	pub const fn new_hosting_public(headless: bool) -> Self {
		NetcodeConfig::Server {
			ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
			port: DEFAULT_PORT,
			headless,
		}
	}

	pub const fn new_hosting_machine_local(headless: bool) -> Self {
		NetcodeConfig::Server {
			ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
			port: DEFAULT_PORT,
			headless,
		}
	}

	pub const fn new_client_machine_local() -> Self {
		NetcodeConfig::Client {
			ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
			port: DEFAULT_PORT,
		}
	}

	pub const fn get_headless(&self) -> bool {
		match self {
			NetcodeConfig::Server { headless, .. } => *headless,
			NetcodeConfig::Client { .. } => false,
		}
	}

	/// Used in a `.run_if` to signify a system that should only run if
	/// a client/player is being controlled by the current instance
	pub fn not_headless() -> impl Fn(Res<NetcodeConfig>) -> bool {
		|config| !config.into_inner().get_headless()
	}
}

impl NetcodePlugin {
	/// sets up the server / client depending on [NetcodeConfig]
	fn add_netcode(
		mut commands: Commands,
		network_channels: Res<NetworkChannels>,
		config: Res<NetcodeConfig>,
		_manual_server_feedback: EventWriter<ServerEvent>,
	) {
		match config.into_inner() {
			NetcodeConfig::Server { ip, port, headless } => {
				info!("Setting up as server, hosting on {}:{}", ip, port);
				let server_channels_config = network_channels.get_server_configs();
				let client_channels_config = network_channels.get_client_configs();

				let server = RenetServer::new(ConnectionConfig {
					server_channels_config,
					client_channels_config,
					..Default::default()
				});

				let current_time = SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap();
				let public_addr = SocketAddr::new(*ip, *port);

				let socket = UdpSocket::bind(public_addr).expect("Couldn't bind to UdpSocket");

				let server_config = ServerConfig {
					current_time,
					max_clients: 10,
					protocol_id: PROTOCOL_ID,
					public_addresses: vec![public_addr],
					authentication: ServerAuthentication::Unsecure,
				};
				let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

				commands.insert_resource(server);
				commands.insert_resource(transport);

				if !headless {
					commands.spawn(PlayerBlueprint::default_at(SERVER_ID, Transform::default()));
				}
			}
			NetcodeConfig::Client { ip, port } => {
				info!(
					"Setting up as client, connecting to {:?} on port {}",
					ip, port
				);
				let server_channels_config = network_channels.get_server_configs();
				let client_channels_config = network_channels.get_client_configs();

				let client = RenetClient::new(ConnectionConfig {
					server_channels_config,
					client_channels_config,
					..Default::default()
				});

				let current_time = SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap();
				let client_id = ClientId::from_raw(current_time.as_millis() as u64);
				let server_addr = SocketAddr::new(*ip, *port);
				let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
					.expect("Couldn't bind to (unspecified) socket");
				let authentication = ClientAuthentication::Unsecure {
					client_id,
					protocol_id: PROTOCOL_ID,
					server_addr,
					user_data: None,
				};
				let transport = NetcodeClientTransport::new(current_time, authentication, socket)
					.expect("Couldn't join to server");

				commands.insert_resource(client);
				commands.insert_resource(transport);
			}
		}
	}

	fn disconnect_netcode(
		config: Res<NetcodeConfig>,
		mut client: ResMut<RenetClient>,
		mut server: ResMut<RenetServer>,
		mut commands: Commands,
	) {
		match config.into_inner() {
			NetcodeConfig::Server { .. } => {
				info!("Disconnecting as server");
				server.disconnect_all();
				commands.remove_resource::<RenetServer>();
				commands.remove_resource::<NetcodeClientTransport>();
			}
			NetcodeConfig::Client { .. } => {
				info!("Disconnecting client");
				client.disconnect();
				commands.remove_resource::<RenetClient>();
				commands.remove_resource::<NetcodeClientTransport>();
			}
		}
	}

	/// Logs server events and spawns a new player whenever a client connects.
	fn server_event_system(mut server_event: EventReader<ServerEvent>, mut commands: Commands) {
		for event in server_event.read() {
			match event {
				ServerEvent::ClientConnected { client_id } => {
					info!("New player with id {client_id} connected");

					commands.spawn(PlayerBlueprint::default_at(
						*client_id,
						Transform::default(),
					));
				}
				ServerEvent::ClientDisconnected { client_id, reason } => {
					info!("Client {client_id} disconnected because: {reason}");
				}
			}
		}
	}
}
