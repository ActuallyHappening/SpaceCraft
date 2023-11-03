use super::*;

pub enum StructureBundle {
	Pixel {
		visual: PbrBundle,
	},
	Thruster {
		visual: PbrBundle,
		data: Thruster,
		particles: ParticleEffectBundle,
	},
	Weapon {
		visual: PbrBundle,
		data: Weapon,
	}
}

impl StructureBundle {
	pub fn default_spawn_to_parent(self, parent: &mut ChildBuilder) {
		match self {
			StructureBundle::Pixel { visual } => {
				parent.spawn(visual);
			}
			StructureBundle::Thruster {
				visual,
				particles,
				data,
			} => {
				parent.spawn(visual).with_children(|parent| {
					parent.spawn(particles.insert(data));
				});
			}
			StructureBundle::Weapon { visual, data } => {
				parent.spawn(visual.insert(data));
			}
		};
	}
}
