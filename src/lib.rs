mod bouncing_balls;
mod global;
mod netcode;
mod player;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

use self::netcode::NetcodePlugin;
use self::player::PlayerPlugins;
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
				GlobalSystemSet::AfterPhysics.after(GlobalSystemSet::Physics),
				(PhysicsSet::Prepare, PhysicsSet::StepSimulation, PhysicsSet::Sync).in_set(GlobalSystemSet::Physics),
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
					BloomSettings {
						intensity: 1.0,
						low_frequency_boost: 0.5,
						low_frequency_boost_curvature: 0.5,
						high_pass_frequency: 0.5,
						prefilter_settings: BloomPrefilterSettings {
							threshold: 3.0,
							threshold_softness: 0.6,
						},
						composite_mode: BloomCompositeMode::Additive,
					},
				))
				.insert(VisibilityBundle::default())
				.named("Main Camera")
				.render_layer(GlobalRenderLayers::InGame);
		});

		// states
		app.add_state::<GlobalGameStates>();

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
			PlayerPlugins,
			UiPlugins,
			ReplicationPlugins,
			NetcodePlugin,
		));

		// testing
		// app.add_systems(
		// 	OnEnter(GlobalGameStates::InGame),
		// 	|mut commands: Commands, mut mma: MMA| {
		// 		commands.spawn((
		// 			PbrBundle {
		// 				material: mma.mats.add(StandardMaterial {
		// 					emissive: Color::RED * 5.,

		// 					..default()
		// 				}),
		// 				mesh: mma.meshs.add(
		// 					shape::Capsule {
		// 						radius: 0.1,
		// 						rings: 5,
		// 						depth: 4.,
		// 						..default()
		// 					}
		// 					.into(),
		// 				),
		// 				..default()
		// 			},
		// 			Name::new("Bullet"),
		// 		));
		// 	},
		// );
	}
}
