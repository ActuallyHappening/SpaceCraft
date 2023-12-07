use bevy::{prelude::*, ecs::system::SystemParam};

#[allow(clippy::upper_case_acronyms)]
#[derive(SystemParam)]
pub struct MMA<'w> {
	pub meshs: ResMut<'w, Assets<Mesh>>,
	pub mats: ResMut<'w, Assets<StandardMaterial>>,
	pub ass: Res<'w, AssetServer>,
}

trait Blueprint: Clone {
	type For: bevy::ecs::bundle::Bundle;
	type StampSystemParam<'w, 's>: bevy::ecs::system::SystemParam;

	fn stamp<'w, 's>(&self, system_param: &mut Self::StampSystemParam<'w, 's>) -> Self::For;
}

#[derive(Clone)]
struct PBlueprint {
	f: f32
}

#[derive(bevy::ecs::bundle::Bundle)]
struct PBundle {
	pbr: PbrBundle,
}

impl Blueprint for PBlueprint {
	type For = PBundle;
	type StampSystemParam<'w, 's> = MMA<'w>;

	fn stamp<'w, 's>(&self, mma: &mut Self::StampSystemParam<'w, 's>) -> Self::For {
		PBundle {
			pbr: PbrBundle {
				material: mma.mats.add(StandardMaterial {
					..default()
				}),
				..default()
			}
		}
	}
}

fn main() {
	
}