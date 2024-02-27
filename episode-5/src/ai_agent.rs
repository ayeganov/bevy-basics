use bevy::{prelude::*};


#[derive(Component, Debug)]
pub struct AiAgent;


pub struct AiAgentPlugin;


#[derive(Component, Debug, Clone, Reflect)]
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


fn make_decisions()
{
}
