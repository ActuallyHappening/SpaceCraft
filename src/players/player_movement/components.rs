use crate::prelude::*;

/// Stores all of the data concerning thruster movements.
/// Placed on players.
///
/// Is not replicated, is derived data
#[derive(Component, Debug, Reflect, Serialize, Deserialize, Default)]
pub(super) struct ThrusterStrengths {
	blocks: HashMap<BlockId, f32>,
}

impl ThrusterStrengths {
	pub(super) fn get_blocks_strength(&self) -> HashMap<&BlockId, &f32> {
		self
			.blocks
			.iter()
			.map(|(id, strength)| (id, strength))
			.collect()
	}

	pub(super) fn new(blocks: impl IntoIterator<Item = (BlockId, f32)>) -> Self {
		Self {
			blocks: blocks.into_iter().collect(),
		}
	}
}

/// Can maybe be cached after first computation,
/// depending on whether player rebuild their ships.
///
/// Is not replicated, is derived data.
#[derive(Debug, Component, Reflect, Default)]
pub(super) struct ThrusterAxis {
	blocks: HashMap<BlockId, ForceAxis>,
}

use force_axis::ForceAxis;

use super::{Velocity6Dimensions, Velocity6DimensionsMut};
mod force_axis;

impl ThrusterAxis {
	pub(super) fn new<'w>(
		center_of_mass: &'w CenterOfMass,
		blocks: impl IntoIterator<Item = (BlockId, &'w Transform)>,
	) -> Self {
		Self {
			blocks: blocks
				.into_iter()
				.map(|(id, t)| (id, ForceAxis::new(t, center_of_mass)))
				.collect(),
		}
	}

	pub(super) fn get_blocks(&self) -> impl Iterator<Item = (BlockId, &ForceAxis)> {
		self.blocks.iter().map(|(id, force_axis)| (*id, force_axis))
	}
}

#[derive(Debug, Reflect, Component, Default, Serialize, Deserialize, Clone, Copy)]
pub(super) struct IntendedVelocity {
	forward: f32,
	right: f32,
	up: f32,
	turn_right: f32,
	tilt_up: f32,
	roll_right: f32,
}

impl Velocity6Dimensions for IntendedVelocity {
	fn velocity_forward(&self) -> f32 {
		self.forward
	}

	fn velocity_rightward(&self) -> f32 {
		self.right
	}

	fn velocity_upward(&self) -> f32 {
		self.up
	}

	fn angular_turn_right(&self) -> f32 {
		self.turn_right
	}

	fn angular_tilt_up(&self) -> f32 {
		self.tilt_up
	}

	fn angular_roll_right(&self) -> f32 {
		self.roll_right
	}
}
impl Velocity6DimensionsMut for IntendedVelocity {
	fn forward_mut(&mut self) -> &mut f32 {
		&mut self.forward
	}

	fn right_mut(&mut self) -> &mut f32 {
		&mut self.right
	}

	fn up_mut(&mut self) -> &mut f32 {
		&mut self.up
	}

	fn turn_right_mut(&mut self) -> &mut f32 {
		&mut self.turn_right
	}

	fn tilt_up_mut(&mut self) -> &mut f32 {
		&mut self.tilt_up
	}

	fn roll_right_mut(&mut self) -> &mut f32 {
		&mut self.roll_right
	}
}

#[derive(Debug, Reflect, Component, Default, Clone, Copy)]
pub(super) struct ActualVelocity {
	forward: f32,
	right: f32,
	up: f32,
	turn_right: f32,
	tilt_up: f32,
	roll_right: f32,
}

impl Velocity6Dimensions for ActualVelocity {
	fn velocity_forward(&self) -> f32 {
		self.forward
	}

	fn velocity_rightward(&self) -> f32 {
		self.right
	}

	fn velocity_upward(&self) -> f32 {
		self.up
	}

	fn angular_turn_right(&self) -> f32 {
		self.turn_right
	}

	fn angular_tilt_up(&self) -> f32 {
		self.tilt_up
	}

	fn angular_roll_right(&self) -> f32 {
		self.roll_right
	}
}
impl Velocity6DimensionsMut for ActualVelocity {
	fn forward_mut(&mut self) -> &mut f32 {
		&mut self.forward
	}

	fn right_mut(&mut self) -> &mut f32 {
		&mut self.right
	}

	fn up_mut(&mut self) -> &mut f32 {
		&mut self.up
	}

	fn turn_right_mut(&mut self) -> &mut f32 {
		&mut self.turn_right
	}

	fn tilt_up_mut(&mut self) -> &mut f32 {
		&mut self.tilt_up
	}

	fn roll_right_mut(&mut self) -> &mut f32 {
		&mut self.roll_right
	}
}
