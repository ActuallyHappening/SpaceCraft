use bevy::prelude::*;

// example types

#[derive(Resource, Clone)] // Clone for simplicity, there may be a way to relax this requirement
pub struct ExampleCollectReturn;

/// do some computation, maybe read from a [Query]
pub fn collect() -> ExampleCollectReturn {
	ExampleCollectReturn
}

/// time for demonstration purposes, you can put any system params you like
pub fn consumer_1(In(_collect_return): In<ExampleCollectReturn>, time: Res<Time>) {
	// do something with the result
}

/// query for demonstration purposes, you can put any system params you like
pub fn consumer_2(In(_collect_return): In<ExampleCollectReturn>, query: Query<Entity>) {
	// do something with the result
}

// optimization specific types

#[derive(Resource)]
pub struct _PipingResource {
	collect_return: ExampleCollectReturn,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, SystemSet)]
pub struct _PipingSet;

fn _store_collect_return(In(collect_return): In<ExampleCollectReturn>, mut commands: Commands) {
	commands.insert_resource(_PipingResource { collect_return });
}

fn _get_collect_return(_data: Res<_PipingResource>) -> ExampleCollectReturn {
	// There is probably way to avoid cloning and use references / Box, but its fine for my use cases
	_data.collect_return.clone()
}

// example usage (plugin)

pub struct ExamplePlugin;
impl Plugin for ExamplePlugin {
	fn build(&self, app: &mut App) {
		// not optimized way
		app.add_systems(Update, (collect.pipe(consumer_1), collect.pipe(consumer_2)));

		// optimized way
		app.add_systems(
			Update,
			(
				collect.pipe(_store_collect_return).in_set(_PipingSet),
				_get_collect_return.pipe(consumer_1).after(_PipingSet),
				_get_collect_return.pipe(consumer_2).after(_PipingSet),
			),
		);
	}
}
