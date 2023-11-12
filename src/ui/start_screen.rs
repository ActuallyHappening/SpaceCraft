use bevy::sprite::Mesh2dHandle;
use bevy::text::Text2dBounds;

use super::manual_ui::*;
use super::path_tracing::*;
use super::ui_cameras::CorrectCamera;
use crate::netcode::NetcodeConfig;
use crate::prelude::*;

/// Plugin
pub struct StartScreen;

/// Which CAM this entity belongs to, for start screen only
#[derive(Component, Deref)]
struct Cam(UiCameras);

impl Plugin for StartScreen {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				ButtonParticle::follow_parent_bbox,
				StartScreen::handle_hover_interactions,
				StartScreen::handle_click_interactions,
			)
				.run_if(in_state_start_menu),
		);

		// initial menu
		app
			.add_systems(
				OnEnter(GlobalGameStates::StartMenu(StartScreenStates::Initial)),
				// spawning
				Self::spawn_initial,
			)
			.add_systems(
				OnEnter(GlobalGameStates::InGame),
				// cleanup
				Self::despawn_initial,
			);

		// hosting submenu
		app
			.add_systems(
				OnEnter(GlobalGameStates::StartMenu(
					StartScreenStates::ConfigureHosting,
				)),
				Self::spawn_configure_host,
			)
			.add_systems(
				OnExit(GlobalGameStates::StartMenu(
					StartScreenStates::ConfigureHosting,
				)),
				Self::despawn_configure_host,
			);

		// client submenu
		app
			.add_systems(
				OnEnter(GlobalGameStates::StartMenu(
					StartScreenStates::ConfigureClient,
				)),
				Self::spawn_configure_client,
			)
			.add_systems(
				OnExit(GlobalGameStates::StartMenu(
					StartScreenStates::ConfigureClient,
				)),
				Self::despawn_configure_client,
			);
	}
}

/// List of buttons that can be clicked
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum InitialUiButtons {
	InitialHostGame,
	InitialJoinGame,
	// InitialSolo,
}

impl InitialUiButtons {
	const fn get_text(self) -> &'static str {
		match self {
			InitialUiButtons::InitialHostGame => "Host Game",
			InitialUiButtons::InitialJoinGame => "Join Game",
		}
	}
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum HostGameButtons {
	HostPublicGame,
	HostMachineLocalGame,
}

impl HostGameButtons {
	const fn get_text(self) -> &'static str {
		match self {
			HostGameButtons::HostPublicGame => "Host Public Game",
			HostGameButtons::HostMachineLocalGame => "Host Machine-Local Game",
		}
	}
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum ClientGameButtons {
	// PublicGame,
	MachineLocalGame,
}

impl ClientGameButtons {
	const fn get_text(self) -> &'static str {
		match self {
			// ClientGameButtons::HostPublicGame => "Host Public Game",
			ClientGameButtons::MachineLocalGame => "Join Machine-Local Game",
		}
	}
}

impl StartScreen {
	const INITIAL_CAM: UiCameras = UiCameras::MiddleLeft;

	fn spawn_initial(
		mut commands: Commands,
		mut mma: MM2,
		ass: Res<AssetServer>,
		mut effects: ResMut<Assets<EffectAsset>>,
	) {
		let mut column = ManualColumn {
			const_x: 200.,
			const_width: 200.,
			current_y: 0.,
			item_height: 50.,
			margin: 10.,
		}
		.center_with(2);

		for btn in InitialUiButtons::iter() {
			let manual_node = column.next();
			let wrap_size = manual_node.bbox.dimensions();
			commands
				.spawn(GameButtonBundle::new(btn, manual_node, &mut mma))
				.render_layer(GlobalRenderLayers::Ui(Self::INITIAL_CAM))
				.insert(Cam(Self::INITIAL_CAM))
				.with_children(|parent| {
					parent
						.spawn(ButtonParticles::new(&mut effects))
						.render_layer(GlobalRenderLayers::Ui(Self::INITIAL_CAM));
					parent
						.spawn(ButtonText::new(btn.get_text(), 40., wrap_size, &ass))
						.render_layer(GlobalRenderLayers::Ui(Self::INITIAL_CAM));
				});
		}
	}

