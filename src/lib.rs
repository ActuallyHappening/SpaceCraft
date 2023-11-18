mod global;
mod netcode;
mod players;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

use self::netcode::NetcodePlugin;
use self::players::PlayerPlugins;
use self::ui::UiPlugins;
#[allow(unused_imports)]
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};
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
				(
					GlobalSystemSet::BlueprintExpansion("player"),
					GlobalSystemSet::BlueprintExpansion("blocks"),
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

		// spawn initial Main Camera
		app.add_systems(Startup, |mut commands: Commands| {
			commands.spawn(DirectionalLightBundle {
				directional_light: DirectionalLight {
					shadows_enabled: true,
					..default()
				},
				..default()
			});

			commands
				.spawn((
					Camera3dBundle {
						transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
						camera: Camera {
							hdr: true,
							..default()
						},
						camera_3d: Camera3d {
							clear_color: ClearColorConfig::Custom(Color::BLACK),
							..default()
						},
						tonemapping: Tonemapping::None,
						..default()
					},
					// BloomSettings {
					// 	intensity: 1.0,
					// 	low_frequency_boost: 0.5,
					// 	low_frequency_boost_curvature: 0.5,
					// 	high_pass_frequency: 0.5,
					// 	prefilter_settings: BloomPrefilterSettings {
					// 		threshold: 3.0,
					// 		threshold_softness: 0.6,
					// 	},
					// 	composite_mode: BloomCompositeMode::Additive,
					// },
				))
				.insert(VisibilityBundle::default())
				.named("Main Camera")
				.render_layer(GlobalRenderLayers::InGame);
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
			HanabiPlugin,
			ReplicationPlugins
				.build()
				.set(ServerPlugin::new(TickPolicy::Manual)),
			NetcodePlugin,
			TimewarpPlugin::new(TimewarpConfig::new(
				GlobalSystemSet::GameLogic,
				GlobalSystemSet::GameLogic,
			)),
		));
		app.add_plugins((
			PlayerPlugins,
			UiPlugins,
		));
		app.replicate::<Position>().replicate::<Rotation>();
	}
}
