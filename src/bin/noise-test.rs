use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::NoiseFn;
use noise::{Fbm, Perlin};

fn main() {
	let fbm = Fbm::<Perlin>::new(0);

	let width = 10.;

	let noise = PlaneMapBuilder::<_, 2>::new(&fbm)
		.set_size(10, 10)
		.set_x_bounds(-width, width)
		.set_y_bounds(-width, width)
		.build();

	println!("Pixel at (0, 0): {}", noise.get_value(1, 0));

	noise.write_to_file("fbm.png");

	// let noise = Perlin::new(69);
	let noise = Fbm::<Perlin>::new(69);

	for x in 0..10 {
		for y in 0..10 {
			println!("Perlin at ({y}, {y}): {}", noise.get([x as f64, y as f64]));
		}
	}
}
