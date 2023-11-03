use super::*;
use serde::{Serialize, Deserialize};

macro_rules! thrust_stage {
	($(#[$($attrss:tt)*])* $(pub)? struct $name:ident; type = $type:ty) => {
		#[doc = concat!("Dimension type = ", stringify!($type), "\n")]
		$(#[$($attrss)*])*

		#[derive(Debug, Clone, Serialize, Deserialize)]
		pub struct $name;
		impl ThrustStage for $name {
			type DimensionType = $type;
		}
	};
}

thrust_stage!(
	/// type = [Option] < bool >
	///
	/// FLAGGED
	///
	/// Information about each dimension of movement, used for enacting
	/// player movement after processing of what should actually been done.
	#[derive(Default)]
	pub struct GenericInputFlags; type = Option<bool>
);

thrust_stage!(
	/// type = [bool]
	///
	/// FLAGGED - chosen by player
	///
	/// What types of movements should be "friction-ed"
	pub struct ArtificialFrictionFlags; type = bool
);

impl Default for Thrust<ArtificialFrictionFlags> {
	fn default() -> Self {
		Thrust::<ArtificialFrictionFlags> {
			forward: true,
			up: true,
			right: true,

			turn_right: true,
			tilt_up: true,
			roll_right: true,

			_stage: PhantomData,
		}
	}
}

thrust_stage!(
	/// type = [ThrustReactions]
	/// 
	/// Intermediate stage for processing in `thrust_reactions`.rs
	#[derive(Default)]
	pub struct ThrustReactionsStage; type = ThrustReactions
);

thrust_stage!(
	/// type = [Vec3]
	///
	/// UN FLAGGED
	/// Vectors of length 1, used to give information about whereabouts and
	/// rotation of the player
	pub struct BasePositionNormalVectors; type = Vec3
);

thrust_stage!(
	/// FLAGGED
	/// Vectors of length 1, used as a helper stage. signed
	/// This MUST take into account the InputFlags, the 'flagged' part of the name
	/// means when the input flags are None, then the vector is zero [Vec3::ZERO]
	pub struct FlaggedPositionNormalVectors; type = Signed<Vec3>
);

thrust_stage!(
	/// type = [f32]
	///
	/// UN FLAGGED
	///
	/// Used for *braking*
	///
	/// Vectors of maximum, used as a helper stage
	pub struct MaxAllowableVelocityMagnitudes; type = f32
);

thrust_stage!(
	/// type = [f32]
	///
	/// Semi flagged, because taking actual player velocity into account.
	///
	/// Used for UI and for *braking*.
	/// Is public, and used for the HUD.
	///
	/// Represents the factor of current velocity, broken up into separate
	/// **linearly independent** dimensions.
	pub struct RelativeVelocityMagnitudes; type = f32
);

thrust_stage!(
	/// type = [f32]
	///
	/// FLAGGED
	///
	/// UI and processing
	///
	/// Shows how much of the maximum power can be used
	/// Used for animating the player and for relative readings
	#[derive(Default)]
	pub struct RelativeStrength; type = f32
);

thrust_stage!(
	/// UN FLAGGED
	/// Factors multiplied into the final thrust vectors
	pub struct ForceFactors; type = f32
);

thrust_stage!(
	/// Final result which is applied to physics engine
	pub struct FinalVectors; type = Vec3
);

thrust_stage!(
	/// Just needs to be multiplied by the [ForceFactors] before it is a final thrust type
	pub struct AlmostFinalVectors; type = Vec3
);

impl std::ops::Mul<Thrust<BasePositionNormalVectors>> for Thrust<GenericInputFlags> {
	type Output = Thrust<FlaggedPositionNormalVectors>;

	fn mul(self, base: Thrust<BasePositionNormalVectors>) -> Self::Output {
		Thrust::<FlaggedPositionNormalVectors> {
			forward: self.forward.wrap_signed(base.forward),
			right: self.right.wrap_signed(base.right),
			up: self.up.wrap_signed(base.up),

			roll_right: self.roll_right.wrap_signed(base.roll_right),
			turn_right: self.turn_right.wrap_signed(base.turn_right),
			tilt_up: self.tilt_up.wrap_signed(base.tilt_up),

			_stage: PhantomData,
		}
	}
}

// relative (F) * normals (U) = almost final (FLAGGED)
impl std::ops::Mul<Thrust<RelativeStrength>> for Thrust<BasePositionNormalVectors> {
	type Output = Thrust<AlmostFinalVectors>;

	fn mul(self, rhs: Thrust<RelativeStrength>) -> Self::Output {
		Thrust::<AlmostFinalVectors> {
			forward: self.forward * rhs.forward,
			up: self.up * rhs.up,
			right: self.right * rhs.right,

			turn_right: self.turn_right * rhs.turn_right,
			tilt_up: self.tilt_up * rhs.tilt_up,
			roll_right: self.roll_right * rhs.roll_right,
			_stage: PhantomData,
		}
	}
}

// Almost final * force factor
impl std::ops::Mul<Thrust<ForceFactors>> for Thrust<AlmostFinalVectors> {
	type Output = Thrust<FinalVectors>;

	fn mul(self, rhs: Thrust<ForceFactors>) -> Self::Output {
		Thrust::<FinalVectors> {
			forward: self.forward * rhs.forward,
			up: self.up * rhs.up,
			right: self.right * rhs.right,

			turn_right: self.turn_right * rhs.turn_right,
			tilt_up: self.tilt_up * rhs.tilt_up,
			roll_right: self.roll_right * rhs.roll_right,
			_stage: PhantomData,
		}
	}
}

// force factor * constant
impl std::ops::Mul<f32> for Thrust<ForceFactors> {
	type Output = Thrust<ForceFactors>;

	fn mul(self, rhs: f32) -> Self::Output {
		Thrust::<ForceFactors> {
			forward: self.forward * rhs,
			up: self.up * rhs,
			right: self.right * rhs,

			turn_right: self.turn_right * rhs,
			tilt_up: self.tilt_up * rhs,
			roll_right: self.roll_right * rhs,

			_stage: PhantomData,
		}
	}
}
