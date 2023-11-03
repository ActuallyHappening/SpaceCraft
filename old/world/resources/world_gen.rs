use noise::{Fbm, NoiseFn, Perlin};

pub struct WorldGen<const TW: usize> {
	/// Matrix of numbers between 0 and 1
	heights: [[f64; TW]; TW],
	max_height: usize,
}

impl<const TW: usize> WorldGen<TW> {
	pub fn new(max_height: usize) -> Self {
		let noise = Fbm::<Perlin>::new(rand::random());
		let mut ret = Self {
			heights: [[0.; TW]; TW],
			max_height,
		};
		for x in 0..TW {
			for z in 0..TW {
				let h = noise.get([x as f64, z as f64]);
				ret.heights[x][z] = h;
			}
		}

		ret
	}

	fn get_height(&self, x: i32, z: i32) -> f64 {
		assert!(
			x <= TW as i32 / 2 && z <= TW as i32 / 2,
			"x and z must be less than or equal to {}",
			TW / 2
		);

		let x: usize = (x + TW as i32 / 2).try_into().expect("x is too small");
		assert!(
			x < TW,
			"X value of {x} is larger than {} which is half of the total width TM={}",
			TW / 2,
			TW
		);
		let z: usize = (z + TW as i32 / 2).try_into().expect("z is too small");
		assert!(
			z < TW,
			"Z value of {z} is larger than {} which is half of the total width TM={}",
			TW / 2,
			TW
		);

		let ret = (self.heights[x][z] + 1.) / 2.;

		assert!(
			(0. ..=1.).contains(&ret),
			"Height at ({}, {}) is {} which is not expected",
			x,
			z,
			ret
		);

		ret
	}

	fn normalized_into_height(&self, height: f64) -> i32 {
		(height * self.max_height as f64).round() as i32
	}
}

/// Iterator yielding (i32, i32, i32) with x&z in range [-TW/2, TW/2) and y in range [0, 1)
impl<const TW: usize> IntoIterator for WorldGen<TW> {
	type Item = (i32, i32, i32);
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		let radius: i32 = (TW / 2).try_into().expect("TW is too large");
		let mut ret = Vec::with_capacity(TW * TW);
		for x in -radius..radius {
			for z in -radius..radius {
				ret.push((x, self.normalized_into_height(self.get_height(x, z)), z));
			}
		}
		ret.into_iter()
	}
}

#[test]
fn test_world_gen() {
	let world = WorldGen::<10>::new(7);

	for x in -5..5 {
		for z in -5..5 {
			let h = world.get_height(x, z);
			println!("Height at ({}, {}): {}", x, z, h);
		}
	}
}
