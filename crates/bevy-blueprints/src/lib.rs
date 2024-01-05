mod prelude {
	pub use crate::plugin::*;
	pub use crate::sets::*;
	pub use crate::components::*;

	// bevy
	pub use bevy::prelude::*;
	pub use bevy::ecs::schedule::{ScheduleLabel, InternedScheduleLabel};
}

mod plugin {
	use crate::prelude::*;

	#[derive(Debug)]
	pub struct BlueprintsPlugin {
		schedule: InternedScheduleLabel,
	}

	impl Plugin for BlueprintsPlugin {
		fn build(&self, app: &mut App) {
			type BS = BlueprintsSet;
			app
				.register_type::<BlueprintNeedsUpdating>()
				.configure_sets(
					self.schedule,
					(
						BS::ApplyDeferred1,
						BS::MarkChanged,
						BS::ApplyDeferred2,
						BS::ExpandBlueprints,
						BS::ApplyDeferred3,
					)
						.chain(),
				);
		}
	}

	impl BlueprintsPlugin {
		pub fn new(schedule: impl ScheduleLabel) -> Self {
			Self {
				schedule: schedule.intern(),
			}
		}
	}
}

mod sets {
	use crate::prelude::*;

	/// Set that is initialized by the [BlueprintsPlugin].
	#[derive(SystemSet, Hash, Clone, Eq, PartialEq, Debug)]
	pub enum BlueprintsSet {
		ApplyDeferred1,

		/// Adds the [BlueprintNeedsUpdating] marker component to
		/// relevant [Entity]s.
		MarkChanged,

		ApplyDeferred2,

		/// Expands blueprints that have the [BlueprintNeedsUpdating] marker component.
		ExpandBlueprints,

		ApplyDeferred3,
	}
}

mod components {
	use crate::prelude::*;

	/// Marker [Component] that communicates an [Entity] is still in the
	/// process of being expanded.
	/// Added in the [BlueprintsSet] [BlueprintsSet::MarkChanged],
	/// and maybe removed in the [BlueprintsSet::ExpandBlueprints] depending
	/// on implementation details per blueprint type.
	#[derive(Component, Reflect, Debug, Default)]
	pub struct BlueprintNeedsUpdating;
}

#[cfg(test)]
mod tests {
	use crate::prelude::*;

	fn test_app() -> App {
		App::new()
	}

	#[derive(ScheduleLabel, Hash, Clone, Copy, PartialEq, Eq, Debug)]
	struct BlueprintSchedule;

	#[test]
	fn plugin_initializes() {
		let mut app = test_app();

		app.add_plugins((MinimalPlugins, BlueprintsPlugin::new(BlueprintSchedule)));
	}
}
