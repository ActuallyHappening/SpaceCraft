use crate::{players::player::ControllablePlayer, prelude::*};

// pub struct InputProcessingPlugin;

// impl Plugin for InputProcessingPlugin {
// 	fn build(&self, app: &mut App) {
// 		app
// 			.add_systems(PreUpdate, process_action_diffs::<PlayerInput, Key>)
// 			.add_systems(
// 				PostUpdate,
// 				generate_action_diffs::<PlayerInput, Key>.run_if(NetcodeConfig::not_headless()),
// 			)
// 			// .register_type::<Key>()
// 			.add_client_event::<ActionDiff<PlayerInput, Key>>(SendType::Unreliable);
// 	}
// }

#[derive(ActionLike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum PlayerInput {
	Forward,
	Backward,
	Left,
	Right,
}

impl PlayerInput {
	pub const FORCE_FACTOR: f32 = 2.;
	pub const ROTATION_FACTOR: f32 = 2.;
}

// #[derive(SystemParam, Debug)]
// pub struct PlayerInputs<'w, 's> {
// 	query: Query<'w, 's, (Entity, &'static ActionState<PlayerInput>)>,
// }

// impl PlayerInputs<'_, '_> {
// 	// pub fn get_from_id(&self, player_id: ClientId) -> Option<&ActionState<PlayerInput>> {
// 	// 	self
// 	// 		.query
// 	// 		.iter()
// 	// 		.find(|(_, player)| player.get_network_id() == player_id)
// 	// 		.map(|(action_state, _)| action_state)
// 	// }

// 	// pub fn get(&self, e: Entity) -> Option<&ActionState<PlayerInput>> {
// 	// 	self.query.get(e).ok().map(|(action_state, _)| action_state)
// 	// }

// 	pub fn iter(&self) -> impl Iterator<Item = (Entity, &ActionState<PlayerInput>)> {
// 		self.query.iter()
// 	}
// }

impl PlayerInput {
	pub fn new() -> InputManagerBundle<Self> {
		InputManagerBundle {
			action_state: ActionState::default(),
			input_map: InputMap::new([
				(KeyCode::W, PlayerInput::Forward),
				(KeyCode::S, PlayerInput::Backward),
				(KeyCode::A, PlayerInput::Left),
				(KeyCode::D, PlayerInput::Right),
			]),
		}
	}
}
