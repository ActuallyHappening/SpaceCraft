use crate::utils::*;
use bevy::transform::TransformSystem;
use bevy_hanabi::HanabiPlugin;
use bevy_mod_picking::{
	prelude::{DebugPickingPlugin, DefaultHighlightingPlugin},
	DefaultPickingPlugins,
};

#[cfg(feature = "dev")]
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

mod stars;
use stars::*;
// mod testparticles;

use super::camera::CameraPlugin;
use crate::utils::*;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(Msaa::default())
			.add_systems(Startup, (setup,))
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>(), // .disable::<DebugPickingPlugin>(),
			)
			.add_systems(Update, blink_stars);


		#[cfg(feature = "dev")]
		app.add_plugins((
			ScreenFrameDiagnosticsPlugin,
			ScreenDiagnosticsPlugin::default(),
		));

		app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false));
		app.add_systems(
			Startup,
			|mut physics_config: ResMut<RapierConfiguration>| {
				physics_config.gravity = Vec3::ZERO;
			},
		);

		// manually configures physics systems to only run when not hosting
		app.configure_sets(
			PostUpdate,
			(
				PhysicsSet::SyncBackend,
				PhysicsSet::SyncBackendFlush,
				PhysicsSet::StepSimulation,
				PhysicsSet::Writeback,
			)
				.chain()
				.before(TransformSystem::TransformPropagate),
		);

		app.add_systems(
			PostUpdate,
			(
				RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
					.in_set(PhysicsSet::SyncBackend),
				RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackendFlush)
					.in_set(PhysicsSet::SyncBackendFlush),
				RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
					.in_set(PhysicsSet::StepSimulation),
				RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
					.in_set(PhysicsSet::Writeback),
			)
				.run_if(|state: Res<State<ServerConnections>>| state.should_simulate()),
		);

		#[cfg(feature = "hanabi_particles")]
		app.add_plugins(HanabiPlugin);

		#[cfg(feature = "debugging")]
		app.add_plugins(RapierDebugRenderPlugin::default());

		// #[cfg(feature = "debugging")]
		// app.add_plugins(OverlayPlugin { font_size: 23.0, ..default() });

		#[cfg(feature = "dev")]
		#[cfg(feature = "debugging")]
		app
			.add_plugins((
				bevy_editor_pls::prelude::EditorPlugin::default(),
				// bevy_egui::EguiPlugin,
			))
			.insert_resource(editor_controls());
	}
}

pub fn setup(mut commands: Commands, mut mma: MMA) {
	// cam
	commands.spawn(CameraPlugin::default());

	// light
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 5000000.0,
			range: 25000.,
			// shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0., LIGHT_HEIGHT, 0.),
		..default()
	});

	// stars
	for _ in 0..5_000 {
		stars::spawn_random_star(&mut commands, &mut mma);
		// tracing::info!("Spawned star");
	}
}

#[cfg(feature = "dev")]
fn editor_controls() -> bevy_editor_pls::controls::EditorControls {
	use bevy_editor_pls::controls;
	use bevy_editor_pls::controls::EditorControls;

	let mut editor_controls = EditorControls::default_bindings();
	editor_controls.unbind(controls::Action::PlayPauseEditor);

	editor_controls.insert(
		controls::Action::PlayPauseEditor,
		controls::Binding {
			input: controls::UserInput::Single(controls::Button::Keyboard(KeyCode::Backslash)),
			conditions: vec![controls::BindingCondition::ListeningForText(false)],
		},
	);

	editor_controls
}