	fn despawn_initial(mut commands: Commands, btns: Query<Entity, With<InitialUiButtons>>) {
		for btn in btns.iter() {
			commands.entity(btn).despawn_recursive();
		}
	}

	const HOST_CAM: UiCameras = UiCameras::MiddleRight;

	fn spawn_configure_host(
		mut commands: Commands,
		mut mma: MM2,
		ass: Res<AssetServer>,
		mut effects: ResMut<Assets<EffectAsset>>,
	) {
		let mut column = ManualColumn {
			const_x: -200.,
			const_width: 200.,
			current_y: 0.,
			item_height: 50.,
			margin: 10.,
		}
		.center_with(2);

		for btn in HostGameButtons::iter() {
			let manual_node = column.next();
			let text_wrap = manual_node.bbox.dimensions();
			commands
				.spawn(GameButtonBundle::new(btn, manual_node, &mut mma))
				.render_layer(GlobalRenderLayers::Ui(Self::HOST_CAM))
				.insert(Cam(Self::HOST_CAM))
				.with_children(|parent| {
					parent
						.spawn(ButtonParticles::new(&mut effects))
						.render_layer(GlobalRenderLayers::Ui(Self::HOST_CAM));
					parent
						.spawn(ButtonText::new(btn.get_text(), 25., text_wrap, &ass))
						.render_layer(GlobalRenderLayers::Ui(Self::HOST_CAM));
				});
		}
	}

	fn despawn_configure_host(mut commands: Commands, btns: Query<Entity, With<HostGameButtons>>) {
		for btn in btns.iter() {
			commands.entity(btn).despawn_recursive();
		}
	}

	const CLIENT_CAM: UiCameras = UiCameras::MiddleRight;

	fn spawn_configure_client(
		mut commands: Commands,
		mut mma: MM2,
		ass: Res<AssetServer>,
		mut effects: ResMut<Assets<EffectAsset>>,
	) {
		let mut column = ManualColumn {
			const_x: -200.,
			const_width: 200.,
			current_y: 0.,
			item_height: 50.,
			margin: 10.,
		}
		.center_with(2);

		for btn in ClientGameButtons::iter() {
			let manual_node = column.next();
			let text_wrap = manual_node.bbox.dimensions();
			commands
				.spawn(GameButtonBundle::new(btn, manual_node, &mut mma))
				.render_layer(GlobalRenderLayers::Ui(Self::CLIENT_CAM))
				.insert(Cam(Self::CLIENT_CAM))
				.with_children(|parent| {
					parent
						.spawn(ButtonParticles::new(&mut effects))
						.render_layer(GlobalRenderLayers::Ui(Self::CLIENT_CAM));
					parent
						.spawn(ButtonText::new(btn.get_text(), 25., text_wrap, &ass))
						.render_layer(GlobalRenderLayers::Ui(Self::CLIENT_CAM));
				});
		}
	}

	fn despawn_configure_client(
		mut commands: Commands,
		btns: Query<Entity, With<ClientGameButtons>>,
	) {
		for btn in btns.iter() {
			commands.entity(btn).despawn_recursive();
		}
	}

