use bevy::sprite::Mesh2dHandle;

use super::manual_ui::*;
use super::path_tracing::*;
use crate::prelude::*;

/// Plugin
pub struct StartScreen;

impl Plugin for StartScreen {
	fn build(&self, app: &mut App) {
		app
			.add_state::<StartScreenStates>()
			.add_systems(OnEnter(StartScreenStates::Initial), Self::spawn_initial)
			.add_systems(Update, ButtonParticle::follow_parent_bbox);
	}
}

#[derive(States, Component, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
enum StartScreenStates {
	#[default]
	Initial,

	ConfigureHost,

	ConfigureClient,

	ConfigureSolo,
}

impl StartScreen {
	fn spawn_initial(mut commands: Commands, mut mma: MM2, effects: ResMut<Assets<EffectAsset>>) {
		let mut column = ManualColumn {
			const_x: 0.,
			const_width: 100.,
			current_y: 0.,
			item_height: 50.,
			margin: 10.,
		};

		commands
			.spawn(HostGameButtonBundle::new(column.next(), &mut mma))
			.with_children(|parent| {
				parent.spawn(ButtonParticles::new(effects));
			});
	}
}

// todo: add particle effects
#[derive(Bundle)]
struct HostGameButtonBundle {
	mesh: Mesh2dHandle,
	material: Handle<ColorMaterial>,
	spatial: SpatialBundle,
	name: Name,
	path: BevyPath,

	layer: RenderLayers,
}

impl HostGameButtonBundle {
	fn new(manual_node: ManualNode, mma: &mut MM2) -> Self {
		Self {
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
			layer: GlobalRenderLayers::Ui(UiCameras::Center).into(),
		}
	}
}

#[derive(Component)]
struct ButtonParticle;

#[derive(Bundle)]
struct ButtonParticles {
	particles: ParticleEffectBundle,
	marker: ButtonParticle,

	layer: RenderLayers,
	name: Name,
}

impl ButtonParticles {
	fn new(mut effects: ResMut<Assets<EffectAsset>>) -> Self {
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
			EffectAsset::new(32768, Spawner::rate(1000.0.into()), writer.finish())
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
			layer: GlobalRenderLayers::Ui(UiCameras::Center).into(),
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
				let time = time.elapsed_seconds_wrapped().mul(10.) % 1.;
				let pos = path.get_pos_at_time(time);
				transform.translation.x = pos.x;
				transform.translation.y = pos.y;
			} else {
				error!("Particle spawner's parent does not have a BBox component");
			}
		}
	}
}
