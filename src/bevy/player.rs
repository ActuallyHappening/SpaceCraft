use crate::core::PlayerInventory;

use self::weapons::{handle_firing, should_fire_this_frame, toggle_fire, update_bullets};

use super::{
	camera::handle_camera_movement,
	netcode::{AuthoritativeUpdate, ClientUpdate},
	ClientID,
};
use crate::utils::*;
use bevy::ecs::system::SystemParam;
use lazy_static::lazy_static;

mod thrust;
use thrust::*;
pub use thrust::{
	calculate_relative_velocity_magnitudes, get_base_normal_vectors, types, PlayerInputs,
	RelativeStrength, RelativeVelocityMagnitudes, Thrust, ThrustReactions, ThrustReactionsStage,
};

mod weapons;
pub use weapons::WeaponFlags;

pub struct PlayerPlugin;

/// After player thrusts and movement have been handled
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct PlayerMove;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<PlayerInventory>()
			.register_type::<ControllablePlayer>()
			.replicate::<ControllablePlayer>()
			.add_client_event::<PlayerInputs>(EventType::Ordered)
			// .add_systems(Update, (update_bullets,).in_set(AuthoritativeUpdate))
			.add_systems(
				Update,
				(
					(handle_camera_movement, gather_input_flags.pipe(send_event)).in_set(ClientUpdate),
					// should_fire_this_frame.pipe(toggle_fire).pipe(handle_firing),
					authoritative_player_movement.in_set(PlayerMove),
					trigger_player_thruster_particles.after(PlayerMove),

				),
			);
	}
}

#[derive(Debug, Default, Deserialize, Event, Serialize)]
struct DummyEvent;

/// Denotes the main, controllable player
#[derive(Component, Default, Deserialize, Serialize, Reflect)]
pub struct ControllablePlayer {
	pub network_id: u64,

	/// Current relative strength, used for UI
	#[reflect(ignore)]
	relative_strength: Thrust<RelativeStrength>,

	/// Used for UI
	#[reflect(ignore)]
	relative_velocity_magnitudes: Thrust<RelativeVelocityMagnitudes>,

	/// Current inputs including braking info, used for UI
	#[reflect(ignore)]
	thrust_responses: Thrust<ThrustReactionsStage>,

	/// Optional artificial friction flags, starts all enabled
	#[reflect(ignore)]
	artificial_friction_flags: Thrust<ArtificialFrictionFlags>,
}

impl ControllablePlayer {
	pub fn new(network_id: u64) -> Self {
		ControllablePlayer {
			network_id,
			..Default::default()
		}
	}

	pub fn get_relative_strength(&self) -> &Thrust<RelativeStrength> {
		&self.relative_strength
	}

	pub fn get_relative_velocity_magnitudes(&self) -> &Thrust<RelativeVelocityMagnitudes> {
		&self.relative_velocity_magnitudes
	}

	pub fn get_thrust_responses(&self) -> &Thrust<ThrustReactionsStage> {
		&self.thrust_responses
	}

	pub fn get_artificial_friction_flags(&self) -> &Thrust<ArtificialFrictionFlags> {
		&self.artificial_friction_flags
	}

	/// TODO: remove this, and get UI to send update to server
	pub fn get_mut_artificial_friction_flags(&mut self) -> &mut Thrust<ArtificialFrictionFlags> {
		&mut self.artificial_friction_flags
	}
}

#[derive(SystemParam)]
pub struct ThisControllablePlayer<'w, 's> {
	id: ClientID<'w>,
	players: Query<'w, 's, &'static ControllablePlayer>,
}

impl<'world, 'state> ThisControllablePlayer<'world, 'state> {
	pub fn get(&self) -> Option<&ControllablePlayer> {
		self.players.iter().find(|p| p.network_id == self.id.id())
	}
}

lazy_static! {
	pub static ref PLAYER_STRUCTURE: Structure = Structure::new([
		(PixelVariant::PlayerSteel, (0, 0, 0)), // center
		(PixelVariant::PlayerSteel, (0, 0, -1)), // front 1
		(PixelVariant::PlayerSteel, (0, 0, -2)), // front 2
		(PixelVariant::PlayerLargeEngineDecoration, (0, 0, 1)), // back 1
		(PixelVariant::PlayerSteel, (-1, 0, 0)), // left 1
		(PixelVariant::PlayerSteel, (-2, 0, 0)), // left 2
		(PixelVariant::PlayerSteel, (-2, 0, -1)), // left 2, front 1
		(PixelVariant::PlayerSteel, (-1, 0, 1)), // surrounding engine left
		(PixelVariant::PlayerSteel, (0, 1, 1)), // surrounding engine above
	]).with([
		(Thruster::new(Direction::Up, ThrusterFlags::builder().up_down(false).tilt_up(true).roll_right(true).build().unwrap()), (-1, 1, 1)),
		(Thruster::new(Direction::Left, ThrusterFlags::builder().right_left(true).turn_right(false).roll_right(false).build().unwrap()), (-1, 1, 1)),
		(Thruster::new(Direction::Backward, ThrusterFlags::builder().forward_back(true).build().unwrap()), (-1, 0, 2)),
		(Thruster::new(Direction::Forward, ThrusterFlags::builder().forward_back(false).build().unwrap()), (-1, 0, -1)),
	]).with([
		(Weapon::new(Direction::Forward), (-2, 0, -2)), // left 2, front 2
	])
	.reflect_horizontally()
	.reflect_vertically();
}

#[derive(Bundle)]
pub struct AuthorityPlayerBundle {
	controllable_player: ControllablePlayer,
	transform: Transform,
	computed_visibility: ComputedVisibility,
	visibility: Visibility,
	name: Name,
	physics: PhysicsBundle,
	to_spawn: SpawnChildStructure,
	replicate: Replication,
}

impl AuthorityPlayerBundle {
	pub fn new(
		controllable_player: ControllablePlayer,
		structure: Structure,
		transform: Transform,
	) -> Self {
		AuthorityPlayerBundle {
			name: Name::new(format!("Player {}", controllable_player.network_id)),
			controllable_player,
			transform,
			computed_visibility: ComputedVisibility::default(),
			visibility: Visibility::Inherited,
			physics: PhysicsBundle::new(structure.compute_collider()),
			to_spawn: SpawnChildStructure::new(structure),
			replicate: Replication,
		}
	}
}

#[derive(Bundle)]
pub struct PhysicsBundle {
	collider: Collider,
	rigid_body: RigidBody,
	velocity: Velocity,
	damping: Damping,
	external_force: ExternalForce,
	sleeping: Sleeping,
}

impl PhysicsBundle {
	fn new(collider: Collider) -> Self {
		PhysicsBundle {
			collider,
			rigid_body: RigidBody::Dynamic,
			velocity: Velocity {
				linvel: Vec3::ZERO,
				angvel: Vec3::ZERO,
			},
			damping: Damping {
				linear_damping: 0.,
				angular_damping: 0.,
			},
			external_force: ExternalForce {
				force: Vec3::ZERO,
				torque: Vec3::ZERO,
			},
			sleeping: Sleeping::disabled(),
		}
	}
}

/// Spawns the initial player
pub fn authoritative_spawn_initial_player(mut commands: Commands) {
	commands.spawn(AuthorityPlayerBundle::new(
		ControllablePlayer::new(SERVER_ID),
		PLAYER_STRUCTURE.clone(),
		Transform::from_translation(Vec3::new(0., 0., 0.)),
	));
}
