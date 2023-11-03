use bevy::prelude::*;

use self::randomness::WorldGen;
use crate::{core::Pixel, utils::*};

pub fn spawn_random_world(mut commands: Commands, mut mma: MMA) {
	let r = WorldGen::<WORLD_WIDTH>::new(7);
	for (x, height, z) in r {
		for y in 0..height {
			let point = WorldPoint { x, y, z };
			commands.spawn(PixelVariant::weighted_random_pixel().into_bundle(point, &mut mma));
		}
	}
}

impl PixelVariant {
	fn weighted_natural_pool() -> (Vec<(PixelVariant, u16)>, u32) {
		let pool = PixelVariant::natural_pool();
		let sum = pool
			.iter()
			.map(|(_, natural)| natural.frequency as u32)
			.sum();
		(
			pool.into_iter().map(|(p, n)| (p, n.frequency)).collect(),
			sum,
		)
	}

	fn weighted_random_pixel() -> Pixel {
		let (pool, sum) = Self::weighted_natural_pool();
		let r = rand::random::<u32>() % sum;

		let mut acc = 0;
		for (pixel, freq) in pool.into_iter() {
			acc += freq as u32;
			if acc >= r {
				return pixel.get_default_pixel();
			}
		}
		unreachable!("This should never happen unless there are no pixels in the pool");
	}
}

impl Pixel {
	fn into_bundle(self, point: WorldPoint, MMA { meshs, mats, .. }: &mut MMA) -> impl Bundle {
		(
			PbrBundle {
				material: mats.add(self.colour.into()),
				mesh: meshs.add(Mesh::from(shape::Cube { size: PIXEL_SIZE })),
				transform: Transform::from_translation(point.into_bevy_vector()),
				..Default::default()
			},
			self,
			On::<Pointer<Down>>::run(handle_callback),
		)
			.pickable()
	}
}

fn handle_callback(
	event: Listener<Pointer<Down>>,
	query: Query<&Pixel>,
	mut send_e: EventWriter<PlayerMinedPixel>,
) {
	if let Ok(component) = query.get(event.target()) {
		if let Some(e) = PlayerMinedPixel::new(component.clone()) {
			send_e.send(e);
		}
	} else {
		error!("No entity found??");
	}
}

mod randomness {
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

	/// Iterator yielding (i32, i32, i32) with x&z in range [-TW/2, TW/2) and y in range [0, height)
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
				assert!(h != 0.);
				println!("Height at ({}, {}): {}", x, z, h);
			}
		}
	}
}
