use super::RawPixel;
use crate::utils::*;

mod structure;
pub use structure::Structure;

mod structure_bundle;
pub use structure_bundle::StructureBundle;

mod structure_part;
pub use structure_part::StructurePart;

pub trait Reflection {
	fn reflect_horizontally(self) -> Self;
	fn reflect_vertically(self) -> Self;
}

mod thruster;
pub use thruster::Thruster;

mod thruster_flags;
pub use thruster_flags::ThrusterFlags;

mod weapons;
pub use weapons::*;

mod direction;
pub use direction::Direction;

mod relative_pixel_point;
pub use relative_pixel_point::RelativePixelPoint;

#[derive(Component, Constructor, Deref, Serialize, Deserialize, Debug)]
pub struct SpawnChildStructure {
	pub structure: Structure,
}

pub fn hydrate_structure(
	mut commands: Commands,
	mut mma: MMA,
	mut effects: ResMut<Assets<EffectAsset>>,
	skeleton_players: Query<
		(
			Entity,
			&SpawnChildStructure,
			Option<&ComputedVisibility>,
			Option<&GlobalTransform>,
			Option<&Visibility>,
		),
		Added<SpawnChildStructure>,
	>,
) {
	for (entity, structure, computed_visibility, global_transform, visibility) in skeleton_players.iter() {
		debug!("Hydrating structure");

		let mut parent = commands.entity(entity);

		if computed_visibility.is_none() {
			parent.insert(ComputedVisibility::default());
		}
		if global_transform.is_none() {
			parent.insert(GlobalTransform::default());
		}
		if visibility.is_none() {
			parent.insert(Visibility::Inherited);
		}


		// spawn structure
		parent.with_children(|parent| {
			for part in structure.compute_bundles(&mut mma, Some(&mut effects)) {
				part.default_spawn_to_parent(parent);
			}
		});
	}
}
