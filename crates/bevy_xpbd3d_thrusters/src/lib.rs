pub mod prelude {
	pub(crate) use bevy::{prelude::*, utils::HashMap};
}

mod old_types {
	use bevy::prelude::*;

	/// Forces are between [-1, 1],
	/// Torques are 'normalized' to [-1, 1] // TODO
	///
	/// The Greek philosopher, Archimedes, said,
	/// “Give me a lever long enough and a fulcrum on which to place it, and I shall move the world.”
	#[derive(Debug, Reflect, Default, Clone, Copy)]
	pub struct ForceAxis {
		forward: f32,
		right: f32,
		upwards: f32,
		turn_right: f32,
		pitch_up: f32,
		roll_right: f32,
	}
}

mod strategies {
	use crate::prelude::*;

	#[derive(Debug, Hash, Clone, PartialEq, PartialOrd, Eq, Ord, Reflect)]
	pub struct ID(u64);

	#[reflect_trait]
	pub trait Strategy {
		fn calculate(&self, blocks: HashMap<&ID>)
	}
}
