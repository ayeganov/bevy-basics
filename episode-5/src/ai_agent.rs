use bevy::{prelude::*};
use rand::Rng;

use crate::movement::Velocity;

const ROTATION_SPEED: f32 = 2.5;
const SPEED: f32 = 15.0;


#[derive(Component, Debug)]
pub struct AiAgent;


pub struct AiAgentPlugin;


#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub enum AgentType
{
  #[default]
  Random,
  Human,
  Neat
}


impl Plugin for AiAgentPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(Update, make_decisions);
  }
}


fn make_decisions(mut query: Query<(&mut Transform, &mut Velocity), With<AiAgent>>,
                  time: Res<Time>,
)
{
  let mut rng = rand::thread_rng();
  for (mut transform, mut velocity) in query.iter_mut()
  {
    let mut rotation = 0.0;
    let mut movement = 0.0;

    let do_rotate_right = rng.gen_ratio(1, 2);
    let do_rotate_left = rng.gen_ratio(1, 2);

    if do_rotate_right
    {
      rotation = -ROTATION_SPEED * time.delta_seconds();
    }
    else if do_rotate_left
    {
      rotation = ROTATION_SPEED * time.delta_seconds();
    }

    let do_move_forward = rng.gen_ratio(1, 2);
    let do_move_backward = rng.gen_ratio(1, 8);

    if do_move_backward
    {
      movement = -SPEED;
    }
    else if do_move_forward
    {
      movement = SPEED;
    }

    // Rotate around the Y-axis.
    // Ignores the Z-axis rotation applied below.
    transform.rotate_y(rotation);

    // Update the spaceship's velocity based on new direction.
    velocity.value = transform.forward() * movement;
  }
}
