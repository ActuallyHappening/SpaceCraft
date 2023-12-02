// #![deny(clippy::arithmetic_side_effects)]
// #![warn(
// 	clippy::all,
// 	// clippy::restriction,
// 	clippy::pedantic,
// 	clippy::nursery,
// 	clippy::cargo
// )]
// #![allow(
// 	clippy::module_name_repetitions,
// 	clippy::needless_pass_by_value,
// 	clippy::too_many_arguments,
// 	clippy::wildcard_imports,
// 	clippy::use_self
// )]
#![windows_subsystem = "windows"]
#![allow(clippy::too_many_arguments)]

mod blocks;
mod cameras;
mod global;
mod netcode;
mod players;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

#[allow(unused_imports)]
use bevy_mod_picking::{
	prelude::{DebugPickingPlugin, DefaultHighlightingPlugin},
	DefaultPickingPlugins,
};
use bevy_replicon::ReplicationPlugins;

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

		info!("MainPlugin initializing ...");
		app.add_systems(Startup, || info!("Startup running"));

		// global system set configuration
		app.configure_sets(
			FixedUpdate,
			(
				(BlueprintExpansion::Player, BlueprintExpansion::Thruster)
					.chain()
					.in_set(GlobalSystemSet::BlueprintExpansion),
				(
					GlobalSystemSet::BlueprintExpansion,
					GlobalSystemSet::ThrustersSync,
					GlobalSystemSet::RawPhysics,
					GlobalSystemSet::GameLogic,
				)
					.chain(),
				(
					PhysicsSet::Prepare,
					PhysicsSet::StepSimulation,
					PhysicsSet::Sync,
				)
					.in_set(GlobalSystemSet::RawPhysics),
				ServerSet::Send.after(GlobalSystemSet::GameLogic),
			),
		);

		// spawn initial light
		app.add_systems(Startup, |mut commands: Commands| {
			commands.spawn(DirectionalLightBundle {
				directional_light: DirectionalLight {
					shadows_enabled: true,
					..default()
				},
				..default()
			});
		});

		// will take cli inputs, or default to start menu
		// app.add_state::<GlobalGameStates>();
		let state;
		if std::env::args().len() > 1 {
			info!("Using options provided by CLI");
			state = GlobalGameStates::InGame;
			app.insert_resource(self::netcode::NetcodeConfig::parse());
		} else {
			state = GlobalGameStates::StartMenu;
		}
		app
			.insert_resource::<State<GlobalGameStates>>(State::new(state))
			.init_resource::<NextState<GlobalGameStates>>()
			.add_systems(
				StateTransition,
				(
					bevy::ecs::schedule::run_enter_schedule::<GlobalGameStates>.run_if(run_once()),
					apply_state_transition::<GlobalGameStates>,
				)
					.chain(),
			);

		// dep plugins
		app.add_plugins((
			bevy_editor_pls::EditorPlugin::default(),
			ScreenDiagnosticsPlugin::default(),
			ScreenFrameDiagnosticsPlugin,
			DefaultPickingPlugins
				.build()
				// .disable::<DebugPickingPlugin>()
				.disable::<DefaultHighlightingPlugin>(),
			PhysicsPlugins::new(FixedUpdate),
			PhysicsDebugPlugin::default(),
			bevy_xpbd3d_parenting::PhysicsParentingPlugin,
			HanabiPlugin,
			ReplicationPlugins
				.build()
				.set(ServerPlugin::new(TickPolicy::Manual)),
			TimewarpPlugin::new(TimewarpConfig::new(
				GlobalSystemSet::GameLogic,
				GlobalSystemSet::GameLogic,
			)),
			crate::utils::scenes::HelperScene,
		));
		// dep configuration
		app.insert_resource(Gravity(Vec3::ZERO));

		// game logic plugins
		app.add_plugins((
			self::netcode::NetcodePlugin,
			self::cameras::CameraPlugin,
			self::ui::UiPlugins,
			self::players::PlayerPlugins,
		));
		app.register_type::<BlockId>();

		// network replication
		app.replicate::<Position>().replicate::<Rotation>();
	}
}
