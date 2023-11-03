use crate::utils::*;

pub struct StartScreenPlugin;
impl Plugin for StartScreenPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_state::<StartScreens>()
			.add_systems(OnExit(ScreenState::StartScreen), cleanup_ui)
			.add_systems(OnEnter(StartScreens::Default), setup_default_ui)
			.add_systems(OnExit(StartScreens::Default), cleanup_ui)
			.add_systems(
				Update,
				(
					handle_default_ui.run_if(in_state(StartScreens::Default)),
					handle_host_controls_ui.run_if(in_state(StartScreens::HostControls)),
					handle_client_controls_ui.run_if(in_state(StartScreens::ClientControls)),
				)
					.run_if(in_state(ScreenState::StartScreen)),
			)
			.add_systems(OnEnter(StartScreens::HostControls), setup_host_controls_ui)
			.add_systems(OnExit(StartScreens::HostControls), cleanup_ui)
			.add_systems(
				OnEnter(StartScreens::ClientControls),
				setup_client_controls_ui,
			)
			.add_systems(OnExit(StartScreens::ClientControls), cleanup_ui);
	}
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
pub enum StartScreens {
	#[default]
	Default,

	HostControls,
	ClientControls,
}

#[derive(Component)]
struct StartScreenUi;

#[derive(Component)]
struct DefaultBtn(ServerConnections);

fn setup_default_ui(mut commands: Commands, ass: ResMut<AssetServer>) {
	info!("Start screen");
	commands
		.spawn(NodeBundle {
			style: Style {
				width: Val::Percent(100.),
				height: Val::Percent(100.),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.insert(StartScreenUi)
		.with_children(|parent| {
			let btn = |name: &'static str, btn_type: ServerConnections, parent: &mut ChildBuilder| {
				parent
					.spawn(ButtonBundle {
						style: Style {
							width: Val::Px(400.),
							height: Val::Px(50.),
							justify_content: JustifyContent::Center,
							align_items: AlignItems::Center,
							..default()
						},
						background_color: Color::BLACK.into(),
						..default()
					})
					.insert(DefaultBtn(btn_type))
					.with_children(|parent| {
						parent.spawn(TextBundle::from_section(
							name,
							TextStyle {
								font: ass.load(Font::Medium),
								font_size: 30.,
								color: Color::WHITE,
							},
						));
					});
			};
			btn("Start (Hosted) Game", ServerConnections::Hosting, parent);
			btn("Start Private Game", ServerConnections::Local, parent);
			btn("Join hosted Game", ServerConnections::Client, parent);
		});
}

fn handle_default_ui(
	btns: Query<(&DefaultBtn, &Interaction), Changed<Interaction>>,
	mut next_screen: ResMut<NextState<StartScreens>>,
	mut start_game: ResMut<NextState<ServerConnections>>,
) {
	for (btn, interaction) in btns.iter() {
		if interaction == &Interaction::Pressed {
			match btn.0 {
				ServerConnections::Local => start_game.0 = Some(ServerConnections::Local),
				ServerConnections::Client => next_screen.0 = Some(StartScreens::ClientControls),
				ServerConnections::Hosting => next_screen.0 = Some(StartScreens::HostControls),
				_ => {}
			}
		}
	}
}

#[derive(Component)]
struct HostControlsBtn;

fn setup_host_controls_ui(mut commands: Commands, ass: ResMut<AssetServer>) {
	info!("- Host Controls UI");
	commands
		.spawn(NodeBundle {
			style: Style {
				width: Val::Percent(100.),
				height: Val::Percent(100.),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.insert(StartScreenUi)
		.with_children(|parent| {
			parent
				.spawn(ButtonBundle {
					style: Style {
						width: Val::Px(400.),
						height: Val::Px(50.),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						..default()
					},
					background_color: Color::BLACK.into(),
					..default()
				})
				.insert(HostControlsBtn)
				.with_children(|parent| {
					parent.spawn(TextBundle::from_section(
						"Host Public Game",
						TextStyle {
							font: ass.load(Font::Medium),
							font_size: 30.,
							color: Color::WHITE,
						},
					));
				});
		});
}

fn handle_host_controls_ui(
	btn: Query<&Interaction, (With<HostControlsBtn>, Changed<Interaction>)>,
	mut start_game: ResMut<NextState<ServerConnections>>,
	mut in_game: ResMut<NextState<ScreenState>>,
) {
	if let Some(Interaction::Pressed) = btn.iter().next() {
		start_game.0 = Some(ServerConnections::Hosting);
		in_game.0 = Some(ScreenState::InGame);
	}
}

#[derive(Component)]
struct ClientControlsBtn;

fn setup_client_controls_ui(mut commands: Commands, ass: ResMut<AssetServer>) {
	info!("- Client Controls UI");
	commands
		.spawn(NodeBundle {
			style: Style {
				width: Val::Percent(100.),
				height: Val::Percent(100.),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.insert(StartScreenUi)
		.with_children(|parent| {
			parent
				.spawn(ButtonBundle {
					style: Style {
						width: Val::Px(400.),
						height: Val::Px(50.),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						..default()
					},
					background_color: Color::BLACK.into(),
					..default()
				})
				.insert(ClientControlsBtn)
				.with_children(|parent| {
					parent.spawn(TextBundle::from_section(
						"Join Machine-Local Game",
						TextStyle {
							font: ass.load(Font::Medium),
							font_size: 30.,
							color: Color::WHITE,
						},
					));
				});
		});
}

fn handle_client_controls_ui(
	btn: Query<&Interaction, (With<ClientControlsBtn>, Changed<Interaction>)>,
	mut start_game: ResMut<NextState<ServerConnections>>,
	mut in_game: ResMut<NextState<ScreenState>>,
) {
	if let Some(Interaction::Pressed) = btn.iter().next() {
		start_game.0 = Some(ServerConnections::Client);
		in_game.0 = Some(ScreenState::InGame);
	}
}

fn cleanup_ui(mut commands: Commands, ui: Query<Entity, With<StartScreenUi>>) {
	commands.entity(ui.single()).despawn_recursive();
}
