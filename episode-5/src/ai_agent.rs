use bevy::prelude::*;

use rand::prelude::*;

use crate::ai_framework::Environment;
use crate::movement::Velocity;
use crate::ai_framework::Sensor;
use crate::ai_framework::Sensing;

const ROTATION_SPEED: f32 = 2.5;
const SPEED: f32 = 15.0;


/// What is the purpose of the agent - to make decisions and affect other
/// agents/environments
#[derive(Component, Debug, Default)]
pub struct AiAgent;


/// What is the purpose of an environment - to provide RESOURCES and SENSORY
/// data
#[derive(Component, Debug, Default)]
pub struct AiEnvironment;


/// Universal information processor - chooses what sensory information to
/// process and produces an array of outputs to drive the agents behavior
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub enum Brain
{
  Random(RandomBrain),
  Human,
  Neat
}


#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct RandomBrain
{

}


impl Default for Brain
{
  fn default() -> Self
  {
    Brain::Random(Default::default())
  }
}


impl Default for RandomBrain
{
  fn default() -> Self
  {
    RandomBrain {}
  }
}


pub trait AgentBrain
{
  // TODO: How to collect inputs?
  fn process_input(&mut self, sensations: Vec<f32>) -> Vec<f32>;
}


impl AgentBrain for Brain
{
  fn process_input(& mut self, sensations: Vec<f32>) -> Vec<f32>
  {
    match self
    {
      Brain::Random(brain) => brain.process_input(sensations),
      Brain::Human => vec![],
      Brain::Neat => vec![]
    }
  }
}


impl AgentBrain for RandomBrain
{
  fn process_input(& mut self, sensations: Vec<f32>) -> Vec<f32>
  {
    vec![]
  }
}


pub struct AiAgentPlugin;


impl Plugin for AiAgentPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(Update, (make_decisions, update_agents));
  }
}


fn update_agents(agents_query: Query<(Entity, &AiAgent, &Sensor), (With<AiAgent>, With<Sensor>)>,
                 sensors_query: Query<(Entity, &Sensor), With<Sensor>>,
                 images: Res<Assets<Image>>,
                 time: Res<Time>,
)
{
  for (sensor_ent, sensor) in sensors_query.iter()
  {
    match sensor
    {
      Sensor::Vision(sensing) =>
      {
//        info!("Id of vision: {}", sensing.id);
//        info!("Image address in update_agents: {:?}", &sensing.visual_sensor);
        if let Some(sensing) = sensing.sense(Environment::VisibleEnvironment{}, &images)
        {
          println!("Sensing: {:?}", sensing.len());
        }
        else
        {
//          println!("No sensing");
        }
      }
    }
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