	fn handle_hover_interactions(
		mut start_hover_events: EventReader<Pointer<Move>>,
		mut end_hover_events: EventReader<Pointer<Out>>,
		this: Query<(&Cam, &Children)>,
		mut particle_spawners: Query<&mut EffectSpawner>,
		correct_camera: CorrectCamera,
	) {
		for start_event in start_hover_events.read() {
			if let Ok((cam, this)) = this.get(start_event.target) {
				// found callback target
				let camera = start_event.event.hit.camera;
				if correct_camera.confirm(&camera, **cam) {
					// correct camera

					if let Some(particle_spawner_entity) = this
						.iter()
						.find(|child| particle_spawners.get(**child).is_ok())
					{
						// found particle spawner
						let mut spawner = particle_spawners.get_mut(*particle_spawner_entity).unwrap();

						spawner.set_active(true);
					} else {
						warn!("Cannot find particle spawner");
					}
				}
			} else {
				warn!("Cannot find target callback");
			}
		}

		for end_event in end_hover_events.read() {
			if let Ok((cam, this)) = this.get(end_event.target) {
				// found callback target
				let camera = end_event.event.hit.camera;
				if correct_camera.confirm(&camera, **cam) {
					// correct camera

					if let Some(particle_spawner_entity) = this
						.iter()
						.find(|child| particle_spawners.get(**child).is_ok())
					{
						// found particle spawner
						let mut spawner = particle_spawners.get_mut(*particle_spawner_entity).unwrap();

						spawner.set_active(false);
					} else {
						warn!("Cannot find particle spawner");
					}
				}
			} else {
				warn!("Cannot find target callback");
			}
		}
	}

	fn handle_click_interactions(
		mut click_events: EventReader<Pointer<Click>>,
		initial_btns: Query<(&Cam, &InitialUiButtons)>,
		host_btns: Query<(&Cam, &HostGameButtons)>,
		client_btns: Query<(&Cam, &ClientGameButtons)>,
		correct_camera: CorrectCamera,

		mut next_state: ResMut<NextState<GlobalGameStates>>,
		mut commands: Commands,
	) {
		for click_event in click_events.read() {
			if let Ok((cam, btn)) = initial_btns.get(click_event.target) {
				// found callback target
				let camera = click_event.event.hit.camera;
				if correct_camera.confirm(&camera, **cam) {
					// correct camera

					match btn {
						InitialUiButtons::InitialHostGame => {
							next_state.set(GlobalGameStates::StartMenu(
								StartScreenStates::ConfigureHosting,
							));
						}
						InitialUiButtons::InitialJoinGame => {
							next_state.set(GlobalGameStates::StartMenu(
								StartScreenStates::ConfigureClient,
							));
						}
					}
				}
			} else if let Ok((cam, btn)) = host_btns.get(click_event.target) {
				// found callback target
				let camera = click_event.event.hit.camera;
				if correct_camera.confirm(&camera, **cam) {
					// correct camera

					next_state.set(GlobalGameStates::InGame);
					commands.insert_resource(match btn {
						HostGameButtons::HostPublicGame => NetcodeConfig::new_hosting_public(),
						HostGameButtons::HostMachineLocalGame => NetcodeConfig::new_hosting_machine_local(),
					});
				}
			} else if let Ok((cam, btn)) = client_btns.get(click_event.target) {
				// found callback target
				let camera = click_event.event.hit.camera;
				if correct_camera.confirm(&camera, **cam) {
					// correct camera

					next_state.set(GlobalGameStates::InGame);
					commands.insert_resource(match btn {
						ClientGameButtons::MachineLocalGame => NetcodeConfig::new_client_machine_local(),
					});
				}
			} else {
				warn!("Cannot find target callback");
			}
		}
	}
}

#[derive(Bundle)]
struct GameButtonBundle<T: Component + Send + Sync + 'static> {
	mesh: Mesh2dHandle,
	material: Handle<ColorMaterial>,
	spatial: SpatialBundle,
	path: BevyPath,

	btn: T,

	name: Name,
}

impl<T: Component + Send + Sync + 'static> GameButtonBundle<T> {
	fn new(btn: T, manual_node: ManualNode, mma: &mut MM2) -> Self {
		Self {
			btn,
			mesh: mma
				.meshs
				.add(
					shape::Quad::new(Vec2::new(
						manual_node.bbox.half_width * 2.,
						manual_node.bbox.half_height * 2.,
					))
					.into(),
				)
				.into(),
			material: mma.mats.add(Color::WHITE.into()),
			spatial: SpatialBundle::from_transform(Transform::from_xyz(
				manual_node.position.x,
				manual_node.position.y,
				1.,
			)),
			name: Name::new("Host Game Button"),
			path: BevyPath::rectangle_from_bbox(manual_node.bbox),
		}
	}
}

