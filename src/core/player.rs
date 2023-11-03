use crate::utils::*;

use super::pixels::PixelVariant;

#[derive(Resource, Debug, DerefMut, Deref)]
pub struct PlayerInventory(HashMap<PixelVariant, u32>);

impl Default for PlayerInventory {
	fn default() -> Self {
		let mut invent = HashMap::new();
		for variant in PixelVariant::iter() {
			invent.insert(variant, 0);
		}
		PlayerInventory(invent)
	}
}

impl std::ops::Index<PixelVariant> for PlayerInventory {
	type Output = u32;

	fn index(&self, index: PixelVariant) -> &Self::Output {
		&self.0[&index]
	}
}

impl std::ops::IndexMut<PixelVariant> for PlayerInventory {
	fn index_mut(&mut self, index: PixelVariant) -> &mut Self::Output {
		self.0.entry(index).or_insert_with(|| {
			error!("PlayerInventory does not contain key {:?}", index);
			0
		});
		self.0.get_mut(&index).unwrap()
	}
}
