use bevy::prelude::*;

use rand::prelude::*;

use crate::ai_framework::Environment;
use crate::movement::Velocity;
use crate::ai_framework::Sensor;
use crate::ai_framework::Sensing;
use crate::vision::VisionView;

const ROTATION_SPEED: f32 = 2.5;
const SPEED: f32 = 15.0;


pub enum Action
{
  Shoot,
  Rotate(f32),
  Go(f32)
}


#[repr(usize)]
enum ActionIndex
{
  Rotation = 0,
  Movement = 1,
  Shooting = 2,
}


/// What is the purpose of the agent - to make decisions and affect other
/// agents/environments
#[derive(Component, Debug, Default)]
pub struct Agent;


/// What is the purpose of an environment - to provide RESOURCES and SENSORY
/// data
#[derive(Component, Debug, Default)]
pub struct AiEnvironment;


/// Universal information processor - chooses what sensory information to
/// process and produces an array of outputs to drive the agents behavior
#[derive(Component, Debug, Clone)]
pub enum Brain
{
  Random(RandomBrain),
  Human,
  Neat
}


#[derive(Component, Debug, Clone,)]
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
  fn process_input(&mut self, sensations: &Vec<f32>) -> Vec<f32>;
}


impl AgentBrain for Brain
{
  fn process_input(&mut self, sensations: &Vec<f32>) -> Vec<f32>
  {
    match self
    {
      Brain::Random(brain) => {
        brain.process_input(sensations)
      },
      Brain::Human => {
        vec![]
      }
      Brain::Neat => {
        vec![]
      }
    }
  }
}


impl AgentBrain for RandomBrain
{
  fn process_input(&mut self, _sensations: &Vec<f32>) -> Vec<f32>
  {
    let mut rng = rand::thread_rng();
    let rotation = rng.gen_range(-1.0f32..=1.0f32);
    let movement = rng.gen_range(-1.0f32..=1.0f32);
    let shoot = rng.gen_range(0.0f32..=1.0f32);

    vec![rotation, movement, shoot]
  }
}


pub struct AiAgentPlugin;


impl Plugin for AiAgentPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(Update, update_agents);
  }
}


fn collect_sensations(sensors_query: &Query<&Sensor>,
                      children: &Children,
                      vision_view: &VisionView,
) -> Vec<f32>
{
  // TODO: make sure to deal sensations order once you have more than one type
  // of sensor
  let mut sensations = vec![];
  for &child in children.iter()
  {
    if let Ok(sensor) = sensors_query.get(child)
    {
      match sensor
      {
        Sensor::Vision(sensing) =>
        {
  //        info!("Id of vision: {}", sensing.id);
  //        info!("Image address in update_agents: {:?}", &sensing.visual_sensor);
          if let Some(sensing) = sensing.sense(Environment::VisibleEnvironment{}, &vision_view)
          {
//            println!("Sensing: {:?}", sensing.len());
            sensations.extend(sensing);
          }
          else
          {
  //          println!("No sensing");
          }
        }
      }
    }
  }

  sensations
}


fn brain_process(brain_query: &mut Query<&mut Brain>,
                 children: &Children,
                 sensations: &Vec<f32>
) -> Vec<f32>
{
  let mut outputs = vec![];
  for &child in children.iter()
  {
    if let Ok(mut brain) = brain_query.get_mut(child)
    {
      let brain_out: Vec<_> = brain.process_input(&sensations);
      outputs.extend(brain_out);
      break;
    }
  }
  outputs
}


fn update_agents(agents_query: Query<(Entity, &Children), With<Agent>>,
                 sensors_query: Query<&Sensor>,
                 mut brain_query: Query<&mut Brain>,
                 mut transform_velocity_q: Query<(&mut Transform, &mut Velocity), With<Agent>>,
                 vision_view: VisionView,
                 time: Res<Time>,
)
{
  for (agent_entity, children) in agents_query.iter()
  {
    let sensations = collect_sensations(&sensors_query, &children, &vision_view);

    let brain_output = brain_process(&mut brain_query, &children, &sensations);

    info!("Number of velocities: {}", transform_velocity_q.iter().len());
    if let Ok((mut transform, mut velocity)) = transform_velocity_q.get_mut(agent_entity)
    {
      update_agent_state(&mut transform, &mut velocity, &brain_output, &time);
    }
  }
}


fn update_agent_state(transform: &mut Transform,
                      velocity: &mut Velocity,
                      brain_output: &Vec<f32>,
                      time: &Res<Time>,
)
{
  let mut rotation = 0.0;
  let mut movement = 0.0;

  let do_rotate_right = brain_output[ActionIndex::Rotation as usize] < -0.1;
  let do_rotate_left = brain_output[ActionIndex::Rotation as usize] > 0.1;

  if do_rotate_right
  {
//    info!("right");
    rotation = -ROTATION_SPEED * time.delta_seconds();
  }
  else if do_rotate_left
  {
//    info!("left");
    rotation = ROTATION_SPEED * time.delta_seconds();
  }

  let do_move_forward = brain_output[ActionIndex::Movement as usize] < -0.1;
  let do_move_backward = brain_output[ActionIndex::Movement as usize] > 0.1;

  if do_move_backward
  {
//    info!("backward");
    movement = -SPEED;
  }
  else if do_move_forward
  {
//    info!("forward");
    movement = SPEED;
  }

  // Rotate around the Y-axis.
  // Ignores the Z-axis rotation applied below.
  transform.rotate_y(rotation);

  info!("agent state Velocity: {:?},\ntransform: {:?},\nmovement: {},\nforward: {:?}", velocity, transform, movement, transform.forward());

  let value = transform.forward() * movement;
  info!("velocity value: {}", value);
  velocity.value = transform.forward() * movement;
}
