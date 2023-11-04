use super::*;

pub fn apply_thrust(
	thrust: Thrust<FinalVectors>,
	player: &mut ExternalForce,
	time: &Res<Time>,
) -> Thrust<FinalVectors> {
	let delta = time.delta_seconds_f64() as f32;

	impl ControllablePlayer {
		const MAX_TOTAL_ANGULAR_FORCE: f32 = 10_000_000.;
		const MAX_TOTAL_LINEAR_FORCE: f32 = 10_000_000.;
	}

	player.force = (thrust.forward + thrust.up + thrust.right)
		.clamp_length(0., ControllablePlayer::MAX_TOTAL_LINEAR_FORCE);
	player.force *= delta;

	player.torque = (thrust.turn_right + thrust.tilt_up + thrust.roll_right)
		.clamp_length(0., ControllablePlayer::MAX_TOTAL_ANGULAR_FORCE);
	player.torque *= delta;

	// info!("Thrust: (ang len = {})");

	thrust
}

pub fn trigger_player_thruster_particles(
	player: Query<&ControllablePlayer>,
	mut particles: Query<(&mut EffectSpawner, &Thruster)>,
) {
	for ControllablePlayer {
		thrust_responses: thrust,
		..
	} in player.iter()
	{
		impl ThrusterFlags {
			/// If flags match, add relative strength, else add nothing
			fn degree_of_match(&self, actual: &Thrust<ThrustReactionsStage>) -> f32 {
				let flags = self;
				let mut counter = 0.;

				actual.for_each(|reaction, thrust_type| match reaction {
					ThrustReactions::Normal {
						input: Some(actual),
					}
					| ThrustReactions::Braking {
						braking_direction: Some(actual),
					} => {
						if flags[thrust_type].is_some_and(|f| f == *actual) {
							counter += 1.;
						}
					}
					_ => {}
				});

				counter
			}
		}

		for (mut spawner, Thruster { flags, .. }) in particles.iter_mut() {
			// todo: show gradient of particles, change acceleration / lifetime?

			let degree = flags.degree_of_match(thrust);

			if degree > 0. {
				spawner.set_active(true);
			} else {
				spawner.set_active(false);
			}
		}
	}
}
