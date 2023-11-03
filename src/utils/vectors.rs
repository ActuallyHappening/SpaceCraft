use std::ops::RangeInclusive;

use rand::rngs::ThreadRng;

use crate::utils::*;

/// Dividing space by radius from the origin
pub enum SpaceRegions {
	FarAway,
	VisibleNotInsidePlayer,
}

pub enum VelocityRanges {
	Slow,
}

#[extension(pub trait RangeDefinedRegion)]
impl SpaceRegions {
	fn range(&self) -> RangeInclusive<f32> {
		match self {
			Self::FarAway => 20_000. ..=400_000.,
			Self::VisibleNotInsidePlayer => 50. ..=1_000.,
		}
	}
}

impl VelocityRanges {
	/// 0: linvel length ranges, 1: angvel length ranges
	fn ranges(&self) -> (RangeInclusive<f32>, RangeInclusive<f32>) {
		(0. ..=3., 0. ..=0.05)
	}
}

#[extension(trait Vec3Ext)]
impl Vec3 {
	/// Radius in normal units
	/// Theta in radians
	/// phi in radians
	fn from_polar_normal(theta: f32, phi: f32) -> Self {
		Self {
			x: theta.sin() * phi.cos(),
			y: theta.sin() * phi.sin(),
			z: theta.cos(),
		}
	}

	fn gen_random_sphere_normal(rng: &mut ThreadRng,) -> Self {
		let phi = rng.gen_range(0. ..TAU);
		let z = rng.gen_range(-1. ..1.);

		assert!((0f32..TAU).contains(&phi));
		assert!((-1f32..=1f32).contains(&z));

		// let phi = 1. - seed2.powi(2);
		let theta = z.acos();
		// let phi = rng.gen_range(0. .. PI);

		// let ret = Vec3::new(phi.mul(theta.cos()), phi.mul(theta.sin()), seed2);
		let ret = Vec3::from_polar_normal(theta, phi);

		assert!(
			ret.length().round() == 1.,
			"ret length: {} (rounded = {})",
			ret.length(),
			ret.length().round()
		);

		ret.normalize()
	}
}

fn checked_random(range: RangeInclusive<f32>) -> Vec3 {
	let (lower, upper) = (*range.start(), *range.end());
	assert!(lower >= 0., "Lower bound passed to random util func must be greater than 0, because radius must be positive.");
	let mut rng = rand::thread_rng();

	let normal = Vec3::gen_random_sphere_normal(&mut rng);

	let ret = normal.mul(rng.gen_range(lower..=upper));

	assert!(range.contains(&ret.length()));

	ret
}

pub fn random_pos(range: SpaceRegions) -> Vec3 {
	checked_random(range.range())
}

pub fn random_velocity(range: VelocityRanges) -> Velocity {
	Velocity {
		linvel: checked_random(range.ranges().0),
		angvel: checked_random(range.ranges().1),
	}
}