#[derive(Bundle)]
struct ButtonText {
	text_bundle: Text2dBundle,

	name: Name,
	// render_layer: RenderLayers,
}

impl ButtonText {
	fn new(
		// cam: UiCameras,
		text: impl Into<Cow<'static, str>>,
		font_size: f32,
		wrap_size: Vec2,
		ass: &AssetServer,
	) -> Self {
		let style = TextStyle {
			font: ass.load(GlobalFont::Default),
			font_size,
			color: Color::MIDNIGHT_BLUE,
		};

		ButtonText {
			text_bundle: Text2dBundle {
				text: Text::from_section(text.into(), style.clone()).with_alignment(TextAlignment::Center),
				transform: Transform::from_translation(Vec3::Z),
				text_2d_bounds: Text2dBounds { size: wrap_size },
				..default()
			},
			name: Name::new("Button Text"),
			// render_layer: GlobalRenderLayers::Ui(cam).into(),
		}
	}
}

#[derive(Component)]
struct ButtonParticle;

#[derive(Bundle)]
struct ButtonParticles {
	particles: ParticleEffectBundle,
	marker: ButtonParticle,

	// layer: RenderLayers,
	name: Name,
}

impl ButtonParticles {
	fn new(
		// cam: UiCameras,
		mut effects: &mut Assets<EffectAsset>,
	) -> Self {
		let mut gradient = Gradient::new();
		// gradient.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 1.0));
		// gradient.add_key(0.1, Vec4::new(0.5, 0.5, 0.0, 1.0));
		// gradient.add_key(0.4, Vec4::new(0.5, 0.0, 0.0, 1.0));
		// gradient.add_key(1.0, Vec4::splat(0.0));
		gradient.add_key(0.0, Vec4::splat(1.));
		gradient.add_key(1.0, Vec4::new(0., 0., 0., 1.));

		let writer = ExprWriter::new();

		let age = writer.lit(0.).uniform(writer.lit(0.3)).expr();
		let init_age = SetAttributeModifier::new(Attribute::AGE, age);

		let lifetime = writer.lit(1.).uniform(writer.lit(1.5)).expr();
		let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

		let init_pos = SetPositionSphereModifier {
			center: writer.lit(Vec3::ZERO).expr(),
			radius: writer.lit(25.).expr(),
			dimension: ShapeDimension::Volume,
		};

		let init_vel = SetVelocitySphereModifier {
			center: writer.lit(Vec3::ZERO).expr(),
			speed: writer.lit(15.).expr(),
		};

		let effect = effects.add(
			EffectAsset::new(
				32768,
				Spawner::rate(1000.0.into()).with_starts_active(false),
				writer.finish(),
			)
			.with_name("gradient")
			.init(init_pos)
			.init(init_vel)
			.init(init_age)
			.init(init_lifetime)
			.render(ColorOverLifetimeModifier { gradient }),
		);

		Self {
			particles: ParticleEffectBundle::new(effect),
			marker: ButtonParticle,
			// layer: GlobalRenderLayers::Ui(cam).into(),
			name: Name::new("Button Particles"),
		}
	}
}

impl ButtonParticle {
	fn follow_parent_bbox(
		mut spawner: Query<(&Parent, &mut Transform), With<Self>>,
		parents: Query<&BevyPath>,
		time: Res<Time>,
	) {
		for (parent, mut transform) in spawner.iter_mut() {
			if let Ok(path) = parents.get(parent.get()) {
				const FACTOR: f32 = 250.;
				let time = time.elapsed().as_millis() as f32 % FACTOR / FACTOR;
				let pos = path.get_pos_at_time(time);
				transform.translation.x = pos.x;
				transform.translation.y = pos.y;
			} else {
				error!("Particle spawner's parent does not have a BBox component");
			}
		}
	}
}
